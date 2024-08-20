use nalgebra::Vector2;

pub struct Player {
    pub pos: Vector2<f32>,
    pub a: f32, // angle of view
    pub fov: f32, // field of view
}
