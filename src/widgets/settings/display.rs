use egui::{ComboBox, RichText};

use crate::state::State;

pub struct DisplaySettings<'a> {
    state: &'a mut State,
}

impl<'a> DisplaySettings<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn display(self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.monospace("Pixels per point");

            ComboBox::from_id_source("pixels_per_point")
                .width(50.0)
                .selected_text(Self::dpi_repr(self.state.pixels_per_point))
                .show_ui(ui, |ui| {
                    for n in Self::dpi_range() {
                        ui.selectable_value(
                            &mut self.state.pixels_per_point,
                            n,
                            RichText::new(Self::dpi_repr(n)),
                        );
                    }
                });
        });

        egui::widgets::global_dark_light_mode_buttons(ui);

        let ppp = self.state.pixels_per_point;
        if ui.ctx().pixels_per_point() != ppp {
            ui.ctx().set_pixels_per_point(ppp);
        }
    }

    // TODO test this on a 4k monitor, we might need to go up to 4.0
    fn dpi_repr(f: f32) -> &'static str {
        const LOOKUP: [&str; 11] = [
            "1.0", "1.1", "1.2", "1.3", "1.4", "1.5", //
            "1.6", "1.7", "1.8", "1.9", "2.0",
        ];
        let index = ((f * 10.0) as usize) - 10;
        LOOKUP[index]
    }

    fn dpi_range() -> impl Iterator<Item = f32> {
        std::iter::successors(Some(1.0_f32), |a| Some(a + 0.1)).take(11)
    }
}
