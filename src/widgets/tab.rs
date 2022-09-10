use crate::{helix::CachedImages, tabs::Tab};

pub struct TabWidget<'a> {
    pub tab: &'a mut Tab,
    pub cached_images: &'a mut CachedImages,
    pub stick: bool,
}

impl<'a> egui::Widget for TabWidget<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // ui.horizontal(|ui| {
        let resp = egui::containers::ScrollArea::vertical()
            .id_source(&self.tab.title)
            .hscroll(false)
            .stick_to_bottom(self.stick)
            .auto_shrink([false, false])
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                // ui.vertical(|ui| {
                for line in self.tab.entries() {
                    ui.add(self.tab.as_widget(line));
                }
                // });
            });

        if self.tab.show_user_list {
            egui::panel::SidePanel::right(&self.tab.title)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.add(self.tab.as_chatters(self.cached_images));
                });
        }
        // })
        egui::Frame::none().show(ui, |ui| {}).response
    }
}
