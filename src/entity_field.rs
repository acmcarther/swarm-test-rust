extern crate cgmath;

use anchor_ent::AnchorEnt;
use world_manifold::WorldManifold;
use swarm_ent::SwarmEnt;

use cgmath::{Vector, Vector3, EuclideanVector};

static SWARM_FIELD_SIZE: f32 = 2.0;
static SWARM_FIELD_STR: f32 = 1.0;
static GRAVITY_STR: f32 = -10.0;

pub struct EntityField {
  pub anchor: AnchorEnt,
  pub world: WorldManifold,
  pub swarm: Vec<SwarmEnt>
}

impl EntityField {
  pub fn default() -> EntityField {
    let anchor = AnchorEnt::default();
    let world = WorldManifold::default();
    let swarm = vec![SwarmEnt{id: 0, pos: Vector3::new(0.1,1.0,2.0), vel: Vector3::new(0.0, 1.0, 0.5)}];

    return EntityField{anchor: anchor, world: world, swarm: swarm};
  }

  pub fn tick(&mut self, delta_t: f32) -> () {
    for entity in self.swarm.iter() {
      self.world.deform(entity.pos, SWARM_FIELD_STR * delta_t, SWARM_FIELD_SIZE);
    }

    for entity in self.swarm.iter_mut() {
      let anchor_accel = self.anchor.damped_force_at(entity.pos, entity.vel);
      let swarm_accel = self.world.gradient_at(entity.pos);
      let gravity_accel = Vector3::new(0.0, 0.0, GRAVITY_STR);
      let total_accel = anchor_accel.add_v(&swarm_accel).add_v(&gravity_accel);

      entity.integrate(delta_t, total_accel);
    }

    let mut collisions: Vec<int> = Vec::new();

    for first_ent in self.swarm.iter() {
      for second_ent in self.swarm.iter() {
        if first_ent.id < second_ent.id {
          // Collision detection
        }
      }
    }

    for collision in collisions.iter() {
      // Collision resolution
    }
  }
}



