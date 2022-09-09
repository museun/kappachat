use egui::{Align, Grid, Key, Layout, TextEdit};

use crate::state::SettingsState;

pub struct TwitchAutojoin<'a> {
    settings_state: &'a mut SettingsState,
}

impl<'a> TwitchAutojoin<'a> {
    pub fn new(settings_state: &'a mut SettingsState) -> Self {
        Self { settings_state }
    }
}

impl<'a> egui::Widget for TwitchAutojoin<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui
            .vertical(|ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Channels");

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if self.settings_state.adding_channel.is_some() {
                                if ui.small_button("❌").clicked() {
                                    self.settings_state.adding_channel.take();
                                }
                            }

                            if let Some(str) = &mut self.settings_state.adding_channel {
                                let resp = ui.add(TextEdit::singleline(str).lock_focus(true));
                                self.settings_state.adding_channel_id.replace(resp.id);

                                if resp.lost_focus() && ui.ctx().input().key_pressed(Key::Enter) {
                                    if let Some(channel) =
                                        std::mem::take(&mut self.settings_state.adding_channel)
                                    {
                                        let channel = channel.trim();

                                        // TODO report errors so we can bail
                                        if channel.contains(' ') {
                                            eprintln!("'{channel}' cannot contain spaces");
                                        } else {
                                            let channel = if channel.starts_with('#') {
                                                channel.to_string()
                                            } else {
                                                format!("#{channel}")
                                            };

                                            if self
                                                .settings_state
                                                .channels
                                                .iter()
                                                .any(|c| c == &channel)
                                            {
                                                eprintln!("duplicate: {channel}")
                                            } else {
                                                self.settings_state.channels.push(channel)
                                            }
                                        }
                                        self.settings_state.adding_channel_id.take();
                                    }
                                }
                            }

                            if self.settings_state.adding_channel.is_none() {
                                if ui.small_button("➕").clicked() {
                                    self.settings_state.adding_channel.replace(String::new());
                                    if let Some(id) = self.settings_state.adding_channel_id {
                                        ui.ctx().memory().request_focus(id);
                                    }
                                }
                            }
                        });
                    });

                    ui.separator();
                    Grid::new("twitch_channels")
                        .num_columns(2)
                        .striped(true)
                        .show(ui, |ui| {
                            for channel in &self.settings_state.channels {
                                ui.monospace(channel);

                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("❌").clicked() {
                                        self.settings_state
                                            .channels_to_remove
                                            .push(channel.clone());
                                    }
                                });
                                ui.end_row()
                            }
                        })
                });
            })
            .response;

        for channel in self.settings_state.channels_to_remove.drain(..) {
            if let Some(pos) = self
                .settings_state
                .channels
                .iter()
                .position(|c| c == &channel)
            {
                self.settings_state.channels.remove(pos);
            }
        }

        resp
    }
}
