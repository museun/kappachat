use eframe::{
    egui::{CentralPanel, Frame, Response, SidePanel, TopBottomPanel},
    epaint::Shadow,
};
use egui::{vec2, Align, ComboBox, Layout, Rect, RichText, Rounding, Slider, Stroke};
use egui_extras::RetainedImage;

use crate::{state::State, widgets::main::Position};

pub struct DisplaySettings<'a> {
    state: &'a mut State,
    dark_mask: &'a RetainedImage,
}

impl<'a> DisplaySettings<'a> {
    pub fn new(state: &'a mut State, dark_mask: &'a RetainedImage) -> Self {
        Self { state, dark_mask }
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

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.state.chat_view_state.show_mask,
                "Use an circle image mask",
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add(
                    Slider::new(&mut self.state.chat_view_state.image_size, 16.0..=64.0)
                        .step_by(4.0)
                        .fixed_decimals(1)
                        .text("Tab image size"),
                );
            })
        });

        ui.separator();

        let size = self.state.chat_view_state.image_size;

        let resp = Frame::none().shadow(Shadow::small_dark()).show(ui, |ui| {
            macro_rules! show_tab_bar {
                ($ui:expr) => {
                    let size = vec2(size, size);

                    for kappa in self.state.start_state.kappas.iter().take(4) {
                        let cursor = $ui.cursor();
                        let rect = $ui.painter().rect(
                            Rect::from_min_size(cursor.left_top(), size),
                            Rounding::none(),
                            $ui.style().visuals.extreme_bg_color,
                            Stroke::none(),
                        );

                        let resp = $ui.put(
                            Rect::from_min_size(cursor.left_top(), size),
                            |ui: &mut egui::Ui| kappa.show_max_size(ui, size),
                        );

                        if self.state.chat_view_state.show_mask {
                            $ui.put(resp.rect, |ui: &mut egui::Ui| {
                                self.dark_mask.show_max_size(ui, size)
                            });
                        }
                    }
                };
            }

            let range = size..=size;
            match self.state.chat_view_state.tab_bar_position {
                Position::Top => {
                    TopBottomPanel::top("kappa_demo_top")
                        .resizable(false)
                        .frame(Frame::none())
                        .height_range(range)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                show_tab_bar!(ui);
                            });
                        });
                }
                Position::Left => {
                    SidePanel::left("kappa_demo_left")
                        .resizable(false)
                        .frame(Frame::none())
                        .width_range(range)
                        .show_inside(ui, |ui| {
                            ui.vertical(|ui| {
                                show_tab_bar!(ui);
                            });
                        });
                }
                Position::Right => {
                    SidePanel::right("kappa_demo_right")
                        .resizable(false)
                        .frame(Frame::none())
                        .width_range(range)
                        .show_inside(ui, |ui| {
                            ui.vertical(|ui| {
                                show_tab_bar!(ui);
                            });
                        });
                }
                Position::Bottom => {
                    TopBottomPanel::bottom("kappa_demo_bottom")
                        .resizable(false)
                        .frame(Frame::none())
                        .height_range(range)
                        .show_inside(ui, |ui| {
                            ui.horizontal(|ui| {
                                show_tab_bar!(ui);
                            });
                        });
                }
            }

            CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        nothing(ui);
                        ui.selectable_value(
                            &mut self.state.chat_view_state.tab_bar_position,
                            Position::Top,
                            "⬆",
                        );
                        nothing(ui);
                    });
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.state.chat_view_state.tab_bar_position,
                            Position::Left,
                            "⬅",
                        );
                        nothing(ui);
                        ui.selectable_value(
                            &mut self.state.chat_view_state.tab_bar_position,
                            Position::Right,
                            "➡",
                        )
                    });
                    ui.horizontal(|ui| {
                        nothing(ui);
                        ui.selectable_value(
                            &mut self.state.chat_view_state.tab_bar_position,
                            Position::Bottom,
                            "⬇",
                        );
                        nothing(ui)
                    });
                });
            });
        });

        fn nothing(ui: &mut egui::Ui) -> Response {
            ui.add_visible_ui(false, |ui: &mut egui::Ui| {
                ui.selectable_value(&mut false, false, "⬇")
            })
            .response
        }

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
