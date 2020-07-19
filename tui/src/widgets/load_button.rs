//! A button that loads the current program file.

use crate::tui::program_source::{
    ProgramSource,
    file_requires_assembly,
    assemble_mem_dump,
};
use super::widget_impl_support::*;

use lc3_traits::control::load::{load_whole_memory_dump, Progress, LoadMemoryProgress};
use lc3_traits::control::metadata::ProgramId;

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::Builder as ThreadBuilder;

use crossbeam::thread::scope;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Attempt {
    Success(Instant),
    Failure(Instant),
}

impl Attempt {
    #[cfg(not(target_arch = "wasm32"))]
    fn failed() -> Option<Attempt> {
        // TODO: fix time for wasm! (WASM-TIME-FIX)
        Some(Self::Failure(Instant::now()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn succeeded() -> Option<Attempt> {
        // TODO: fix time for wasm! (WASM-TIME-FIX)
        Some(Self::Success(Instant::now()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn expired(&self, dur: Duration) -> bool {
        // TODO: fix time for wasm! (WASM-TIME-FIX)
        Instant::now().duration_since(match self {
            Self::Success(i) => *i,
            Self::Failure(i) => *i,
        }) >= dur
    }

    // TODO: these are stubs! fix them! (WASM-TIME-FIX)
    #[cfg(target_arch = "wasm32")]
    fn failed() -> Option<Attempt> { None }
    #[cfg(target_arch = "wasm32")]
    fn succeeded() -> Option<Attempt> { None }
    #[cfg(target_arch = "wasm32")]
    fn expired(&self, _dur: Duration) -> bool { true }

    fn message(&self) -> TuiText<'static> {
        match self {
            Self::Failure(_) => TuiText::styled(s!(FailureMsg), Style::default().fg(c!(Error))),
            Self::Success(_) => TuiText::styled(s!(SuccessMsg), Style::default().fg(c!(Success))),
        }
    }
}

// No block!
#[derive(Debug)]
pub struct LoadButton {
    area: Option<Rect>,
    attempt: Option<Attempt>,
    last_file_check_time: Mutex<Option<SystemTime>>, // We don't actually need this field to be Sync but we need the struct to be Sync which is why we're using a Mutex instead of just a Cell.
    program_is_out_of_date: Arc<Mutex<bool>>,
    assembler_background_thread_running: Arc<Mutex<bool>>,
    fullscreen_load: bool,
}

impl LoadButton {
    pub fn new() -> Self {
        Self {
            area: None,
            attempt: None,
            fullscreen_load: true,

            last_file_check_time: Mutex::new(None),
            program_is_out_of_date: Arc::new(Mutex::new(false)),
            assembler_background_thread_running: Arc::new(Mutex::new(false)),
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

/*    #[cfg(target_arch = "wasm32")]
    fn load<'a, C, B>(&self, sim: &mut C, terminal: &mut Terminal<B>, path: &PathBuf, with_os: bool) -> Result<String, String>
    where
        C: Control + ?Sized + 'a,
        B: Backend,
    {
        todo!()
    }*/

    fn load<'a, C, B>(&self, sim: &mut C, terminal: &mut Terminal<B>, src: &ProgramSource, with_os: bool) -> Result<String, String>
    where
        C: Control + ?Sized + 'a,
        B: Backend,
        Terminal<B>: ConditionalSendBound,
    {
        // TODO: don't bother writing out the assembled program to a file; just
        // use the already in-memory MemoryDump. (update: this is done)
        //
        // Better yet use the loadable iterator thing. (this is not done)

        // TODO: spin this off into its own module and introduce an abstraction
        // over the program source (i.e. can come from files, URLs, etc; should
        // work on wasm too).

        if src.requires_assembly() {
            terminal.draw(|mut f| {
                // TODO: spin this boilerplate into a function.
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

                let (_, info) = (chunks[0], chunks[1]);

                // If the screen changed and we're not displayed
                // anymore, skip the redraw (so we don't crash).
                if f.size().intersection(area) != area {
                    return;
                }

                let text = [
                    TuiText::styled(format!("Assembling..\n"), Style::default().fg(c!(InProgress))),
                ];
                let para = Paragraph::new(text.iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true);

                f.render_widget(para, info);
            });
        }

        let (memory_dump, metadata) = src.to_memory_dump(with_os)?;

        // TODO: fix time for wasm! (WASM-TIME-FIX)
        #[cfg(not(target_arch = "wasm32"))]
        let progress = Progress::new_with_time().unwrap();
        #[cfg(target_arch = "wasm32")]
        let progress = Progress::new();

        scope(|s| {
            #[cfg(not(target_arch = "wasm32"))]
            let (send, recv) = mpsc::channel();

            #[cfg(not(target_arch = "wasm32"))]
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

                                f.render_widget(
                                    Gauge::default()
                                        // .block(Block::default().borders(Borders::ALL).title("Progress"))
                                        .style(Style::default().fg(Colour::Green).bg(Colour::Black).modifier(Modifier::ITALIC | Modifier::BOLD))
                                        .ratio(progress.progress().min(1.0f32).max(0.0f32).into()),
                                    gauge,
                                );

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

                                let text = [
                                    TuiText::styled(format!("~{}", time_remaining), Style::default().fg(Colour::Green)),
                                    TuiText::styled(format!(" // "), Style::default().fg(Colour::Gray)),
                                    TuiText::styled(format!("{} success", success_rate), Style::default().fg(Colour::Magenta)),
                                ];
                                let para = Paragraph::new(text.iter())
                                    .style(Style::default().fg(Colour::White))
                                    .alignment(Alignment::Center)
                                    .wrap(true);

                                f.render_widget(para, info);
                            }).unwrap();

                            std::thread::sleep(Duration::from_millis(30))
                        },
                        Err(mpsc::TryRecvError::Disconnected) => panic!(),
                    }
                }
            });

            #[cfg(target_arch = "wasm32")]
            {
                // TODO: wasm draw "Loading..."
            }

            let res = load_whole_memory_dump(sim, &memory_dump, Some(&progress));

            #[cfg(not(target_arch = "wasm32"))]
            {
                send.send(()).unwrap();
                handle.join().unwrap();
            }

            res
        }).unwrap()
        .map_err(|e| format!("Error during load: {:?}", e))
        .map(|_| {
            *self.program_is_out_of_date.lock().unwrap() = false;
            // TODO: time on wasm (WASM-TIME-FIX)
            #[cfg(not(target_arch = "wasm32"))]
            { *self.last_file_check_time.lock().unwrap() = Some(SystemTime::now()); }

            // Update the metadata:
            sim.set_program_metadata(metadata);

            // TODO: time on wasm (WASM-TIME-FIX)
            #[cfg(not(target_arch = "wasm32"))]
            let x = format!(
                "Successful Load (`{}`)! Finished in {}.",
                src,
                chrono::Duration::from_std(progress.time_elapsed().unwrap()).unwrap(),
            );
            #[cfg(target_arch = "wasm32")]
            let x = format!("Successful Load (`{}`)!", src);
            x
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn check_for_program_changes<C: Control + ?Sized>(&self, sim: &C, p: &PathBuf, with_os: bool) {
        use lc3_shims::memory::FileBackedMemoryShim;

        // An optimization would be to check if we're already out of date (and
        // then to just not do any additional checks if so). Unfortunately if
        // the file gets modified and then changed back we want to correctly say
        // that a reload isn't needed so we can't do this.

        // Check if we're already up to date:
        // (if the last check time is more recent than the file modification time)
        let last_file_check_time = self.last_file_check_time.lock().unwrap().unwrap_or(SystemTime::UNIX_EPOCH);
        let file_modified_at = p.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);

        if file_modified_at <= last_file_check_time { return; }

        *self.last_file_check_time.lock().unwrap() = Some(SystemTime::now());

        let current_hash = sim.get_program_metadata().id;

        if file_requires_assembly(p) {
            let running = self.assembler_background_thread_running.lock().unwrap();

            // If we're already running an assembler thread, bail.
            if *running { return }
            drop(running);

            // Otherwise, re-assemble and check if the actual memory dump is any
            // different.
            let out_of_date = self.program_is_out_of_date.clone();
            let running = self.assembler_background_thread_running.clone();
            let path = p.clone();

            ThreadBuilder::new()
                .name("TUI: Assembler Background Thread".to_string())
                .stack_size(32 * 1024 * 1024)
                .spawn(move || {
                    if let Ok(mem) = assemble_mem_dump(&path, with_os)  {
                        if ProgramId::new(&mem) != current_hash {
                            *out_of_date.lock().unwrap() = true;
                        } else {
                            // This covers the case where the file switched _back_.
                            *out_of_date.lock().unwrap() = false;
                        }
                    } else {
                        // Since the program no longer assembles things are
                        // indeed out of date.
                        *out_of_date.lock().unwrap() = true;

                        // Don't report errors here; they'll know when they try
                        // to load the program.
                    }

                    *running.lock().unwrap() = false;
                })
                .unwrap();

        } else {
            if let Ok(f) = FileBackedMemoryShim::from_existing_file(p) {
                if f.metadata.id != current_hash {
                    *self.program_is_out_of_date.lock().unwrap() = true;
                } else {
                    // This covers the case where the file switched _back_.
                    *self.program_is_out_of_date.lock().unwrap() = false;
                }
            } else {
                // If it failed to load, let's call it out of date.
                *self.program_is_out_of_date.lock().unwrap() = true;
            }
        }
    }
}

impl<'a, 'int, C, I, O, B> Widget<'a, 'int, C, I, O, B> for LoadButton
where
    C: Control + ?Sized + 'a,
    I: InputSink + ?Sized + 'a,
    O: OutputSource + ?Sized + 'a,
    B: Backend,
    Terminal<B>: ConditionalSendBound,
{
    fn draw(&mut self, data: &TuiData<'a, 'int, C, I, O>, area: Rect, buf: &mut Buffer) {
        self.area = Some(area);

        if let Some(ref a) = self.attempt {
            if a.expired(Duration::from_secs(3)) {
                drop(self.attempt.take());
            }
        }

        match &data.program_source {
            None => {
                let msg = TuiText::styled("No Program Given!\n", Style::default().fg(c!(Error)));

                Paragraph::new([msg].iter())
                    .style(Style::default().fg(Colour::White))
                    .alignment(Alignment::Center)
                    .wrap(true)
                    .render(area, buf)
            },

            Some(s) => {
                let (text, gauge) = Self::split_for_text_and_gauge(area);

                #[cfg(not(target_arch = "wasm32"))]
                let () = if let ProgramSource::FilePath(p) = s {
                    self.check_for_program_changes(data.sim, p, data.use_os);
                };

                // Paragraph::new([msg1].iter())
                //     .style(Style::default().fg(Colour::White))
                //     .alignment(Alignment::Center)
                //     .wrap(true)
                //     .draw(text, buf);

                match &self.attempt {
                    Some(attempt) => Paragraph::new([attempt.message()].iter())
                        .style(Style::default())
                        .alignment(Alignment::Center)
                        .wrap(true)
                        .render(gauge, buf),

                    None => {
                        let name = s.to_string();
                        let name = trim_to_width(&name, area.width - 2);

                        let msg1 = TuiText::styled(format!("`{}`", name), Style::default().fg(c!(LoadText)).bg(
                            if *self.program_is_out_of_date.lock().unwrap() { c!(LoadPendingChanges) } else { c!(LoadNoChanges) }
                        ));

                        Paragraph::new([msg1].iter())
                            .style(Style::default().fg(Colour::White))
                            .alignment(Alignment::Center)
                            .wrap(true)
                            .render(area, buf);
                    }
                }
            }
        }
    }

    fn update(&mut self, event: WidgetEvent, data: &mut TuiData<'a, 'int, C, I, O>, terminal: &mut Terminal<B>) -> bool {
        use WidgetEvent::*;

        match event {
            Focus(FocusEvent::GotFocus) => data.program_source.is_some(),
            Focus(FocusEvent::LostFocus) => false,
            Mouse(MouseEvent::Up(_, _, _, _)) => data.program_source.is_some(),

            Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL } ) | Key(KeyEvent { code: KeyCode::Enter, .. } ) | Mouse(MouseEvent::Down(_, _, _, _)) => {
                // Timeout so we don't repeated try again on key mashes / accidental
                // double clicks
                if self.attempt.is_some() {
                    return false
                }

                match data.program_source {
                    Some(ref p) => {
                        match self.load(data.sim, terminal, p, data.use_os) {
                            Ok(msg) => {
                                self.attempt = Attempt::succeeded();
                                data.log(format!("[Load] {}\n", msg), c!(Success))
                            },
                            Err(msg) => {
                                self.attempt = Attempt::failed();
                                data.log(format!("[Load] {}\n", msg), c!(Error))
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
