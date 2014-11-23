extern crate cgmath;

use anchor_ent::AnchorEnt;
use world_manifold::WorldManifold;
use swarm_ent::SwarmEnt;

use cgmath::{Vector, Vector3, EuclideanVector};


static SWARM_FIELD_SIZE: f32 = 2.0;
static SWARM_FIELD_STR: f32 = 1.0;
static GRAVITY_STR: f32 = -10.0;
static COLL_DIAMETER: f32 = 1.0;

pub struct Collision {
  pub ent1_id: int,
  pub ent2_id: int,
}

pub struct EntityField {
  pub anchor: AnchorEnt,
  pub world: WorldManifold,
  pub swarm: Vec<SwarmEnt>
}

impl EntityField {
  pub fn default() -> EntityField {
    let anchor = AnchorEnt::default();
    let world = WorldManifold::default();
    let swarm = vec![SwarmEnt{id: 0, pos: Vector3::new(0.1,1.0,2.0), vel: Vector3::new(0.0, 1.0, 0.5)},
                     SwarmEnt{id: 1, pos: Vector3::new(0.3,0.5,2.0), vel: Vector3::new(0.1, 1.0, 0.2)},
        ];

    return EntityField{anchor: anchor, world: world, swarm: swarm};
  }

  pub fn tick(&mut self, delta_t: f32) -> () {
    for entity in self.swarm.iter() {
      // I dont think this should use delta t, dt is factored in @ integration time
      self.world.deform(entity.pos, SWARM_FIELD_STR * delta_t, SWARM_FIELD_SIZE);
    }

    for entity in self.swarm.iter_mut() {
      let anchor_accel = self.anchor.damped_force_at(entity.pos, entity.vel);
      let swarm_accel = self.world.gradient_at(entity.pos);
      let gravity_accel = Vector3::new(0.0, 0.0, GRAVITY_STR);
      let total_accel = anchor_accel.add_v(&swarm_accel).add_v(&gravity_accel);

      entity.integrate(delta_t, total_accel);
    }

    self.resolve_all_collisions();

  }

  fn resolve_all_collisions(&mut self) {
    let mut collisions = self.find_collisions();
    let mut iterations: int = 0;

    while( !collisions.is_empty() && iterations < 5 ) {

      for collision in collisions.iter() {
        self.resolve_single_collision(collision);
      }

      iterations = iterations + 1;
      collisions = self.find_collisions();
    }
  }

  fn find_collisions(&self) -> Vec<Collision> {
    let mut collisions: Vec<Collision> = Vec::new();
    for first_ent in self.swarm.iter() {
      for second_ent in self.swarm.iter() {
        if first_ent.id < second_ent.id {
          // Collision detection
          if first_ent.pos.sub_v(&second_ent.pos).length() < COLL_DIAMETER {
            collisions.push(Collision{ent1_id: first_ent.id, ent2_id: second_ent.id})
          }
        }
      }
    }
    return collisions;
  }

  fn resolve_single_collision(&mut self, collision: &Collision) -> () {
    let first_id = collision.ent1_id;
    let second_id = collision.ent2_id;

    // weird optimization
    let mut first_ent = self.swarm[mut][first_id as uint];
    let mut second_ent = self.swarm[mut][second_id as uint];

    // Move the two ents
    let collision_vec = first_ent.pos.sub_v(&second_ent.pos);
    let overlap = COLL_DIAMETER - collision_vec.length();
    first_ent.pos = first_ent.pos.add_v(&collision_vec.normalize_to(overlap/2.0));
    second_ent.pos = second_ent.pos.add_v(&collision_vec.neg().normalize_to(overlap/2.0));

    // Make them bounce
    let total_vel = first_ent.vel.add_v(&second_ent.vel);
  }
}



