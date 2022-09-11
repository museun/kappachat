use crate::state::State;

use super::{SettingsView, StartView};

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainView {
    #[default]
    Main,
    Settings,
}

// TODO rename this
pub struct MainWidget<'a> {
    state: &'a mut State,
}

impl<'a> MainWidget<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn display(self, ui: &mut egui::Ui) {
        match self.state.current_view {
            MainView::Main => {
                StartView::new(&mut self.state.start_state).display(ui);
            }

            MainView::Settings => {
                if SettingsView::new(self.state).display(ui) {
                    self.state.current_view = self.state.previous_view;
                    std::mem::take(&mut self.state.keybind_state);
                }
            }
        }
    }
}
