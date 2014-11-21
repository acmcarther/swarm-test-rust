
#![feature(phase)]
// Graphics
extern crate cgmath;
extern crate gfx;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate native;
extern crate time;

use cgmath::FixedArray;
use cgmath:: {Matrix4, Point3, Vector3};
use cgmath::{Transform, AffineMatrix3};
use gfx::{Device, DeviceHelper, ToSlice};
use glfw::Context;
use std::rand;
use std::rand::Rng;

// Others

static SPACE_FRICTION_COEF: f32 = 5.0;
static ANCHOR_FIELD_LEN: f32 = 15.0;
static ANCHOR_FIELD_STR: f32 = 15.0;
static ANCHOR_FIELD_DAMP: f32 = 0.2;
static SWARM_FIELD_SIZE: f32 = 2.0;
static SWARM_FIELD_STR: f32 = 1.0;
static GRAVITY_STR: f32 = -10.0;

// Graphics
#[vertex_format]
struct Vertex {
  #[name = "a_Pos"]
  pos: [f32, ..3],

  #[name = "a_Color"]
  color: [f32, ..3],
}

#[shader_param(Entity)]
struct Params {
  #[name= "u_Model"]
  model: [[f32, ..4], ..4],

  #[name= "u_View"]
  view: [[f32, ..4], ..4],

  #[name= "u_Proj"]
  proj: [[f32, ..4], ..4],
}

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    attribute vec3 a_Pos;
    attribute vec3 a_Color;
    varying vec3 v_Color;

    uniform mat4 u_Model;
    uniform mat4 u_View;
    uniform mat4 u_Proj;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
"
GLSL_150: b"
    #version 150 core
    in vec3 a_Pos;
    in vec3 a_Color;
    out vec3 v_Color;

    uniform mat4 u_Model;
    uniform mat4 u_View;
    uniform mat4 u_Proj;

    void main() {
        v_Color = a_Color;
        gl_Position = u_Proj * u_View * u_Model * vec4(a_Pos, 1.0);
    }
"
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
GLSL_120: b"
    #version 120

    varying vec3 v_Color;
    out vec4 o_Color;

    void main() {
      o_Color = vec4(v_Color, 1.0);
    }
"
GLSL_150: b"
    #version 150 core
    in vec3 v_Color;
    out vec4 o_Color;

    uniform sampler2D t_Color;
    void main() {
       o_Color = vec4(v_Color, 1.0);
    }
"
};

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
      let gravity_accel = Vec3::new(0.0, 0.0, GRAVITY_STR);
      let total_accel = anchor_accel.transform(swarm_accel).transform(gravity_accel);

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
    // Probably a 2d array. Maybe something else thats faster
    // Composition of gaussians faster?
    return WorldManifold{power_level: 5};
  }

  pub fn flatten(&mut self) -> () {
    // NOTE: What
    // Probably set all indices in the array to zero. Ouch.
    self.power_level = 0
  }

  pub fn deform(&self, pos: Vec3, magnitude: f32, diameter: f32) -> () {
    // TODO: Something. At all.
    // Probably apply a gaussian deformation onto a 2d array
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
  distance: f32,
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

    return delta.transform(idle_pos.negated()).scale(self.strength);
  }
}

// --------- Main -----------

fn generate_colored_model(r:f32, g:f32, b:f32) -> Vec<Vertex> {
  vec![
      Vertex { pos: [-1.0, -1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0, -1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0,  1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [-1.0,  1.0,  1.0], color: [r, g, b] },
      // bottom (0, 0, -1)
      Vertex { pos: [-1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0, -1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [-1.0, -1.0, -1.0], color: [r, g, b] },
      // right (1, 0, 0)
      Vertex { pos: [ 1.0, -1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0,  1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0, -1.0,  1.0], color: [r, g, b] },
      // left (-1, 0, 0)
      Vertex { pos: [-1.0, -1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [-1.0,  1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [-1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [-1.0, -1.0, -1.0], color: [r, g, b] },
      // front (0, 1, 0)
      Vertex { pos: [ 1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [-1.0,  1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [-1.0,  1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0,  1.0,  1.0], color: [r, g, b] },
      // back (0, -1, 0)
      Vertex { pos: [ 1.0, -1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [-1.0, -1.0,  1.0], color: [r, g, b] },
      Vertex { pos: [-1.0, -1.0, -1.0], color: [r, g, b] },
      Vertex { pos: [ 1.0, -1.0, -1.0], color: [r, g, b] },
  ]
}

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
  native::start(argc, argv, main)
}

fn main() {
  println!("Hello, world!");
  println!("Now from my gaping maw I sing the song to end everthing.");

  let glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

  glfw.window_hint(glfw::ContextVersion(3, 2));
  glfw.window_hint(glfw::OpenglForwardCompat(true));
  glfw.window_hint(glfw::OpenglProfile(glfw::OpenGlProfileHint::Core));

  let (window, events) = glfw
      .create_window(960, 1080, "Physics example", glfw::Windowed)
      .expect("Failed to create GLFW window.");

  window.make_current();
  glfw.set_error_callback(glfw::FAIL_ON_ERRORS);
  window.set_key_polling(true);

  let (w, h) = window.get_framebuffer_size();
  let frame = gfx::Frame::new(w as u16, h as u16);

  let mut device = gfx::GlDevice::new(|s| window.get_proc_address(s));

  let ent_data: Vec<Vertex> = generate_colored_model(0.2, 0.2, 0.7);
  let anchor_data: Vec<Vertex> = generate_colored_model(0.7, 0.2, 0.2);

  let ent_mesh = device.create_mesh(ent_data.as_slice());
  let anchor_mesh = device.create_mesh(anchor_data.as_slice());

  let index_data: Vec<u32> = vec![
       0,  1,  2,  2,  3,  0, // top
       4,  5,  6,  6,  7,  4, // bottom
       8,  9, 10, 10, 11,  8, // right
       12, 13, 14, 14, 15, 12, // left
       16, 17, 18, 18, 19, 16, // front
       20, 21, 22, 22, 23, 20, // back
  ];

  let slice = device
      .create_buffer_static::<u32>(index_data.as_slice())
      .to_slice(gfx::TriangleList);

  let program = device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone())
                      .unwrap();

  let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
  let mut graphics = gfx::Graphics::new(device);

  let ent_batch: Entity = graphics.make_batch(&program, &ent_mesh, slice, &state).unwrap();
  let anchor_batch: Entity = graphics.make_batch(&program, &anchor_mesh, slice, &state).unwrap();

  let aspect = w as f32 / h as f32;
  let mut data = Params {
      model: Matrix4::identity().into_fixed(),
      view: Matrix4::identity().into_fixed(),
      proj: cgmath::perspective(cgmath::deg(60.0f32), aspect, 0.1, 1000.0).into_fixed(),
  };

  let clear_data = gfx::ClearData {
      color: [0.1, 0.1, 0.1, 1.0],
      depth: 1.0,
      stencil: 0,
  };

  let mut last_time = time::precise_time_ns();
  let mut everything = EntityField::default();
  let mut camera_setting = 0.0;
  let mut range_setting = 16.0;

  let mut going_left = false;
  let mut going_right = false;
  let mut going_fore = false;
  let mut going_back = false;

  let mut rng = rand::task_rng();

  while !window.should_close() {
    let current_time = time::precise_time_ns();
    let delta_t = ((current_time - last_time) as f32) / 1_000_000_000.0 ;
    last_time = current_time;

    everything.tick(delta_t);

    if going_left  {
      camera_setting = camera_setting + 0.1
    }

    if going_right  {
      camera_setting = camera_setting - 0.1
    }

    if going_fore  {
      range_setting = range_setting - 1.0
    }

    if going_back  {
      range_setting = range_setting + 1.0
    }

    glfw.poll_events();
    for (_, event) in glfw::flush_messages(&events) {
        match event {
            glfw::KeyEvent(glfw::Key::Escape, _, glfw::Press, _) =>
               window.set_should_close(true),
            glfw::KeyEvent(glfw::Key::H, _, glfw::Press, _) =>
               going_left = true,
            glfw::KeyEvent(glfw::Key::L, _, glfw::Press, _) =>
               going_right = true,
            glfw::KeyEvent(glfw::Key::H, _, glfw::Release, _) =>
               going_left = false,
            glfw::KeyEvent(glfw::Key::L, _, glfw::Release, _) =>
               going_right = false,
            glfw::KeyEvent(glfw::Key::J, _, glfw::Press, _) =>
               going_fore = true,
            glfw::KeyEvent(glfw::Key::K, _, glfw::Press, _) =>
               going_back = true,
            glfw::KeyEvent(glfw::Key::J, _, glfw::Release, _) =>
               going_fore = false,
            glfw::KeyEvent(glfw::Key::K, _, glfw::Release, _) =>
               going_back = false,
            glfw::KeyEvent(glfw::Key::R, _, glfw::Press, _) => {
               let ent = everything.swarm.get_mut(0).unwrap();
               let new_x = rng.gen_range(-10.0, 10.0);
               let new_y = rng.gen_range(-10.0, 10.0);
               ent.vel = Vec3::new(0.0, 0.0, 0.0);
               ent.pos = Vec3::new(new_x, new_y, 20.0);
            },
            _ => {},
        }
    }

    let x = std::num::FloatMath::sin(camera_setting);
    let y = std::num::FloatMath::cos(camera_setting);

    let view:AffineMatrix3<f32> = Transform::look_at(
      &Point3::new(2.0*range_setting*x, 2.0*range_setting*y, range_setting),
      &Point3::new(0.0, 0.0, 0.0),
      &Vector3::unit_z(),
    );
    data.view = view.mat.into_fixed();

    graphics.clear(clear_data, gfx::COLOR | gfx::DEPTH, &frame);


    // Draw anchor
    data.model = Matrix4::from_translation(&Vector3::new(0.0, 0.0, 0.0)).into_fixed();
    graphics.draw(&anchor_batch, &data, &frame);

    // Draw entity
    let ent_pos = everything.swarm.get(0).unwrap().pos;
    data.model = Matrix4::from_translation(&Vector3::new(ent_pos.x, ent_pos.y, ent_pos.z)).into_fixed();
    graphics.draw(&ent_batch, &data, &frame);

    graphics.end_frame();

    window.swap_buffers();
  }
}
