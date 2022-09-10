use egui::RichText;

use crate::tabs::Tabs;

pub type TabsWidget<'a> = &'a mut Tabs;

impl egui::Widget for TabsWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            let active = self.active;
            for (index, tab) in self
                .tabs
                .iter()
                .enumerate()
                .skip(1)
                .map(|(i, e)| (i - 1, e))
            {
                let text = RichText::new(&tab.title).color(self.tab_color(index));
                if ui.selectable_label(self.is_active(index), text).clicked() {
                    self.active = index;
                }

                // TODO close button
            }
        })
        .response
    }
}
