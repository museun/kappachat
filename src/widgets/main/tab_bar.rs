use egui::{Frame, Id, SidePanel, TopBottomPanel};

use super::Position;

pub struct TabBar {
    side: Position,
    width: f32,
}

impl TabBar {
    pub const fn new(side: Position, width: f32) -> Self {
        Self { side, width }
    }

    pub fn display(
        self,
        ctx: &egui::Context,
        hash_source: impl std::hash::Hash,
        body: impl FnOnce(&mut egui::Ui),
    ) -> (Id, egui::Rect) {
        let frame = Frame::none().fill(ctx.style().visuals.faint_bg_color);
        let range = self.width..=self.width;

        let id = Id::new(hash_source);

        // TODO enable scroll but disable scroll bars
        let resp = match (self.side.as_side(), self.side.as_top_bottom()) {
            (None, Some(top_bottom)) => {
                TopBottomPanel::new(top_bottom, id.with("tab_bar"))
                    .resizable(false)
                    .frame(frame)
                    .height_range(range)
                    // TODO this style exists: pub scroll_bar_width: f32,
                    .show(ctx, |ui| {
                        ui.horizontal(body);
                    })
                    .response
            }
            (Some(side), None) => {
                SidePanel::new(side, id.with("tab_bar"))
                    .resizable(false)
                    .frame(frame)
                    .width_range(range)
                    // TODO this style exists: pub scroll_bar_width: f32,
                    .show(ctx, |ui| {
                        ui.vertical(body);
                    })
                    .response
            }
            _ => unreachable!(),
        };

        (id, resp.rect)
    }
}
