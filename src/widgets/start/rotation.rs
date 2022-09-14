use crate::RequestPaint;

pub struct StartRotation {
    pub rotation: f32,
    pub hovered: bool,
    pub speed: f32,
    pub spinning: bool,
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

    pub fn rotate_cw(&mut self, rot: f32, repaint: &impl RequestPaint) {
        self.speed = (self.speed + Self::SPEED_DELTA).max(Self::SPEED_MAX);
        self.rotation += rot;
        repaint.request_repaint();
    }

    pub fn rotate_ccw(&mut self, rot: f32, repaint: &impl RequestPaint) {
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
