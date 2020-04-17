//! A button that loads the current program file.

use super::widget_impl_support::*;

use lc3_traits::control::load::{load_whole_memory_dump, Progress, LoadMemoryProgress};

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use crossbeam::thread::scope;
use std::fs;
use lc3_assembler::lexer::Lexer;
use lc3_assembler::parser::{parse, LeniencyLevel};
use lc3_assembler::error::extract_file_errors;
use lc3_assembler::assembler::assemble;
use lc3_shims::memory::FileBackedMemoryShim;
use annotate_snippets::display_list::{DisplayList, FormatOptions};
use annotate_snippets::snippet::{Snippet, Annotation, Slice, AnnotationType, SourceAnnotation};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Attempt {
    Success(Instant),
    Failure(Instant),
}

impl Attempt {
    fn failed() -> Option<Attempt> {
        Some(Self::Failure(Instant::now()))
    }

    fn succeeded() -> Option<Attempt> {
        Some(Self::Success(Instant::now()))
    }

    fn expired(&self, dur: Duration) -> bool {
        Instant::now().duration_since(match self {
            Self::Success(i) => *i,
            Self::Failure(i) => *i,
        }) >= dur
    }

    fn message(&self) -> TuiText<'static> {
        match self {
            // Self::Failure(_) => TuiText::styled("Failed to Load!", Style::default().fg(Colour::Red)),
            // Self::Success(_) => TuiText::styled("Successfully Loaded!", Style::default().fg(Colour::Green)),
            Self::Failure(_) => TuiText::styled("Failed!", Style::default().fg(Colour::Red)),
            Self::Success(_) => TuiText::styled("Successful!", Style::default().fg(Colour::Green)),
        }
    }
}

// No block!
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadButton {
    area: Option<Rect>,
    attempt: Option<Attempt>,
    fullscreen_load: bool,
}

impl LoadButton {
    pub fn new() -> Self {
        Self {
            area: None,
            attempt: None,
            fullscreen_load: true,
        }
    }

    fn split_for_text_and_gauge(area: Rect) -> (Rect, Rect) {
        if let [text, gauge] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(66), Constraint::Percentage(34)].as_ref())
            .split(area)
            [..] {
            (text, gauge)
        } else {
            unreachable!()
        }
    }

    fn load<'a, C, B>(&self, sim: &mut C, terminal: &mut Terminal<B>, path: &PathBuf) -> Result<String, String>
    where
        C: Control + ?Sized + 'a,
        B: Backend,
        Terminal<B>: Send,
    {
        let p = format!("{}", path.display());

        if !path.exists() {
            return Err(format!("`{}` does not exist!", p))
        }

        let assembled_file_path = if file_requires_assembly(path) {
            let path_str = path.clone().into_os_string().into_string().unwrap();
            let string = fs::read_to_string(path).unwrap();
            let src = string.as_str();
            let lexer = Lexer::new(src);
            let cst = parse(lexer, LeniencyLevel::Lenient);

            let errors = extract_file_errors(cst.clone());
            if errors.len() > 0 {
                let mut error_string = String::new();
                for error in errors {
                    let label_string = error.message();
                    let label = label_string.as_str();
                    let annotations = error.annotations();
                    let slices = slices(annotations, src, Some(path_str.as_str()));
                    let snippet = create_snippet(label, slices);
                    let dl = DisplayList::from(snippet);
                    error_string = format!("{}\n{}", error_string, dl);
                }
                let error_string = error_string.replace("\n", "\n|");
                return Err(error_string);
            }
            let background = Some(lc3_os::OS_IMAGE.clone());
            let mem = assemble(cst.objects, background);  // TODO: can still fail. fix in assembler.

            let mut output_path = PathBuf::from(path_str);
            output_path.set_extension("mem");
            let mut file_backed_mem = FileBackedMemoryShim::with_initialized_memory(output_path.clone(), mem);
            file_backed_mem.flush_all_changes().unwrap();
            output_path.clone()
        } else {
            path.clone()
        };

        let shim = lc3_shims::memory::FileBackedMemoryShim::from_existing_file(&assembled_file_path)
            .map_err(|e| format!("Failed to load `{}` as a MemoryDump; got: {:?}", p, e))?;

        let progress = Progress::new_with_time().unwrap();

        scope(|s| {
            let (send, recv) = mpsc::channel();

            let handle = s.spawn(|_| {

                let recv = (move || recv)();

                loop {
                    match recv.try_recv() {
                        Ok(()) => break (),
                        Err(mpsc::TryRecvError::Empty) => {
                            // Update our progress bar:
                            terminal.draw(|mut f| {
                                let area = if self.fullscreen_load {
                                    f.size()
                                } else if let Some(area) = self.area {
                                    area
                                } else {
                                    return; // don't draw if we don't have a rect
                                };

                                let (_, area) = Self::split_for_text_and_gauge(area);

                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                                    .split(area);

                                let (gauge, info) = (chunks[0], chunks[1]);

                                // If the screen changed and we're not displayed
                                // anymore, skip the redraw (so we don't crash).
                                if f.size().intersection(area) != area {
                                    return;
                                }

                                Gauge::default()
                                    // .block(Block::default().borders(Borders::ALL).title("Progress"))
                                    .style(Style::default().fg(Colour::Green).bg(Colour::Black).modifier(Modifier::ITALIC | Modifier::BOLD))
                                    .ratio(progress.progress().max(0.0f32).into())
                                    .render(&mut f, gauge);

                                let success_rate = format!("{:.2}%", progress.success_rate() * 100.0);

                                let time_remaining = progress
                                    .estimate_time_remaining()
                                    .and_then(|d| chrono::Duration::from_std(d).ok())
                                    .map(|d| {
                                        if d.num_seconds() > 60 {
                                            format!("{}m {}.{:03}s", d.num_minutes(), d.num_seconds() % 60, d.num_milliseconds() % 1000)
                                        } else {
                                            format!("{}.{:03}s", d.num_seconds(), d.num_milliseconds() % 1000)
                                        }
                                    })
                                    .unwrap_or("Unknown".to_string());

                                Paragraph::new([
                                        TuiText::styled(format!("~{}", time_remaining), Style::default().fg(Colour::Green)),
                                        TuiText::styled(format!(" // "), Style::default().fg(Colour::Gray)),
                                        TuiText::styled(format!("{} success", success_rate), Style::default().fg(Colour::Magenta)),
                                    ].iter())
                                    .style(Style::default().fg(Colour::White))
                                    .alignment(Alignment::Center)
                                    .wrap(true)
                                    .render(&mut f, info);
                            }).unwrap();

                            std::thread::sleep(Duration::from_millis(30))
                        },
                        Err(mpsc::TryRecvError::Disconnected) => panic!(),
                    }
                }
            });

            let res = load_whole_memory_dump(sim, &shim.into(), Some(&progress));

            send.send(()).unwrap();
            handle.join().unwrap();
            res
        }).unwrap()
        .map_err(|e| format!("Error during load: {:?}", e))
        .map(|_| format!("Successful Load (`{}`)! Finished in {}.", p, chrono::Duration::from_std(progress.time_elapsed().unwrap()).unwrap()))
    }
}

fn file_requires_assembly(path: &PathBuf) -> bool {
    return match path.extension() {
        Some(ext) => ext == "asm",
        _ => false,
    }
}

fn create_snippet<'input>(label: &'input str, slices: Vec<Slice<'input>>) -> Snippet<'input> {
    Snippet {
        title: Some(Annotation {
            label: Some(label),
            id: None,
            annotation_type: AnnotationType::Error
        }),
        footer: vec![],
        slices,
        opt: FormatOptions { color: true, anonymized_line_numbers: false }
    }
}

fn slices<'input>(annotations: Vec<SourceAnnotation<'input>>, source: &'input str, origin: Option<&'input str>) -> Vec<Slice<'input>> {
    let mut slices = Vec::new();
    if !annotations.is_empty() {
        slices.push(
            Slice {
                source,
                origin,
                line_start: 1,
                fold: true,
                annotations,
            }
        );
    }
    slices
}

impl TuiWidget for LoadButton {
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer) {
        unimplemented!("Don't call this! We need TuiData to draw!")
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for LoadButton
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: Send,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        self.area = Some(area);

        if let Some(ref a) = self.attempt {
            if a.expired(Duration::from_secs(3)) {
                drop(self.attempt.take());
            }
        }

        match &data.program_path {
            None => {
                let msg = TuiText::styled("No Program File!\n", Style::default().fg(Colour::Red));

                Paragraph::new([msg].iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .draw(area, buf)
            },

            Some(p) => {
                let file_name = p.file_name()
                    .and_then(|f| f.to_str())
                    .unwrap_or("<unprintable>");

                let msg1 = TuiText::styled(format!("Load Program (from: `{}`)", file_name), Style::default().fg(Colour::Cyan));

                let (text, gauge) = Self::split_for_text_and_gauge(area);

                Paragraph::new([msg1].iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .draw(text, buf);

                match &self.attempt {
                    Some(attempt) => Paragraph::new([attempt.message()].iter())
                        .style(Style::default())
                        .alignment(Alignment::Center)
                        .wrap(true)
                        .draw(gauge, buf),

                    None => Gauge::default()
                        // .block(Block::default().borders(Borders::ALL))
                        .style(Style::default().fg(Colour::Cyan).modifier(Modifier::ITALIC | Modifier::DIM))
                        .percent(0)
                        .draw(gauge, buf)
                }
            }
        }
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;

        match event {
            Focus(FocusEvent::GotFocus) => data.program_path.is_some(),
            Focus(FocusEvent::LostFocus) => false,
            Mouse(MouseEvent::Up(_, _, _, _)) => data.program_path.is_some(),

            Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL } ) | Key(KeyEvent { code: KeyCode::Enter, .. } ) | Mouse(MouseEvent::Down(_, _, _, _)) => {
                // Timeout so we don't repeated try again on key mashes / accidental
                // double clicks
                if self.attempt.is_some() {
                    return false
                }

                match data.program_path {
                    Some(ref p) => {
                        match self.load(data.sim, terminal, p) {
                            Ok(msg) => {
                                self.attempt = Attempt::succeeded();
                                data.log(format!("[Load] {}\n", msg), Colour::Green)
                            },
                            Err(msg) => {
                                self.attempt = Attempt::failed();
                                data.log(format!("[Load] {}\n", msg), Colour::Red)
                            },
                        }

                        data.flush_events();
                        true
                    },
                    None => false,
                }
            }

            _ => false,
        }
    }
}
