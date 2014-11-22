extern crate cgmath;

use cgmath::{Vector, Vector3, EuclideanVector};

static ANCHOR_FIELD_LEN: f32 = 15.0;
static ANCHOR_FIELD_STR: f32 = 15.0;
static ANCHOR_FIELD_DAMP: f32 = 0.2;

pub struct AnchorEnt {
  pos: Vector3<f32>,
  strength: f32,
  distance: f32,
}

impl AnchorEnt {
  pub fn default() -> AnchorEnt {
    let pos = Vector3::new(0.0, 0.0, 0.0);

    return AnchorEnt{pos: pos, strength: ANCHOR_FIELD_STR, distance: ANCHOR_FIELD_LEN};
  }

  pub fn damped_force_at(&self, other_pos: Vector3<f32>, other_vel: Vector3<f32>) -> Vector3<f32> {
    let damping_factor = other_vel.mul_s(ANCHOR_FIELD_DAMP);
    // NOTE: Assumes insignificant anchor velocity
    return self.force_at(other_pos).sub_v(&damping_factor);
  }

  pub fn force_at(&self, other_pos: Vector3<f32>) -> Vector3<f32> {
    let delta = self.pos.sub_v(&other_pos);
    let idle_pos: Vector3<f32> = if delta == Vector3::new(0.0, 0.0, 0.0) {
      Vector3::new(self.distance, 0.0, 0.0)
    } else {
      delta.normalize_to(self.distance)
    };

    return delta.sub_v(&idle_pos).mul_s(self.strength);
  }
}
