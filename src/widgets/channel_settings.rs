use egui::{Align, Layout};

use crate::tabs::Tabs;

pub struct ChannelSettings<'a> {
    pub tabs: &'a mut Tabs,
}

impl<'a> ChannelSettings<'a> {
    pub fn new(tabs: &'a mut Tabs) -> Self {
        Self { tabs }
    }
}

impl<'a> egui::Widget for ChannelSettings<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("channels");
                ui.separator();

                for tab in self.tabs.tabs_mut() {
                    ui.horizontal(|ui| {
                        ui.monospace(tab.title());

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.checkbox(&mut false, "save");
                            ui.checkbox(tab.showing_user_list_mut(), "user list");
                            ui.checkbox(tab.showing_timestamp_mut(), "timestamp");
                        });
                    });
                }
            })
        })
        .response
    }
}
