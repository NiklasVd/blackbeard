use std::f32::consts::PI;
use binary_stream::BinaryStream;
use nalgebra::{ComplexField, RealField};
use rand::Rng;
use rapier2d::{math::{Isometry, Real, Vector}, na::{Point2}};
use tetra::{Context, time::get_delta_time};
use crate::V2;

pub fn get_dt(ctx: &mut Context) -> f32 {
    get_delta_time(ctx).as_secs_f32()
}

pub fn conv_rvec(m_vec: Vector<Real>) -> V2 {
    V2::new(m_vec.x, m_vec.y)
}

pub fn conv_vec(vec: V2) -> Vector<Real> {
    Vector::new(vec.x, vec.y)
}

pub fn conv_vec_point(vec: V2) -> Point2<f32> {
    Point2::new(vec.x, vec.y)
}

pub fn conv_point_vec(point: Point2<f32>) -> V2 {
    V2::new(point.x, point.y)
}

pub fn disassemble_iso(iso: &Isometry<Real>) -> (V2, f32) {
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
    // Non-deterministic version: (dir.y).atan2(dir.x)
    RealField::atan2(dir.y, dir.x)
}

pub fn cartesian_to_polar(cartesian_coord: V2) -> (f32, f32) {
    let angle = get_angle(cartesian_coord);
    let dist = ComplexField::sqrt(cartesian_coord.x.powi(2) + cartesian_coord.y.powi(2));
    (dist, angle)
}

pub fn polar_to_cartesian(dist: f32, angle: f32) -> V2 {
    V2::new(dist * ComplexField::cos(angle), dist * ComplexField::sin(angle))
}

pub fn rand_u32(min: u32, max: u32) -> u32 {
    rand::thread_rng().gen_range(min..=max)
}

pub fn serialize_v2(stream: &mut BinaryStream, vec: V2) -> std::io::Result<()> {
    stream.write_f32(vec.x)?;
    stream.write_f32(vec.y)
}

pub fn deserialize_v2(stream: &mut BinaryStream) -> V2 {
    let x = stream.read_f32().unwrap();
    let y = stream.read_f32().unwrap();
    V2::new(x, y)
}

pub struct Timer {
    pub curr_time: f32,
    pub max: f32
}

impl Timer {
    pub fn new(max: f32) -> Timer {
        Timer {
            curr_time: max, max
        }
    }

    pub fn start(max: f32) -> Timer {
        Timer {
            curr_time: 0.0, max
        }
    }

    pub fn update(&mut self, ctx: &mut Context) {
        self.curr_time += get_dt(ctx);
    }

    pub fn is_running(&self) -> bool {
        !self.is_over()
    }

    pub fn is_over(&self) -> bool {
        self.curr_time >= self.max
    }

    pub fn time_until_over(&self) -> f32 {
        self.max - self.curr_time
    }

    pub fn run(&mut self, ctx: &mut Context) -> bool {
        self.update(ctx);
        self.is_over()
    }

    pub fn reset(&mut self) {
        self.curr_time = 0.0;
    }

    pub fn end(&mut self) {
        self.curr_time = self.max;
    }
}
