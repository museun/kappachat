use egui::{Color32, Label, Layout, Response, RichText};

use crate::{
    chat_layout::ChatLayout,
    tabs::{Line, Tab},
    twitch::TextKind,
};

pub struct LineWidget<'a> {
    pub line: &'a Line,
    pub tab: &'a Tab,
    // cached_images: &'a CachedImages,
}

impl<'a> egui::Widget for LineWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let line = match self.line {
            Line::Twitch { line } => line,
            Line::Status { msg } => return ui.label(msg.clone()),
        };

        let ts = self
            .tab
            .showing_timestamp()
            .then(|| Label::new(RichText::new(line.timestamp.as_str()).weak()));

        let sender = Label::new(RichText::new(&line.sender).color(line.color));

        let data: Box<dyn FnOnce(&mut egui::Ui) -> Response> =
            if !line.spans.iter().any(|c| matches!(c, TextKind::Emote(..))) {
                Box::new(move |ui: &mut egui::Ui| {
                    ui.add(Label::new(RichText::new(&line.data).color(Color32::WHITE)).wrap(true))
                }) as _
            } else {
                Box::new(move |ui: &mut egui::Ui| {
                    ui.add(Label::new(RichText::new(&line.data).color(Color32::WHITE)).wrap(true))
                }) as _

                // let font_id = TextStyle::Body.resolve(&*ui.style());
                // Box::new(move |ui: &mut egui::Ui| ui.small("asdf")) as _
            };

        // let job = line
        //     .spans
        //     .iter()
        //     .fold(LayoutJob::default(), |mut layout, kind| match kind {
        //         TextKind::Emote(id) => {
        //             let id = self.cached_images.emote_map[id];
        //             let img = &self.cached_images.map[&id];

        //             todo!();
        //         }
        //         TextKind::Text(text) => layout.simple(text, font_id.clone(), Color32::WHITE),
        //     });

        match self.tab.line_mode() {
            ChatLayout::Traditional => {
                ui.horizontal(|ui| {
                    if let Some(ts) = ts {
                        ui.add(ts);
                    }
                    ui.add(sender);
                    ui.add(data);
                })
                .response
            }
            ChatLayout::Modern => {
                ui.vertical(|ui| {
                    ui.horizontal_top(|ui| {
                        ui.add(sender);
                        if let Some(ts) = ts {
                            ui.with_layout(Layout::right_to_left(egui::Align::Max), |ui| {
                                ui.add(ts)
                            });
                        }
                    });

                    ui.add(data);
                    ui.separator();
                })
                .response
            }
        }
    }
}
