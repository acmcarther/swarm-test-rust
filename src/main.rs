#![feature(phase)]
#![feature(slicing_syntax)]
// Graphics
extern crate cgmath;
extern crate gfx;
extern crate device;
extern crate render;
#[phase(plugin)]
extern crate gfx_macros;
extern crate glfw;
extern crate genmesh;
extern crate time;

use entity_field::EntityField;

use cgmath::FixedArray;
use cgmath:: {Matrix4, Point3};
use cgmath::{Vector2, Vector3};
use cgmath::{Transform, AffineMatrix3};
use gfx::{Device, DeviceHelper, ToSlice};
use device::BufferUsage;
use render::mesh::Mesh;
use glfw::Context;
use std::rand;
use std::rand::Rng;

use genmesh::{Vertices, Triangulate};
use genmesh::generators::{Plane, SharedVertex, IndexedPolygon};

use std::io::File;

mod entity_field;
mod anchor_ent;
mod swarm_ent;
mod world_manifold;

// Graphics
#[vertex_format]
struct Vertex {
  #[name = "u_pos"]
  pos: [f32, ..3],

  #[name = "u_normal"]
  normal: [f32, ..3],

  #[name = "u_uv"]
  uv: [f32, ..2],
}

#[shader_param(Entity)]
struct Params {
  #[name= "u_Model"]
  model: [[f32, ..4], ..4],

  #[name= "u_View"]
  view: [[f32, ..4], ..4],

  #[name= "u_Proj"]
  proj: [[f32, ..4], ..4],

  #[name= "t_Color"]
  color: gfx::shade::TextureParam,

  #[name = "world_light_pos"]
  light_pos: [f32, ..3],
}

// --------- Main -----------

fn generate_model() -> Vec<Vertex> {
  vec![
      Vertex { pos: [-1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0]},
      Vertex { pos: [ 1.0, -1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0]},
      Vertex { pos: [ 1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0]},
      Vertex { pos: [-1.0,  1.0,  1.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0]},

      // bottom (0, 0, -1)
      Vertex { pos: [-1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0]},
      Vertex { pos: [ 1.0,  1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0]},
      Vertex { pos: [ 1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0]},
      Vertex { pos: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0]},
      // right (1, 0, 0)
      Vertex { pos: [ 1.0, -1.0, -1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 0.0]},
      Vertex { pos: [ 1.0,  1.0, -1.0], normal: [1.0, 0.0, 0.0], uv: [0.0, 1.0]},
      Vertex { pos: [ 1.0,  1.0,  1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 0.0]},
      Vertex { pos: [ 1.0, -1.0,  1.0], normal: [1.0, 0.0, 0.0], uv: [1.0, 1.0]},
      // left (-1, 0, 0)
      Vertex { pos: [-1.0, -1.0,  1.0], normal: [-1.0, 0.0, 0.0], uv: [0.0, 0.0]},
      Vertex { pos: [-1.0,  1.0,  1.0], normal: [-1.0, 0.0, 0.0], uv: [0.0, 1.0]},
      Vertex { pos: [-1.0,  1.0, -1.0], normal: [-1.0, 0.0, 0.0], uv: [1.0, 0.0]},
      Vertex { pos: [-1.0, -1.0, -1.0], normal: [-1.0, 0.0, 0.0], uv: [1.0, 1.0]},
      // front (0, 1, 0)
      Vertex { pos: [ 1.0,  1.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 0.0]},
      Vertex { pos: [-1.0,  1.0, -1.0], normal: [0.0, 1.0, 0.0], uv: [0.0, 1.0]},
      Vertex { pos: [-1.0,  1.0,  1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 0.0]},
      Vertex { pos: [ 1.0,  1.0,  1.0], normal: [0.0, 1.0, 0.0], uv: [1.0, 1.0]},
      // back (0, -1, 0)
      Vertex { pos: [ 1.0, -1.0,  1.0], normal: [0.0, -1.0, 0.0], uv: [0.0, 0.0]},
      Vertex { pos: [-1.0, -1.0,  1.0], normal: [0.0, -1.0, 0.0], uv: [0.0, 1.0]},
      Vertex { pos: [-1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], uv: [1.0, 0.0]},
      Vertex { pos: [ 1.0, -1.0, -1.0], normal: [0.0, -1.0, 0.0], uv: [1.0, 1.0]},
  ]
}

fn main() {
  println!("Tra-la-la");

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

  let ent_data: Vec<Vertex> = generate_model();
  let anchor_data: Vec<Vertex> = generate_model();

  let ent_mesh = device.create_mesh(ent_data.as_slice());
  let anchor_mesh = device.create_mesh(anchor_data.as_slice());

  let plane = Plane::subdivide(512,512);
  let mut plane_vertex_data: Vec<Vertex> = plane.shared_vertex_iter()
      .map(|(x, y)| {
        Vertex{ 
          pos: [ x*100.0, y*100.0, 1.0],
          normal: [0.0, 0.0, 1.0],
          uv: [0.7, 0.7]
        }
      })
      .collect();

  let plane_index_data: Vec<u32> = plane.indexed_polygon_iter()
      .triangulate()
      .vertices()
      .map(|i| i as u32)
      .collect();

  // Using direct instantiation instead of easy mode helpers, these are static though
  let mut plane_idx_buffer = device.create_buffer::<u32>(plane_index_data.len(), device::BufferUsage::Static);
  device.update_buffer(plane_idx_buffer, plane_index_data.as_slice(), 0u);
  let plane_slice = plane_idx_buffer.to_slice(gfx::TriangleList);

  // Dynamic vertices here
  let mut plane_vert_buffer = device.create_buffer(plane_vertex_data.len(), device::BufferUsage::Stream);
  device.update_buffer(plane_vert_buffer, plane_vertex_data.as_slice(), 0u);
  let plane_mesh = render::mesh::Mesh::from_format(plane_vert_buffer, plane_vertex_data.len() as device::VertexCount);

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

  let texture_info = gfx::tex::TextureInfo {
    width: 1,
    height: 1,
    depth: 1,
    levels: 1,
    kind: gfx::tex::Texture2D,
    format: gfx::tex::RGBA8,
  };

  let image_info = texture_info.to_image_info();
  let texture = device.create_texture(texture_info).unwrap();
  device.update_texture(&texture, &image_info, &[0x20u8, 0xA0u8, 0xC0u8, 0x00u8]).unwrap();

  let sampler = device.create_sampler(
    gfx::tex::SamplerInfo::new(gfx::tex::Bilinear, gfx::tex::Clamp)
  );

  let vertex_shader_text: Vec<u8> = File::open(&Path::new("vertex-shader.glsl"))
      .read_to_end()
      .unwrap();

  let fragment_shader_text: Vec<u8> = File::open(&Path::new("fragment-shader.glsl"))
      .read_to_end()
      .unwrap();

  let vertex_shader: gfx::ShaderSource = shaders! {
    GLSL_150: vertex_shader_text.as_slice()
  };

  let fragment_shader: gfx::ShaderSource = shaders! {
    GLSL_150: fragment_shader_text.as_slice()
  };

  let program = device.link_program(vertex_shader.clone(), fragment_shader.clone())
                      .unwrap();

  let state = gfx::DrawState::new().depth(gfx::state::LessEqual, true);
  let mut graphics = gfx::Graphics::new(device);

  let ent_batch: Entity = graphics.make_batch(&program, &ent_mesh, slice, &state).unwrap();
  let anchor_batch: Entity = graphics.make_batch(&program, &anchor_mesh, slice, &state).unwrap();
  let plane_batch: Entity = graphics.make_batch(&program, &plane_mesh, plane_slice, &state).unwrap();

  let aspect = w as f32 / h as f32;
  let mut data = Params {
      light_pos: Vector3::new(0.0, 0.0, -3.0).into_fixed(),
      color: (texture, Some(sampler)),
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

    for vertex in plane_vertex_data.iter_mut() {
      let height = everything.world.height_at(Vector2::new(vertex.pos[0], vertex.pos[1]));
      vertex.pos = [vertex.pos[0], vertex.pos[1], height/20.0];
    }
    graphics.device.update_buffer(plane_vert_buffer, plane_vertex_data.as_slice(), 0u);

    let current_time = time::precise_time_ns();
    let delta_t = ((current_time - last_time) as f32) / 1_000_000_000.0 ;
    last_time = current_time;

    everything.tick(delta_t);

    if going_left  {
      camera_setting = camera_setting + (1.5 * delta_t)
    }

    if going_right  {
      camera_setting = camera_setting - (1.5 * delta_t)
    }

    if going_fore  {
      range_setting = range_setting - (15.0 * delta_t)
    }

    if going_back  {
      range_setting = range_setting + (15.0 * delta_t)
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
          for entity in everything.swarm.iter_mut() {
            let new_x = rng.gen_range(-10.0, 10.0);
            let new_y = rng.gen_range(-10.0, 10.0);
            entity.vel = Vector3::new(0.0, 0.0, 0.0);
            entity.pos = Vector3::new(new_x, new_y, 20.0);
          }
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
    
    // Draw plane
    data.model = Matrix4::from_translation(&Vector3::new(0.0, 0.0, -10.0)).into_fixed();
    graphics.draw(&plane_batch, &data, &frame);

    // Draw anchor
    data.model = Matrix4::from_translation(&Vector3::new(0.0, 0.0, 0.0)).into_fixed();
    graphics.draw(&anchor_batch, &data, &frame);

    // Draw entities
    for entity in everything.swarm.iter() {
      let ent_pos = entity.pos;
      data.model = Matrix4::from_translation(&Vector3::new(ent_pos.x, ent_pos.y, ent_pos.z)).into_fixed();
      graphics.draw(&ent_batch, &data, &frame);
    }

    graphics.end_frame();

    window.swap_buffers();
  }
}
