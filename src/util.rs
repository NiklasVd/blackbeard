use std::f32::consts::PI;
use rapier2d::{math::{Isometry, Real, Vector}};
use tetra::graphics::Texture;
use crate::V2;

pub const UPDATE_TICK_RATE: f32 = 1.0 / 60.0;

pub fn conv_rvec(m_vec: Vector<Real>) -> V2 {
    V2::new(m_vec.x, m_vec.y)
}

pub fn conv_vec(vec: V2) -> Vector<Real> {
    Vector::new(vec.x, vec.y)
}

pub fn deassemble_iso(iso: &Isometry<Real>) -> (V2, f32) {
    (V2::new(iso.translation.vector.x, iso.translation.vector.y), iso.rotation.angle())
}

pub fn pi_to_pi2_range(rads: f32) -> f32 {
    if rads < 0.0 {
        return (PI - rads.abs()) + PI
    }
    else {
        return rads
    }
}

pub fn get_angle(dir: V2) -> f32 {
    (dir.y).atan2(dir.x)
}

pub fn cartesian_to_polar(cartesian_coord: V2) -> (f32, f32) {
    let angle = get_angle(cartesian_coord);
    let dist = (cartesian_coord.x.powi(2) + cartesian_coord.y.powi(2)).sqrt();
    (dist, angle)
}

pub fn polar_to_cartesian(dist: f32, angle: f32) -> V2 {
    V2::new(dist * angle.cos(), dist * angle.sin())
}

pub fn get_texture_origin(tex: Texture) -> V2 {
    V2::new(tex.width() as f32 * 0.5, tex.height() as f32 * 0.5)
}

pub struct Timer {
    pub curr_time: f32,
    pub max: f32
}

impl Timer {
    pub fn new(max: f32) -> Timer {
        Timer {
            curr_time: 0.0, max
        }
    }

    pub fn update(&mut self) {
        self.curr_time += UPDATE_TICK_RATE;
    }

    pub fn is_over(&self) -> bool {
        self.curr_time >= self.max
    } 

    pub fn run(&mut self) -> bool {
        self.update();
        self.is_over()
    }


}
