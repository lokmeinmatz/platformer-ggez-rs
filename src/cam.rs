use cgmath::{Vector2, Point2, EuclideanSpace};

pub struct Cam {
    pub(crate) center: Point2<f32>,
    pub(crate) last_mouse_pos: Point2<f32>,
    pub(crate) zoom: f32,
    pub(crate) last_vp_size: Vector2<f32>
}

impl Cam {
    pub fn new(vp_size: Vector2<f32>) -> Self {
        Cam {
            center: (0.0, 0.0).into(),
            last_mouse_pos: (0.0, 0.0).into(),
            zoom: 30.0,
            last_vp_size: vp_size
        }
    }

    pub fn world_to_screen(&self, world_pos: Point2<f32>) -> Point2<f32> {
        (world_pos - self.center.to_vec()) * self.zoom + (self.last_vp_size * 0.5)
    }
}