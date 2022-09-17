use std::collections::HashSet;

use egui::CentralPanel;

use crate::state::AppState;

use super::{ActiveSettingsView, SettingsView, StartView};

mod chat_line;
use chat_line::{ChatLine, ChatLineView};

mod timestamp;
use timestamp::Timestamp;

mod state;
pub use state::ChatViewState;

mod position;
pub use position::Position;

mod tab_view;
use tab_view::TabView;

mod tab_bar;
use tab_bar::TabBar;

mod chat_view;
use chat_view::ChatView;

mod user_list;
use user_list::UserList;

mod edit_box;
use edit_box::EditBox;

#[derive(Copy, Clone, Default, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum MainView {
    #[default]
    Start,
    Main,
    Settings,
}

// TODO get rid of this
#[derive(Default)]
pub struct MainViewState {
    channels_to_join: Vec<String>,
    channels_not_joined: HashSet<String>,
}

pub struct Main<'a> {
    state: &'a mut AppState,
}

impl<'a> Main<'a> {
    pub fn new(state: &'a mut AppState) -> Self {
        Self { state }
    }

    pub fn display(mut self, ctx: &egui::Context) {
        match self.state.state.view_state.current_view {
            MainView::Start => {
                self.display_main(ctx)

                // CentralPanel::default().show(ctx, |ui| {
                //     self.display_start(ui);
                // });
            }
            MainView::Settings => {
                CentralPanel::default().show(ctx, |ui| {
                    self.display_settings(ui);
                });
            }
            MainView::Main => {
                // CentralPanel::default().show(ctx, |ui| {
                //     self.display_start(ui);
                // });
                self.display_main(ctx)
            }
        }
    }

    fn display_main(&mut self, ctx: &egui::Context) {
        let writer = self.state.writer.clone();
        ChatView::new(self.state, writer).display(ctx);
    }

    fn display_settings(&mut self, ui: &mut egui::Ui) {
        if SettingsView::new(self.state).display(ui) {
            self.state.state.view_state.current_view = self.state.state.view_state.previous_view;
            std::mem::take(&mut self.state.state.keybind_state);
        }
    }

    fn display_start(&mut self, ui: &mut egui::Ui) {
        if StartView::new(
            &mut self.state.state.start_state,
            &mut self.state.state.key_mapping,
            &mut self.state.state.view_state,
        )
        .display(ui)
        {
            if self.state.state.twitch_settings.seems_good() {
                // BUG this blocks, so it should be using a promise
                if let Err(err) = self.state.connect(ui.ctx().clone()) {
                    eprintln!("cannot connect: {err}");
                    todo!("recover from this")
                }

                self.state.state.view_state.current_view = MainView::Main;
                return;
            }
            self.state
                .state
                .view_state
                .switch_to_view(MainView::Settings);
            SettingsView::activate(&mut self.state.state, ActiveSettingsView::Twitch);
        }
    }
}
