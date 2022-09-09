use egui::{Grid, Key, TextEdit};

use crate::{state::SettingsState, EnvConfig};

pub struct TwitchSettings<'a> {
    config: &'a mut EnvConfig,
    settings_state: &'a mut SettingsState,
}

impl<'a> TwitchSettings<'a> {
    pub fn new(config: &'a mut EnvConfig, settings_state: &'a mut SettingsState) -> Self {
        Self {
            config,
            settings_state,
        }
    }
}

impl<'a> egui::Widget for TwitchSettings<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        Grid::new("twitch_settings")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                for (left, right, password) in [
                    ("Name", &mut self.config.twitch_name, false),
                    ("OAuth Token", &mut self.config.twitch_oauth_token, true),
                    ("Client-Id", &mut self.config.twitch_client_id, false),
                    ("Client-Secret", &mut self.config.twitch_client_secret, true),
                ] {
                    ui.monospace(left);
                    ui.horizontal(|ui| {
                        // TODO make this into a custom widget, embed the button
                        if ui
                            .add(TextEdit::singleline(right).password({
                                self.settings_state
                                    .twitch_visible
                                    .get(&SettingsState::make_hash(left))
                                    .copied()
                                    .unwrap_or(password)
                            }))
                            .lost_focus()
                            && ui.ctx().input().key_pressed(Key::Enter)
                        {
                            // XXX: what to do here?
                        }
                        if password {
                            let down = ui.small_button("🔎").is_pointer_button_down_on();
                            *self
                                .settings_state
                                .twitch_visible
                                .entry(SettingsState::make_hash(left))
                                .or_insert(true) = !down;
                        }
                    });

                    ui.end_row()
                }
            })
            .response
    }
}
