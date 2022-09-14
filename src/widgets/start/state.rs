use super::StartRotation;

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

impl StartState {
    pub fn new(kappas: Vec<egui_extras::RetainedImage>) -> Self {
        Self {
            kappas,
            ..Default::default()
        }
    }
}
