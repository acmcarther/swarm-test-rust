extern crate cgmath;

use std::fmt;
use cgmath::{Vector, Vector3};

pub struct SwarmEnt {
  pub id: int,
  pub pos: Vector3<f32>,
  pub vel: Vector3<f32>,
}

impl SwarmEnt {
  pub fn integrate(&mut self, delta_t: f32, accel: Vector3<f32>) -> () {
    self.vel = self.vel.add_v(&accel.mul_s(delta_t));
    self.pos = self.pos.add_v(&self.vel.mul_s(delta_t));
  }
}

impl fmt::Show for SwarmEnt {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "SwarmEnt: {}, pos: {}, vel {}", self.id, self.pos, self.vel)
  }
}
