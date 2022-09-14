use egui::Color32;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

use crate::twitch::EmoteSpan;

#[derive(Clone)]
pub struct Timestamp {
    pub(crate) date_time: OffsetDateTime,
    pub(crate) repr: String,
}

impl Timestamp {
    pub fn now_local() -> Self {
        static FORMAT: &[FormatItem<'static>] = format_description!("[hour]:[minute]:[second]");
        let date_time = OffsetDateTime::now_local().expect("valid time");
        let repr = date_time.format(&FORMAT).expect("valid time");
        Self { date_time, repr }
    }
}

impl std::fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Timestamp {
    pub fn as_str(&self) -> &str {
        &self.repr
    }
}

#[derive(Clone)]
pub struct TwitchLine {
    pub timestamp: Timestamp,
    pub source: String,
    pub sender: String,
    pub data: String,
    pub color: Color32,
    pub spans: Vec<EmoteSpan>,
}

impl TwitchLine {
    pub fn new(
        sender: impl ToString,
        source: impl ToString,
        data: impl ToString,
        spans: Vec<EmoteSpan>,
    ) -> Self {
        Self {
            timestamp: Timestamp::now_local(),
            sender: sender.to_string(),
            source: source.to_string(),
            data: data.to_string(),
            color: Color32::WHITE,
            spans,
        }
    }

    pub fn with_color(self, color: impl Into<Color32>) -> Self {
        Self {
            color: color.into(),
            ..self
        }
    }
}

impl std::fmt::Debug for TwitchLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Line")
            .field("timestamp", &self.timestamp)
            .field("sender", &self.sender)
            .field("data", &self.data)
            .finish()
    }
}
