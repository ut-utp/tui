//! A button that loads the current program file.

use super::widget_impl_support::*;

use lc3_traits::control::load::{load_whole_memory_dump, Progress, LoadMemoryProgress};

use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use crossbeam::thread::scope;

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
            Self::Failure(_) => TuiText::styled("Failed to Load!", Style::default().fg(Colour::Red)),
            Self::Success(_) => TuiText::styled("Successfully Loaded!", Style::default().fg(Colour::Green)),
        }
    }
}

// No block!
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoadButton {
    area: Option<Rect>,
    attempt: Option<Attempt>
}

impl LoadButton {
    pub fn new() -> Self {
        Self {
            area: None,
            attempt: None,
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

        let shim = lc3_shims::memory::FileBackedMemoryShim::from_existing_file(path)
            .map_err(|e| format!("Failed to load `{}` as a MemoryDump; got: {:?}", p, e))?;


        // // TODO: scoped thread to display progress!
        // let _ = load_whole_memory_dump(sim, &shim.into(), Some(&progress))
        //     .map_err(|e| format!("Error during load: {:?}", e))?;


        scope(|s| {
            let (send, recv) = mpsc::channel();
            let progress = Progress::new();

            let handle = s.spawn(|_| {

                let recv = (move || recv)();

                loop {
                    match recv.try_recv() {
                        Ok(()) => break (),
                        Err(mpsc::TryRecvError::Empty) => {
                            // Update our progress bar:
                            if let Some(area) = self.area {

                                let (_, area) = Self::split_for_text_and_gauge(area);

                                let chunks = Layout::default()
                                    .direction(Direction::Vertical)
                                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                                    .split(area);

                                let (gauge, info) = (chunks[0], chunks[1]);

                                terminal.draw(|mut f| {
                                    if !f.size().intersects(area) {
                                        return;
                                    }

                                    Gauge::default()
                                        // .block(Block::default().borders(Borders::ALL).title("Progress"))
                                        .style(Style::default().fg(Colour::Green).bg(Colour::Black).modifier(Modifier::ITALIC | Modifier::BOLD))
                                        .ratio(progress.progress().max(0.0f32).into())
                                        .render(&mut f, gauge);

                                    let success_rate = format!("{:.2}%", progress.success_rate());

                                    let time_remaining = progress
                                        .estimate_time_remaining()
                                        .and_then(|d| chrono::Duration::from_std(d).ok())
                                        .map(|d| format!("{}", d))
                                        .unwrap_or("Unknown".to_string());

                                    Paragraph::new([
                                            TuiText::styled(format!("{} remaining", time_remaining), Style::default().fg(Colour::Green)),
                                            TuiText::styled(format!(" // "), Style::default().fg(Colour::Gray)),
                                            TuiText::styled(format!("{} success", success_rate), Style::default().fg(Colour::Magenta)),
                                        ].iter())
                                        .style(Style::default().fg(Colour::White))
                                        .alignment(Alignment::Center)
                                        .wrap(true)
                                        .render(&mut f, info);
                                }).unwrap();
                            }

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
        .map(|_| format!("Successful Load (`{}`)!", p))
    }
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

                let msg1 = TuiText::styled("Load Program\n", Style::default().fg(Colour::Cyan));
                let msg2 = TuiText::styled(format!("(from: `{}`)", file_name), Style::default().fg(Colour::Gray));

                let (text, gauge) = Self::split_for_text_and_gauge(area);

                Paragraph::new([msg1, msg2].iter())
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
            Mouse(MouseEvent::Up(_, _, _, _)) => true,

            Mouse(MouseEvent::Down(_, _, _, _)) => {
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

                        true
                    },
                    None => false,
                }
            }

            _ => false,
        }
    }
}
