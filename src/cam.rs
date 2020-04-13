
pub struct Cam {
    pub(crate) center: cgmath::Point2<f32>,
    pub(crate) last_mouse_pos: cgmath::Point2<f32>,
    pub(crate) zoom: f32,
}

impl Cam {
    pub fn new() -> Self {
        Cam {
            center: (0.0, 0.0).into(),
            last_mouse_pos: (0.0, 0.0).into(),
            zoom: 30.0,
        }
    }
}