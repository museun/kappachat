use egui::{vec2, Label, RichText, ScrollArea, Sense, TextStyle};
use egui_extras::RetainedImage;

use crate::{
    helix::{Chatters, Kind},
    ImageCache,
};

pub struct UserList<'a> {
    chatters: &'a Chatters,
    images: &'a ImageCache,
}

impl<'a> UserList<'a> {
    pub const fn new(chatters: &'a Chatters, images: &'a ImageCache) -> Self {
        Self { chatters, images }
    }

    fn get_image(&self, kind: Kind) -> Option<&RetainedImage> {
        None
        // self.images.get(&kind.as_str()[..kind.as_str().len() - 1])
    }

    pub fn display(self, ui: &mut egui::Ui) {
        let width = ui
            .fonts()
            .glyph_width(&TextStyle::Body.resolve(ui.style()), ' ');

        ui.scope(|ui| {
            ui.spacing_mut().item_spacing.x = width;
            ui.style_mut().spacing.interact_size.y = width;
            let image_size = vec2(8.0, 8.0);

            ScrollArea::vertical().show(ui, |ui| {
                for (kind, chatter) in &self.chatters.chatters {
                    ui.horizontal(|ui| {
                        if let Some(img) = self.get_image(*kind) {
                            img.show_size(ui, image_size);
                        } else {
                            ui.allocate_exact_size(image_size, Sense::hover());
                        }
                        ui.add(Label::new(RichText::new(chatter).small()).wrap(false));
                    });
                }
            });
        });
    }
}
