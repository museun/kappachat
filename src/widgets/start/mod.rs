use std::time::{Duration, Instant};

use egui::{vec2, Align2, Area, Direction, Layout, Pos2, Rect, Sense, Vec2};

use crate::{state::ViewState, KeyMapping};

use super::MainViewView;

mod rotation;
use rotation::StartRotation;

mod inlay;
use inlay::Inlay;

mod state;
pub use state::StartState;

pub struct StartView<'a> {
    state: &'a mut StartState,
    key_mapping: &'a mut KeyMapping,
    view: &'a mut ViewState,
}

impl<'a> StartView<'a> {
    pub fn new(
        state: &'a mut StartState,
        key_mapping: &'a mut KeyMapping,
        view: &'a mut ViewState,
    ) -> Self {
        Self {
            state,
            key_mapping,
            view,
        }
    }

    const DELAY: Duration = Duration::from_secs(5);

    fn pick_random(&mut self) {
        if self.state.last.elapsed() > Self::DELAY {
            self.state.last = Instant::now();
            self.state.kappa_index = fastrand::usize(0..self.state.kappas.len());
        }
    }

    fn force_random(&mut self) {
        self.state.last = Instant::now();
        self.state.kappa_index = fastrand::usize(0..self.state.kappas.len());
    }
}

impl<'a> StartView<'a> {
    pub fn display(mut self, ui: &mut egui::Ui) -> bool {
        Inlay::new(&mut self.key_mapping, &mut self.view).display(ui);

        self.pick_random();
        ui.ctx().request_repaint_after(Self::DELAY);
        let ppp = ui.ctx().pixels_per_point();

        let img_id = self.state.kappas[self.state.kappa_index].texture_id(ui.ctx());
        let img_size = self.state.kappas[self.state.kappa_index].size_vec2();

        let rot = (self.state.start_rotation.speed * ppp).to_radians();

        if self.state.start_rotation.spinning {
            self.kappas_rotation(ui);
        }

        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            let y = ui.available_height() / 2.0;
            let x = ui.available_width() / 2.0;

            let resp = ui.add(
                egui::Image::new(img_id, img_size)
                    .rotate(self.state.start_rotation.rotation, Vec2::splat(0.6)),
            );

            let resp = ui
                .interact(
                    Rect::from_center_size(Pos2 { x, y }, img_size),
                    resp.id,
                    Sense::click().union(Sense::hover()),
                )
                .on_hover_ui_at_pointer(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Click to");
                        ui.add(egui::Image::new(img_id, img_size / vec2(6.0, 6.0)));
                    });
                });

            if resp.secondary_clicked() {
                self.force_random();
            }

            if resp.hovered() && ui.ctx().input().modifiers.shift_only() {
                if !self.state.start_rotation.hovered {
                    self.state.time = ui.input().time;
                    self.state.rot = self.state.start_rotation.rotation;
                    self.state.start_rotation.spinning = true;
                }

                self.state.start_rotation.hovered = true;
                self.state.start_rotation.rotate_cw(rot, ui.ctx());
            } else {
                self.state.start_rotation.hovered = false;
            }

            if !self.state.start_rotation.hovered {
                self.state.start_rotation.rotate_ccw(rot, ui.ctx());
            }

            resp.clicked()
        })
        .inner
    }

    fn kappas_rotation(&mut self, ui: &mut egui::Ui) {
        Area::new("kps")
            .anchor(Align2::LEFT_TOP, vec2(20.0, 20.0))
            .movable(false)
            .show(ui.ctx(), |ui| {
                ui.small(format!("Kappas per second: {:.3?}", {
                    let dt = ui.input().time - self.state.time;
                    if dt.trunc() % 1.0 != 1.0 {
                        self.state.rot = self.state.start_rotation.rotation.to_radians() * 6.0;
                    }
                    self.state.rot
                }));
            });
    }
}
