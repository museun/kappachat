use egui::{Align, ComboBox, Grid, Layout};

use crate::state::SettingsState;

pub struct DisplaySettings<'a> {
    settings_state: &'a mut SettingsState,
    showing_tab_bar: &'a mut bool, // TODO move this to another state struct
}

impl<'a> DisplaySettings<'a> {
    pub fn new(settings_state: &'a mut SettingsState, show_tab_bar: &'a mut bool) -> Self {
        Self {
            settings_state,
            showing_tab_bar: show_tab_bar,
        }
    }
}

impl<'a> egui::Widget for DisplaySettings<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("display");
                ui.separator();

                Grid::new("display_settings_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.monospace("pixels per point");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ComboBox::from_id_source("pixels_per_point")
                                .selected_text(SettingsState::dpi_repr(
                                    self.settings_state.pixels_per_point,
                                ))
                                .show_ui(ui, |ui| {
                                    for n in SettingsState::dpi_range() {
                                        ui.selectable_value(
                                            &mut self.settings_state.pixels_per_point,
                                            n,
                                            SettingsState::dpi_repr(n),
                                        );
                                    }
                                });
                        });

                        ui.end_row();

                        let ppp = self.settings_state.pixels_per_point;
                        if ui.ctx().pixels_per_point() != ppp {
                            ui.ctx().set_pixels_per_point(ppp);
                        }

                        ui.monospace("show tab bar");
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.checkbox(self.showing_tab_bar, "");
                        });
                        ui.end_row();
                    })
            });
        })
        .response
    }
}
