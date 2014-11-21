extern crate time;

static SPACE_FRICTION_COEF: f32 = 5.0;
static ANCHOR_FIELD_LEN: f32 = 5.0;
static ANCHOR_FIELD_STR: f32 = 10.0;
static ANCHOR_FIELD_DAMP: f32 = 0.8;
static SWARM_FIELD_SIZE: f32 = 2.0;
static SWARM_FIELD_STR: f32 = 1.0;


// --------- PURE MATH -----------------
pub struct Vec3 {
  x: f32,
  y: f32,
  z: f32
}

impl std::cmp::PartialEq for Vec3 {

  fn eq(&self, other: &Vec3) -> bool {
    self.x == other.x && self.y == other.y && self.z == other.z
  }

  fn ne(&self, other: &Vec3) -> bool {
    !self.eq(other)
  }
}

impl Vec3 {

  fn new(x: f32, y: f32, z: f32) -> Vec3 {
    return Vec3{x: x, y: y, z: z};
  }

  fn transform(&self, xformer: Vec3) -> Vec3 {
    let x = self.x + xformer.x;
    let y = self.y + xformer.y;
    let z = self.z + xformer.z;

    return Vec3::new(x, y, z);
  }

  fn scale(&self, scalar: f32) -> Vec3 {
    let x = self.x * scalar;
    let y = self.y * scalar;
    let z = self.z * scalar;

    return Vec3::new(x, y, z);
  }

  fn vec_delta(v1: Vec3, v2: Vec3) -> Vec3 {
    let dx = v1.x - v2.x;
    let dy = v1.y - v2.y;
    let dz = v1.z - v2.z;

    return Vec3::new(dx, dy, dz);
  }

  fn negated(&self) -> Vec3 {
    return Vec3::new(-self.x, -self.y, -self.z);
  }

  fn norm(&self) -> f32 {
    return std::num::Float::sqrt(self.x*self.x + self.y*self.y + self.z*self.z);
  }

  fn normalized(&self) -> Vec3 {
    let l = self.norm();
    if l == 0.0 {
      return Vec3{x: 0.0, y: 0.0, z: 0.0};
    }
    return Vec3{x: self.x/l, y: self.y/l, z: self.z/l};
  }

  fn print(&self) -> () {
    print!("Vec3({},", self.x);
    print!("{},", self.y);
    println!("{})", self.z);
  }
}

// --------- Entities -----------
pub struct EntityField {
  anchor: AnchorEnt,
  world: WorldManifold,
  swarm: Vec<SwarmEnt>
}

impl EntityField {
  pub fn default() -> EntityField {
    let anchor = AnchorEnt::default();
    let world = WorldManifold::default();
    let swarm = vec![SwarmEnt{pos: Vec3::new(0.1,1.0,2.0), vel: Vec3::new(0.0, 1.0, 0.5)}];

    return EntityField{anchor: anchor, world: world, swarm: swarm};
  }

  pub fn tick(&mut self, delta_t: f32) -> () {
    for entity in self.swarm.iter() {
      self.world.deform(entity.pos, SWARM_FIELD_STR * delta_t, SWARM_FIELD_SIZE);
    }

    for entity in self.swarm.iter_mut() {
      let anchor_accel = self.anchor.damped_force_at(entity.pos, entity.vel);
      let swarm_accel = self.world.gradient_at(entity.pos);
      let total_accel = anchor_accel.transform(swarm_accel);

      // NOTE: Remember, no damping
      entity.integrate(delta_t, total_accel);
    }
  }
}

pub struct WorldManifold {
  power_level: int
}

impl WorldManifold {
  pub fn default() -> WorldManifold {
    // TODO: Make this an actual thing not an integer throne
    return WorldManifold{power_level: 5};
  }

  pub fn flatten(&mut self) -> () {
    // NOTE: What
    self.power_level = 0
  }

  pub fn deform(&self, pos: Vec3, magnitude: f32, diameter: f32) -> () {
    // TODO: Something. At all.
    //println!("DEFORM");
  }

  pub fn gradient_at(&self, pos: Vec3) -> Vec3 {
    // TODO: When we get a model, get a gradient
    return Vec3{x: 0.0, y: 0.0, z: 0.0};
  }
}

pub struct SwarmEnt {
  pos: Vec3,
  vel: Vec3
}

impl SwarmEnt {
  pub fn at(pos: Vec3) -> SwarmEnt {
    let vel = Vec3{x: 0.0, y: 0.0, z: 0.0};

    return SwarmEnt{pos: pos, vel: vel};
  }

  pub fn default() -> SwarmEnt {
    let pos = Vec3{x: 0.0, y: 0.0, z: 0.0};
    let vel = Vec3{x: 0.0, y: 0.0, z: 0.0};

    return SwarmEnt{pos: pos, vel: vel};
  }

  pub fn integrate(&mut self, delta_t: f32, accel: Vec3) -> () {
    self.print();
    self.vel = self.vel.transform(accel.scale(delta_t));
    self.pos = self.pos.transform(self.vel.scale(delta_t));
  }

  pub fn print(&self) -> () {
    //println!("Ent @ ");
    self.pos.print();
  }
}

pub struct AnchorEnt {
  pos: Vec3,
  strength: f32,
  distance: f32
}

impl AnchorEnt {
  pub fn default() -> AnchorEnt {
    let pos = Vec3{x: 0.0, y: 0.0, z: 0.0};

    return AnchorEnt{pos: pos, strength: ANCHOR_FIELD_STR, distance: ANCHOR_FIELD_LEN};
  }
  
  pub fn damped_force_at(&self, other_pos: Vec3, other_vel: Vec3) -> Vec3 {
    let damping_factor = other_vel.scale(ANCHOR_FIELD_DAMP).negated();
    // NOTE: Assumes insignificant anchor velocity
    return self.force_at(other_pos).transform(damping_factor);
  }

  pub fn force_at(&self, other_pos: Vec3) -> Vec3 {
    let delta = Vec3::vec_delta(self.pos, other_pos);
    let idle_pos: Vec3 = if (delta == Vec3{x: 0.0, y: 0.0,z: 0.0}) {
      Vec3{x: self.distance, y: 0.0, z: 0.0}
    } else {
      delta.normalized().scale(self.distance)
    };

    // NOTE: Remember, no damping.
    return delta.transform(idle_pos.negated()).scale(self.strength);
  }
}

// --------- Main -----------
fn main() {
  println!("Hello, world!");
  println!("Now from my gaping maw I sing the song to end the earth.");

  let mut last_time = time::precise_time_ns();
  let mut everything = EntityField::default();

  loop {
    let current_time = time::precise_time_ns();
    let delta_t = ((current_time - last_time) as f32) / 1_000_000_000.0 ;
    last_time = current_time;

    everything.tick(delta_t);
  }
}
