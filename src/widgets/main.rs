use std::collections::HashSet;

use crate::state::AppState;

use super::{SettingsView, StartView};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainViewView {
    #[default]
    Start,
    Main,
    Settings,
    Connecting,
    Connected,
}

#[derive(Default)]
pub struct MainViewState {
    channels_to_join: Vec<String>,
    channels_not_joined: HashSet<String>,
}

pub struct MainView<'a> {
    app: &'a mut AppState,
}

impl<'a> MainView<'a> {
    pub fn new(app: &'a mut AppState) -> Self {
        Self { app }
    }

    pub fn display(self, ui: &mut egui::Ui) {
        match self.app.state.current_view {
            MainViewView::Start => {
                if StartView::new(&mut self.app.state.start_state).display(ui) {
                    if !self.app.state.twitch_settings.seems_good() {
                        self.app.state.previous_view = std::mem::replace(
                            &mut self.app.state.current_view, //
                            MainViewView::Settings,
                        );
                        SettingsView::activate(
                            &mut self.app.state,
                            super::ActiveSettingsView::Twitch,
                        );
                        return;
                    }

                    self.app.state.current_view = MainViewView::Connecting;
                }
            }

            MainViewView::Settings => {
                if SettingsView::new(&mut self.app.state).display(ui) {
                    self.app.state.current_view = self.app.state.previous_view;
                    std::mem::take(&mut self.app.state.keybind_state);
                }
            }

            MainViewView::Connecting => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("connecting to Twitch");
                        ui.spinner();
                    });

                    ui.label(format!("our name: {}", self.app.state.config.twitch_name));
                });

                if let Err(err) = self.app.connect(ui.ctx().clone()) {
                    eprintln!("error: {err}");
                    self.app.state.current_view = MainViewView::Start;
                }

                self.app.state.main_view.channels_to_join.extend(
                    self.app
                        .state
                        .channels
                        .iter()
                        .filter_map(|c| c.auto_join.then(|| c.name.clone())),
                );

                for channel in &self.app.state.main_view.channels_to_join {
                    self.app.join_channel(channel);
                }

                self.app.state.current_view = MainViewView::Connected;
            }

            MainViewView::Connected => {
                ui.vertical(|ui| {
                    ui.label(format!(
                        "connected as: {} ({})",
                        self.app.identity().user_name,
                        self.app.identity().user_id
                    ));

                    ui.label("joining channels:");
                    for channel in &self.app.state.main_view.channels_to_join {
                        ui.label(channel);
                    }

                    for join in self.app.state.messages.iter().filter_map(|c| c.as_join()) {
                        if self.app.is_our_name(join.user) {
                            if self
                                .app
                                .state
                                .main_view
                                .channels_not_joined
                                .remove(join.channel)
                            {
                                ui.label(format!("joined: {}", join.channel));
                            }
                        }
                    }
                });

                if self.app.state.main_view.channels_not_joined.is_empty() {
                    self.app.state.current_view = MainViewView::Main;
                }
            }

            MainViewView::Main => {
                ui.vertical(|ui| {
                    for message in self.app.state.messages.iter() {
                        if let Some(pm) = message.as_privmsg() {
                            ui.label(format!("[{}] {}: {}", pm.target, pm.sender, pm.data));
                        }
                    }
                });
            }
        }
    }
}
