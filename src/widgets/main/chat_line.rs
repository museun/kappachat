use std::collections::HashMap;

use egui::{Label, TextStyle};

use time::OffsetDateTime;

use crate::{
    twitch::{self, EmoteSpan},
    ImageCache,
};

use super::Timestamp;

pub struct ChatLine {
    pub ts: Timestamp,
    pub id: uuid::Uuid,
    pub spans: Vec<EmoteSpan>,
    pub msg: twitch::Message,
}

pub struct ChatLineView<'a> {
    line: &'a ChatLine,
    cache: &'a ImageCache,
    emote_map: &'a HashMap<String, String>,
    show_timestamp: bool,
}

impl<'a> ChatLineView<'a> {
    pub const fn new(
        line: &'a ChatLine,
        cache: &'a ImageCache,
        emote_map: &'a HashMap<String, String>,
        show_timestamp: bool,
    ) -> Self {
        Self {
            line,
            cache,
            emote_map,
            show_timestamp,
        }
    }

    pub fn display(&self, ui: &mut egui::Ui) {
        let pm = self.line.msg.as_privmsg().expect("this must be a privmsg");

        ui.horizontal_wrapped(|ui| {
            if self.show_timestamp {
                ui.small(self.line.ts.as_str())
                    .on_hover_ui_at_pointer(|ui| {
                        let s = OffsetDateTime::now_local().unwrap() - self.line.ts.date_time;
                        ui.small(format!(
                            "{} ago",
                            crate::format_seconds(s.whole_seconds() as _)
                        ));
                    });
            }

            ui.scope(|ui| {
                let width = ui
                    .fonts()
                    .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');
                ui.spacing_mut().item_spacing.x = width;

                // if let Some((badge, version)) = pm.badges().next() {
                //     if let Some(img) = self.cache.get(badge) {
                //         img.show_size(ui, vec2(8.0, 8.0));
                //         // .on_hover_text_at_pointer(self.emote_map.get(badge).unwrap());
                //     }
                // }

                ui.colored_label(pm.color(), pm.sender);

                for spans in &self.line.spans {
                    match spans {
                    EmoteSpan::Emote(s) =>
                    // match self.cache.get(s) {
                        // Some(img) => {
                        //     img.show_size(ui, vec2(16.0, 16.0))
                        //         .on_hover_text_at_pointer(self.emote_map.get(s).unwrap());
                        // }
                        // None => {
                           { ui.add(Label::new(s));}
                        // }
                    // },
                    EmoteSpan::Text(s) => {
                        ui.add(Label::new(s));
                    }
                }
                }
            });
        });
    }
}
