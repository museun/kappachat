use egui::{text::LayoutJob, Color32, FontId, TextFormat};

pub trait JobExt: Sized {
    fn simple_with_space<C>(self, text: &str, font_id: FontId, color: C, space: f32) -> Self
    where
        C: Into<Color32>;

    fn simple<C>(self, text: &str, font_id: FontId, color: C) -> Self
    where
        C: Into<Color32>,
    {
        self.simple_with_space(text, font_id, color, 3.0)
    }

    fn simple_no_space<C>(self, text: &str, font_id: FontId, color: C) -> Self
    where
        C: Into<Color32>,
    {
        self.simple_with_space(text, font_id, color, 0.0)
    }
}

impl JobExt for LayoutJob {
    fn simple_with_space<C>(mut self, text: &str, font_id: FontId, color: C, space: f32) -> Self
    where
        C: Into<Color32>,
    {
        let fmt = TextFormat {
            font_id,
            color: color.into(),
            ..Default::default()
        };
        self.append(text, space, fmt);
        self
    }
}
