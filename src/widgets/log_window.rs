use eframe::egui::{ScrollArea, Window};

use crate::{
    font_icon,
    logger::{DisplayOptions, Record},
    Queue,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Level {
    #[default]
    All = 0,
    Error = 1,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for log::Level {
    fn from(level: Level) -> Self {
        match level {
            Level::All | Level::Error => log::Level::Error,
            Level::Warn => log::Level::Warn,
            Level::Info => log::Level::Info,
            Level::Debug => log::Level::Debug,
            Level::Trace => log::Level::Trace,
        }
    }
}

#[derive(Default)]
pub struct LogWindow {
    trace: Queue<Record>,
    debug: Queue<Record>,
    info: Queue<Record>,
    warn: Queue<Record>,
    error: Queue<Record>,

    active_log: Level,
    opts: DisplayOptions,
}

impl LogWindow {
    pub fn with_caps(trace: usize, debug: usize, info: usize, warn: usize, error: usize) -> Self {
        Self {
            trace: Queue::with_capacity(trace),
            debug: Queue::with_capacity(debug),
            info: Queue::with_capacity(info),
            warn: Queue::with_capacity(warn),
            error: Queue::with_capacity(error),
            active_log: Level::default(),
            opts: DisplayOptions::new(),
        }
    }

    pub fn push(&mut self, record: Record) {
        let queue = match record.level {
            log::Level::Error => &mut self.error,
            log::Level::Warn => &mut self.warn,
            log::Level::Info => &mut self.info,
            log::Level::Debug => &mut self.debug,
            log::Level::Trace => &mut self.trace,
        };
        queue.push(record);
    }

    pub fn iter(
        &self,
        level: log::Level,
    ) -> impl Iterator<Item = &Record> + ExactSizeIterator + '_ {
        let queue = match level {
            log::Level::Error => &self.error,
            log::Level::Warn => &self.warn,
            log::Level::Info => &self.info,
            log::Level::Debug => &self.debug,
            log::Level::Trace => &self.trace,
        };
        queue.iter()
    }

    pub fn all(&self) -> Vec<&Record> {
        let mut list = [
            &self.trace,
            &self.debug,
            &self.info,
            &self.warn,
            &self.error,
        ]
        .into_iter()
        .flat_map(Queue::iter)
        .collect::<Vec<_>>();

        list.sort_unstable_by_key(|k| k.timestamp);
        list
    }

    pub fn display(&mut self, show_logs: &mut bool, ctx: &egui::Context) {
        Window::new("logs")
            .default_height(200.0)
            .resizable(true)
            .collapsible(true)
            .open(show_logs)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for (repr, level) in [
                        ("trace", Level::Trace),
                        ("debug", Level::Debug),
                        ("info", Level::Info),
                        ("warn", Level::Warn),
                        ("error", Level::Error),
                        ("all", Level::All),
                    ] {
                        ui.selectable_value(&mut self.active_log, level, repr);
                    }

                    ui.menu_button("Fields", |ui| {
                        ui.horizontal(|ui| {
                            for (opt, repr, desc) in [
                                (&mut self.opts.index, font_icon::NUMBER, "Toggle index"),
                                (&mut self.opts.level, font_icon::UP_TRIANGLE, "Toggle level"),
                                (
                                    &mut self.opts.timestamp,
                                    font_icon::TIME,
                                    "Toggle timestamp",
                                ),
                                (&mut self.opts.target, font_icon::USER_LIST, "Toggle target"),
                            ] {
                                ui.toggle_value(opt, repr).on_hover_text_at_pointer(desc);
                            }
                        })
                    });
                });

                ui.separator();
                ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            if !matches!(self.active_log, Level::All) {
                                for (i, record) in self.iter(self.active_log.into()).enumerate() {
                                    record.display(self.opts, i, ui);
                                }
                            } else {
                                for (i, record) in self.all().into_iter().enumerate() {
                                    record.display(self.opts, i, ui);
                                }
                            }
                        })
                    });
            });
    }
}
