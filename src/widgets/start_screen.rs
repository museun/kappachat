use std::time::Instant;

use egui::{vec2, Direction, Layout, Pos2, Rect, Sense};
use egui_extras::RetainedImage;

pub struct StartScreen<'a> {
    pub images: &'a [RetainedImage],
    pub index: &'a mut usize,
    pub last: &'a mut Instant,
}

impl StartScreen<'_> {
    fn pick_random(&mut self) {
        if self.last.elapsed() > std::time::Duration::from_secs(1) {
            *self.last = Instant::now();
            *self.index = fastrand::usize(0..self.images.len());
        }
    }
}

impl<'a> egui::Widget for StartScreen<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        self.pick_random();

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));

        ui.scope(|ui| {
            ui.style_mut().spacing.button_padding = vec2(100.0, 100.0);
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                let y = ui.available_height() / 2.0;
                let x = ui.available_width() / 2.0;

                let resp = ui.add(
                    egui::ImageButton::new(
                        self.images[*self.index].texture_id(ui.ctx()),
                        self.images[*self.index].size_vec2(),
                    )
                    .frame(false),
                );

                let resp = ui
                    .interact(
                        Rect::from_center_size(Pos2 { x, y }, self.images[*self.index].size_vec2()),
                        resp.id,
                        Sense::click(),
                    )
                    .on_hover_ui_at_pointer(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Click to");
                            ui.add(
                                egui::ImageButton::new(
                                    self.images[*self.index].texture_id(ui.ctx()),
                                    self.images[*self.index].size_vec2() / vec2(6.0, 6.0),
                                )
                                .frame(false),
                            );
                        });
                    });

                if resp.clicked() {
                    eprintln!("connect")
                }
                return resp;
            })
            .inner
        })
        .inner
    }
}
