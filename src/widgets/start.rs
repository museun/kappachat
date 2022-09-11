use std::time::{Duration, Instant};

use egui::{vec2, Direction, Layout, Pos2, Rect, Sense, Vec2};

use crate::RequestPaint;

pub struct StartRotation {
    rotation: f32,
    hovered: bool,
    speed: f32,
    spinning: bool,
}

impl Default for StartRotation {
    fn default() -> Self {
        Self::new()
    }
}

impl StartRotation {
    const fn new() -> Self {
        Self {
            rotation: 0.0,
            speed: 0.5,
            hovered: false,
            spinning: false,
        }
    }

    const SPEED_MAX: f32 = 0.3;
    const SPEED_DELTA: f32 = 0.005;

    fn rotate_cw(&mut self, rot: f32, repaint: &impl RequestPaint) {
        self.speed = (self.speed + Self::SPEED_DELTA).max(Self::SPEED_MAX);
        self.rotation += rot;
        repaint.request_repaint();
    }

    fn rotate_ccw(&mut self, rot: f32, repaint: &impl RequestPaint) {
        if self.rotation % std::f32::consts::TAU > 0.0 {
            self.speed = (self.speed - Self::SPEED_DELTA).max(Self::SPEED_MAX);
            self.rotation = (self.rotation - rot).max(0.0);
            repaint.request_repaint();
            return;
        }

        // reset it
        let _ = std::mem::replace(self, Self::new());
        repaint.request_repaint();
    }
}

pub struct StartState {
    pub kappas: Vec<egui_extras::RetainedImage>,
    pub last: std::time::Instant,
    pub kappa_index: usize,
    pub start_rotation: StartRotation,
    pub time: f64,
    pub rot: f32,
}

impl Default for StartState {
    fn default() -> Self {
        Self {
            kappas: Default::default(),
            last: std::time::Instant::now(),
            kappa_index: Default::default(),
            start_rotation: Default::default(),
            time: 0.0,
            rot: 0.0,
        }
    }
}

pub struct StartView<'a> {
    state: &'a mut StartState,
}

impl<'a> StartView<'a> {
    pub fn new(state: &'a mut StartState) -> Self {
        Self { state }
    }

    const DELAY: Duration = Duration::from_secs(5);

    fn pick_random(&mut self) {
        if self.state.last.elapsed() > Self::DELAY {
            self.state.last = Instant::now();
            self.state.kappa_index = fastrand::usize(0..self.state.kappas.len());
        }
    }
}

impl<'a> StartView<'a> {
    pub fn display(mut self, ui: &mut egui::Ui) -> bool {
        self.pick_random();

        ui.ctx().request_repaint_after(Self::DELAY);

        let ppp = ui.ctx().pixels_per_point();

        let img_id = self.state.kappas[self.state.kappa_index].texture_id(ui.ctx());
        let img_size = self.state.kappas[self.state.kappa_index].size_vec2();

        let rot = (self.state.start_rotation.speed * ppp).to_radians();

        if self.state.start_rotation.spinning {
            ui.small(format!("Kappas per second: {:.3?}", {
                let dt = ui.input().time - self.state.time;
                if dt.trunc() % 1.0 != 1.0 {
                    self.state.rot = self.state.start_rotation.rotation.to_radians() * 6.0;
                }
                self.state.rot
            }));
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
}
