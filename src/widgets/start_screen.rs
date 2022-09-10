use std::time::{Duration, Instant};

use egui::{vec2, Direction, Layout, Pos2, Rect, Sense, Vec2};
use egui_extras::RetainedImage;

use crate::{RequestPaint, State};

pub struct StartRotation {
    rotation: f32,
    hovered: bool,
    speed: f32,
}

impl StartRotation {
    pub const fn new() -> Self {
        Self {
            rotation: 0.0,
            speed: 0.5,
            hovered: false,
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

pub struct StartScreen<'a> {
    images: &'a [RetainedImage],
    index: &'a mut usize,
    last: &'a mut Instant,
    // command: &'a mut Option<Command<'static>>,
    start_rotation: &'a mut StartRotation,
}

impl<'a> StartScreen<'a> {
    pub fn new(
        state: &'a mut State,
        // images: &'a [RetainedImage],
        // index: &'a mut usize,
        // last: &'a mut Instant,
        // command: &'a mut Option<Command<'static>>,
        // start_rotation: &'a mut StartRotation,
    ) -> Self {
        let State {
            kappas: images,
            kappa_index: index,
            last,
            start_rotation,
            ..
        } = state;

        Self {
            images,
            index,
            last,
            start_rotation,
        }
    }

    const DELAY: Duration = Duration::from_secs(5);

    fn pick_random(&mut self) {
        if self.last.elapsed() > Self::DELAY {
            *self.last = Instant::now();
            *self.index = fastrand::usize(0..self.images.len());
        }
    }
}

impl<'a> egui::Widget for StartScreen<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        self.pick_random();

        ui.ctx().request_repaint_after(Self::DELAY);

        let ppp = ui.ctx().pixels_per_point();

        let img_id = self.images[*self.index].texture_id(ui.ctx());
        let img_size = self.images[*self.index].size_vec2();

        let rot = (self.start_rotation.speed * ppp).to_radians();

        ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
            let y = ui.available_height() / 2.0;
            let x = ui.available_width() / 2.0;

            let resp = ui.add(
                egui::Image::new(img_id, img_size)
                    .rotate(self.start_rotation.rotation, Vec2::splat(0.6)),
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
                self.start_rotation.hovered = true;
                self.start_rotation.rotate_cw(rot, ui.ctx());
            } else {
                self.start_rotation.hovered = false;
            }

            if !self.start_rotation.hovered {
                self.start_rotation.rotate_ccw(rot, ui.ctx());
            }

            if resp.clicked() {
                // TODO use a channel for this
                eprintln!("Kappa")
                // self.command.replace(Command::Connect);
            }
            resp
        })
        .response
    }
}
