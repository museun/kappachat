use egui::ScrollArea;

use crate::state::State;

mod keybind;
pub use keybind::{KeybindSettings, KeybindingsState};

mod display;
pub use display::DisplaySettings;

mod channel;
pub use channel::{ChannelSettings, TwitchChannelsState};

mod twitch;
pub use twitch::{TwitchSettings, TwitchSettingsState};

#[derive(Default, PartialEq, PartialOrd, Eq, Ord)]
pub enum ActiveSettingsView {
    #[default]
    Channels,
    KeyBindings,
    Twitch,
    Display,
    None,
}

#[derive(Default)]
pub struct SettingsState {
    active: ActiveSettingsView,
}

pub struct SettingsView<'a> {
    state: &'a mut State,
}

impl<'a> SettingsView<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn activate(state: &'a mut State, view: ActiveSettingsView) {
        state.settings.active = view;
    }

    pub fn display(self, ui: &mut egui::Ui) -> bool {
        use ActiveSettingsView::*;

        let resp = ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.settings.active, Channels, "Channels");
            ui.selectable_value(&mut self.state.settings.active, KeyBindings, "Key Bindings");
            ui.selectable_value(&mut self.state.settings.active, Twitch, "Twitch");
            ui.selectable_value(&mut self.state.settings.active, Display, "Display");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.button("close").clicked()
            })
            .inner
        });

        ui.separator();

        ScrollArea::horizontal()
            .auto_shrink([false, false])
            .max_width(ui.available_width())
            .show(ui, |ui| match self.state.settings.active {
                Channels => self.display_channels(ui),
                KeyBindings => self.display_keybindings(ui),
                Twitch => self.display_twitch(ui),
                Display => self.display_display(ui),
                _ => {}
            });

        resp.inner
    }

    fn display_channels(self, ui: &mut egui::Ui) {
        ChannelSettings::new(&mut self.state.twitch_channels, &mut self.state.channels).display(ui);
    }

    fn display_keybindings(self, ui: &mut egui::Ui) {
        KeybindSettings::new(&mut self.state.keybind_state, &mut self.state.key_mapping).display(ui)
    }

    fn display_twitch(self, ui: &mut egui::Ui) {
        TwitchSettings::new(&mut self.state.config, &mut self.state.twitch_settings).display(ui)
    }

    fn display_display(self, ui: &mut egui::Ui) {
        DisplaySettings::new(self.state).display(ui)
    }
}