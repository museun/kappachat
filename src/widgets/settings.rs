use crate::{state::SettingsState, tabs::Tabs};

use super::{ChannelSettings, DisplaySettings};

pub struct Settings<'a> {
    settings_state: &'a mut SettingsState,
    showing_tab_bar: &'a mut bool,
    tabs: &'a mut Tabs,
}

impl<'a> Settings<'a> {
    pub fn new(
        settings_state: &'a mut SettingsState,
        showing_tab_bar: &'a mut bool,
        tabs: &'a mut Tabs,
    ) -> Self {
        Self {
            settings_state,
            showing_tab_bar,
            tabs,
        }
    }
}

impl<'a> egui::Widget for Settings<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            DisplaySettings::new(self.settings_state, self.showing_tab_bar).ui(ui);
            ChannelSettings::new(self.tabs).ui(ui);
        })
        .response
    }
}
