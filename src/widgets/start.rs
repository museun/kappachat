use std::time::{Duration, Instant};

use eframe::egui::Button;
use egui::{
    style::Margin, vec2, Align2, Area, Direction, Frame, Layout, Pos2, Rect, RichText, Sense, Vec2,
};

use crate::{state::ViewState, Chord, KeyAction, KeyMapping, RequestPaint};

use super::MainViewView;

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

struct Inlay<'a> {
    key_mapping: &'a mut KeyMapping,
    view: &'a mut ViewState,
}

impl<'a> Inlay<'a> {
    fn new(key_mapping: &'a mut KeyMapping, view: &'a mut ViewState) -> Self {
        Self { key_mapping, view }
    }

    fn display(&mut self, ui: &mut egui::Ui) {
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
        view: MainViewView,
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
        use MainViewView::*;

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
