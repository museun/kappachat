use crate::state::State;

use super::{SettingsView, StartView};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainViewState {
    #[default]
    Main,
    Settings,
}

pub struct MainView<'a> {
    state: &'a mut State,
}

impl<'a> MainView<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn display(self, ui: &mut egui::Ui) -> bool {
        match self.state.current_view {
            MainViewState::Main => {
                if StartView::new(&mut self.state.start_state).display(ui) {
                    if !self.state.twitch_settings.seems_good() {
                        self.state.previous_view = std::mem::replace(
                            &mut self.state.current_view, //
                            MainViewState::Settings,
                        );
                        SettingsView::activate(self.state, super::ActiveSettingsView::Twitch);
                        return false;
                    }
                }
            }

            MainViewState::Settings => {
                if SettingsView::new(self.state).display(ui) {
                    self.state.current_view = self.state.previous_view;
                    std::mem::take(&mut self.state.keybind_state);
                }
                return false;
            }
        }

        true
    }
}
