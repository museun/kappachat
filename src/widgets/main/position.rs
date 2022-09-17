use eframe::epaint::Pos2;
use egui::{pos2, vec2, Rect, Vec2};

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Position {
    Top,
    Right,
    Bottom,
    #[default]
    Left,
}

impl Position {
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }

    pub fn rect(&self, size: f32, max: Vec2) -> Rect {
        match self {
            Self::Top => Rect::from_min_size(pos2(0.0, 0.0), max),
            Self::Bottom => Rect::from_min_size(pos2(0.0, max.y), max),
            Self::Left => Rect::from_min_size(pos2(0.0, 0.0), max),
            Self::Right => Rect::from_min_size(pos2(max.x, 0.0), max),
        }
    }

    // BUG why is this different from `rect`?
    pub fn rects(size: f32, max: Vec2) -> [(Self, Rect); 4] {
        use Position::*;

        let right = vec2(max.x, size);
        let bottom = vec2(size, max.y);

        [
            (Top, Rect::from_min_size(Pos2::ZERO, right)),
            (Right, Rect::from_min_size(pos2(max.x - size, 0.0), bottom)),
            (Bottom, Rect::from_min_size(pos2(0.0, max.y - size), right)),
            (Left, Rect::from_min_size(Pos2::ZERO, bottom)),
        ]
    }

    pub const fn as_side(&self) -> Option<egui::panel::Side> {
        Some(match self {
            Self::Right => egui::panel::Side::Right,
            Self::Left => egui::panel::Side::Left,
            _ => return None,
        })
    }

    pub const fn as_top_bottom(&self) -> Option<egui::panel::TopBottomSide> {
        Some(match self {
            Self::Bottom => egui::panel::TopBottomSide::Bottom,
            Self::Top => egui::panel::TopBottomSide::Top,
            _ => return None,
        })
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Top => "Top",
            Self::Right => "Right",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
        }
    }
}
