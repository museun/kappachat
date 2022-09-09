use egui::{text::LayoutJob, Color32, FontId, TextFormat};

pub trait JobExt: Sized {
    fn just_text(self, text: &str, font_id: FontId) -> Self {
        self.simple(text, font_id, Color32::WHITE)
    }

    fn simple<C>(self, text: &str, font_id: FontId, color: C) -> Self
    where
        C: Into<Color32>;
}

impl JobExt for LayoutJob {
    fn simple<C>(mut self, text: &str, font_id: FontId, color: C) -> Self
    where
        C: Into<Color32>,
    {
        let fmt = TextFormat {
            font_id,
            color: color.into(),
            ..Default::default()
        };
        self.append(text, 3.0, fmt);
        self
    }
}
