//! The frame of the scene that the drawing modules share: the screen's size, where the horizon is, the time and the
//! weights of the sky. (The living city of 1.8 is drawn by street.rs, floor.rs, drones.rs and look.rs, translated
//! from pulse.js.)

#[derive(Clone, Copy)]
pub struct Frame {
    pub w: f32,
    pub h: f32,
    pub ox: f32,
    pub oy: f32,
    pub hz: f32,
    pub u: f32,
    pub t: f32,
    /// the weights of the sky: night, twilight, day
    pub sky: [f32; 3],
}
