use std::path::PathBuf;

use egui::{Color32, Id, Label, RichText, Sense};
use log::Level;

use crate::format_seconds;

pub fn init_logger() -> flume::Receiver<Record> {
    log::set_max_level(log::LevelFilter::Trace);
    let (sender, receiver) = flume::unbounded(); // TODO probably should be bounded
    log::set_boxed_logger(Box::new(Logger::new(sender))).unwrap();
    receiver
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct DisplayOptions {
    pub timestamp: bool,
    pub level: bool,
    pub target: bool,
    pub index: bool,
}

impl DisplayOptions {
    pub const fn new() -> Self {
        Self {
            timestamp: true,
            level: true,
            target: true,
            index: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            timestamp: false,
            level: false,
            target: false,
            index: false,
        }
    }
}

impl Default for DisplayOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Record {
    pub timestamp: time::OffsetDateTime,
    pub start: time::OffsetDateTime,
    pub target: Box<str>,
    pub level: Level,
    pub file: Option<&'static str>,
    pub line: Option<u32>,
    pub data: Box<str>,
}

impl Record {
    const fn level(&self) -> &'static str {
        match self.level {
            Level::Error => "error",
            Level::Warn => "warn ",
            Level::Info => "info ",
            Level::Debug => "debug",
            Level::Trace => "trace",
        }
    }

    const fn level_color(&self) -> egui::Color32 {
        match self.level {
            Level::Error => Color32::RED,
            Level::Warn => Color32::YELLOW,
            Level::Info => Color32::GREEN,
            Level::Debug => Color32::from_rgb(0x00, 0xFF, 0xFF),
            Level::Trace => Color32::from_rgb(0xFF, 0x00, 0xFF),
        }
    }

    /// NOTE this assume the timestamp is 9 digits
    fn colorize_timestamp(input: &str) -> [(&str, Color32); 3] {
        const COLORS: [Color32; 3] = [
            Color32::LIGHT_YELLOW,
            Color32::LIGHT_BLUE,
            Color32::LIGHT_GREEN,
        ];
        assert_eq!(input.len(), 9);

        input.as_bytes().chunks(3).enumerate().fold(
            [("", Color32::GRAY); 3],
            |mut f, (i, chunk)| {
                f[i] = (std::str::from_utf8(chunk).expect("valid utf8"), COLORS[i]);
                f
            },
        )
    }

    pub fn display(&self, opts: DisplayOptions, index: usize, ui: &mut egui::Ui) {
        let flex = [opts.timestamp, opts.level, opts.target, opts.index]
            .into_iter()
            .map(|c| c as usize)
            .sum::<usize>()
            > 1;

        let DisplayOptions {
            timestamp,
            level,
            target,
            index: show_index,
        } = opts;

        let dt = self.timestamp - self.start;
        let ts = dt.whole_milliseconds();

        macro_rules! make_left {
            ($ui:expr) => {
                if show_index {
                    $ui.monospace(index.to_string());
                }

                if level {
                    $ui.add(Label::new(
                        RichText::new(self.level())
                            .monospace()
                            .color(self.level_color()),
                    ));
                }

                if timestamp {
                    let resp = $ui
                        .scope(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 0.0;
                            let ts = format!("{ts:0>9}");
                            for (i, (part, color)) in
                                Self::colorize_timestamp(&ts).into_iter().enumerate()
                            {
                                if i != 0 {
                                    ui.monospace(".");
                                }
                                ui.add(Label::new(RichText::new(part).monospace().color(color)));
                            }
                        })
                        .response;
                    if $ui.ctx().input().modifiers.command_only() {
                        resp.on_hover_ui_at_pointer(|ui| {
                            let d = time::OffsetDateTime::now_local().expect("system clock")
                                - self.timestamp;
                            let secs = d.whole_seconds();

                            let label = if secs < 10 {
                                format!("{}ms ago", d.whole_milliseconds())
                            } else {
                                format!("{} ago", format_seconds(secs as u64))
                            };

                            ui.label(label);
                        });
                    }
                }

                if target {
                    $ui.monospace(&*self.target);
                }
            };
        }

        let resp = ui
            .vertical(|ui| {
                if !flex {
                    ui.horizontal_wrapped(|ui| {
                        make_left!(ui);
                        ui.add(Label::new(
                            RichText::new(&*self.data)
                                .monospace()
                                .color(ui.style().visuals.strong_text_color()),
                        ))
                    })
                    .inner
                } else {
                    ui.vertical(|ui| {
                        ui.horizontal_wrapped(|ui| {
                            make_left!(ui);
                        });
                        ui.add(
                            Label::new(
                                RichText::new(&*self.data)
                                    .monospace()
                                    .color(ui.style().visuals.strong_text_color()),
                            )
                            .wrap(true),
                        )
                    })
                    .inner
                }
            })
            .inner;

        let resp = ui.interact(resp.rect, Id::new(self.timestamp), Sense::click());

        if resp.double_clicked() {
            if let Some((file, line)) = self.file.and_then(|file| Some((file, self.line?))) {
                ui.output().open_url(format!(
                    "vscode://file:/{}:{line}",
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join(file)
                        .to_string_lossy()
                ));
            }
        }

        if ui.ctx().input().modifiers.command_only() {
            resp.on_hover_ui_at_pointer(|ui| {
                if let Some((file, line)) = self.file.and_then(|file| Some((file, self.line?))) {
                    ui.horizontal(|ui| {
                        ui.add(Label::new(
                            RichText::new(self.level())
                                .monospace()
                                .color(self.level_color()),
                        ));

                        ui.monospace(&*self.target);
                        ui.scope(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 0.0;
                            ui.horizontal(|ui| {
                                ui.monospace(file);
                                ui.monospace(":");
                                ui.monospace(line.to_string())
                            });
                        });
                    });
                }
            });
        }
    }
}

struct Logger {
    sender: flume::Sender<Record>,
    start: time::OffsetDateTime,
}

impl Logger {
    fn new(sender: flume::Sender<Record>) -> Self {
        Self {
            sender,
            start: time::OffsetDateTime::now_local().expect("system must have a clock"),
        }
    }

    fn log_it(&self, record: &log::Record<'_>) {
        let metadata = record.metadata();
        let args = record.args();

        let timestamp = time::OffsetDateTime::now_local().expect("system must have a clock");
        let _ = self.sender.send(Record {
            timestamp,
            start: self.start,
            target: Box::from(metadata.target()),
            level: metadata.level(),
            data: Box::from(&*args.to_string()),
            file: record.file_static(),
            line: record.line(),
        });
    }

    fn is_from_our_pkg(record: &log::Record<'_>) -> bool {
        record
            .module_path()
            .and_then(|module| module.split_once("::"))
            .filter(|&(head, _)| head == env!("CARGO_PKG_NAME"))
            .is_some()
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record<'_>) {
        if !Self::is_from_our_pkg(record) {
            return;
        }

        self.log_it(record)
    }

    fn flush(&self) {}
}
