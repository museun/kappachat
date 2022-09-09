use egui::{Align, Key, Layout};

use crate::{state::SettingsState, tabs::Tabs, EnvConfig, KeyMapping};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum HelpView {
    KeyBindings,
    Settings,
    Twitch,
    Autojoin,
    #[default]
    None,
}

pub struct Help<'a> {
    showing_help: &'a mut HelpView,
    key_mapping: &'a mut KeyMapping,
    settings_state: &'a mut SettingsState,
    showing_tab_bar: &'a mut bool,
    tabs: &'a mut Tabs,
    config: &'a mut EnvConfig,
}

impl<'a> Help<'a> {
    pub fn new(
        showing_help: &'a mut HelpView,
        key_mapping: &'a mut KeyMapping,
        settings_state: &'a mut SettingsState,
        showing_tab_bar: &'a mut bool,
        tabs: &'a mut Tabs,
        config: &'a mut EnvConfig,
    ) -> Self {
        Self {
            showing_help,
            key_mapping,
            settings_state,
            showing_tab_bar,
            tabs,
            config,
        }
    }
}

impl<'a> egui::Widget for Help<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        use HelpView::*;

        let resp = ui.horizontal(|ui| {
            ui.selectable_value(self.showing_help, KeyBindings, "Key Bindings");
            ui.selectable_value(self.showing_help, Settings, "Settings");
            ui.selectable_value(self.showing_help, Twitch, "Twitch");
            ui.selectable_value(self.showing_help, Autojoin, "Autojoin");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.button("close").clicked()
            })
            .inner
        });

        ui.separator();

        match self.showing_help {
            KeyBindings => {
                super::KeyBindings::new(self.key_mapping).ui(ui);
            }
            Settings => {
                super::Settings::new(self.settings_state, self.showing_tab_bar, self.tabs).ui(ui);
            }
            Twitch => {
                super::TwitchSettings::new(self.config, self.settings_state).ui(ui);
            }
            Autojoin => {
                super::TwitchAutojoin::new(self.settings_state).ui(ui);
            }
            _ => {}
        }

        if resp.inner || ui.ctx().input().key_pressed(Key::Escape) {
            *self.showing_help = HelpView::None;
        }

        resp.response
    }
}
