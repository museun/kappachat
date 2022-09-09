use egui::{vec2, Color32, Frame, Label, RichText, ScrollArea, Sense};

use crate::helix::{CachedImages, Chatters, Kind};

pub struct ChatterList<'a> {
    pub chatters: &'a Chatters,
    pub cached_images: &'a CachedImages,
}

impl<'a> egui::Widget for ChatterList<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Frame::none()
            .show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for (kind, chatters) in &self.chatters.chatters {
                        let id = ui.make_persistent_id(kind.as_str());

                        let mut state =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                id,
                                !matches!(kind, Kind::Viewer),
                            );

                        let header = ui
                            .scope(|ui| {
                                ui.spacing_mut().item_spacing.x = 2.0;

                                ui.horizontal(|ui| {
                                    if let Some(id) = self.cached_images.id_map.get(kind) {
                                        if let Some(img) = self.cached_images.map.get(id) {
                                            img.show_size(ui, vec2(8.0, 8.0));
                                        }
                                    }

                                    ui.add(
                                        Label::new(
                                            RichText::new(kind.as_str())
                                                .color(Color32::WHITE)
                                                .small(),
                                        )
                                        .wrap(false)
                                        .sense(Sense::click()),
                                    )
                                })
                                .inner
                            })
                            .inner;

                        if header.clicked() {
                            state.toggle(ui);
                        }

                        state.show_body_unindented(ui, |ui| {
                            for chatter in chatters {
                                ui.small(chatter);
                            }
                        });
                    }
                })
            })
            .response
    }
}
