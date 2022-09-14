use eframe::egui::Button;
use egui::{style::Margin, vec2, Align2, Area, Frame, RichText, Sense};

use crate::{state::ViewState, Chord, KeyAction, KeyMapping};

use super::MainView;

pub struct Inlay<'a> {
    key_mapping: &'a mut KeyMapping,
    view: &'a mut ViewState,
}

impl<'a> Inlay<'a> {
    pub fn new(key_mapping: &'a mut KeyMapping, view: &'a mut ViewState) -> Self {
        Self { key_mapping, view }
    }

    pub fn display(&mut self, ui: &mut egui::Ui) {
        Area::new("inlay")
            .anchor(Align2::RIGHT_TOP, vec2(0.0, 0.0))
            .movable(false)
            .show(ui.ctx(), |ui| {
                Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            self.display_inlay_list(ui);
                        });
                    });
            });
    }

    fn display_inlay_text(
        ui: &mut egui::Ui,
        repr: &str,
        view: MainView,
        chord: Chord,
        switch: &mut ViewState,
    ) {
        let text = RichText::new(format!("Press {} for {repr}", chord.display()));

        let button = Button::new(text)
            .small()
            .frame(false)
            .sense(Sense::click())
            .wrap(false);

        if ui.add(button).clicked() {
            switch.switch_to_view(view);
        }
    }

    fn display_inlay_list(&mut self, ui: &mut egui::Ui) {
        use KeyAction::*;
        use MainView::*;

        for ((repr, view), chord) in [
            ("Kappas", Start, SwitchToMain),
            ("Settings", Settings, SwitchToSettings),
        ]
        .into_iter()
        .flat_map(|(repr, view, chord)| {
            Some((
                (repr, view),
                self.key_mapping.find_chords_reverse(&chord)?.first()?,
            ))
        }) {
            Self::display_inlay_text(ui, repr, view, *chord, self.view);
        }
    }
}
