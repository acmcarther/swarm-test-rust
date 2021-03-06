extern crate cgmath;

use cgmath::{Vector2, Vector3};
use std::collections::HashMap;
use std::num::Float;
use std::num::SignedInt;

static SQRT_2: f32 = 1.414213562;

pub struct Deformation {
  magnitude: int,
  x: f32,
  y: f32
}

pub struct WorldManifold {
  // 100 to 1 scale;
  field: Box<[[f32, ..10000], ..10000]>,
  deform_memo: HashMap<int, Vec<Vec<f32>>>,
  deform_stack: Vec<Deformation>,
}

impl WorldManifold {
  pub fn default() -> WorldManifold {
    return WorldManifold{field: box [[0.0, ..10000], ..10000], deform_memo: HashMap::new(), deform_stack: Vec::new()};
  }

  pub fn flatten(&mut self) -> () {
    for deformation in self.deform_stack.clone().iter() {
      self.deform(Vector3::new(deformation.x, deformation.y, 0.0), -deformation.magnitude);
    }
    self.deform_stack.clear();
  }

  pub fn deform(&mut self, pos: Vector3<f32>, magnitude: int) -> () {
    // TODO: Something. At all.
    // Probably apply a gaussian deformation onto a 2d array
    self.deform_stack.push(Deformation{magnitude: magnitude, x: pos.x, y: pos.y});

    // A deformation matrix
    let deformation = WorldManifold::find_deformation(&mut self.deform_memo, magnitude);
    //println!("{}", deformation);

    let pos = WorldManifold::world_pos_to_field_pos(pos);
    //println!("fieldpos: {}", pos);

    let half_deformation_size = (deformation.len()/2) as uint;
    for (mat_y, row) in deformation.iter().enumerate() {
      for (mat_x, field_str) in row.iter().enumerate() {
        //println!("x: {}, y: {}", mat_x, mat_y);
        // Sorry m8
        self.field[mat_y - half_deformation_size + pos.y][mat_x - half_deformation_size + pos.x] = *field_str + self.field[mat_y - half_deformation_size + pos.y][mat_x - half_deformation_size + pos.x]
      }
    }
  }

  pub fn gradient_at(&self, pos: Vector3<f32>) -> Vector3<f32> {
    let pos = WorldManifold::world_pos_to_field_pos(pos);

    // TODO: A proper kernel or swappable differential kernel
    //println!("new mat: ");
    //println!("[ {}, {}, {} ]", self.field[pos.y-1][pos.x-1], self.field[pos.y-1][pos.x], self.field[pos.y-1][pos.x+1]);
    //println!("[ {}, {}, {} ]", self.field[pos.y][pos.x-1], self.field[pos.y][pos.x], self.field[pos.y][pos.x+1]);
    //println!("[ {}, {}, {} ]", self.field[pos.y+1][pos.x-1], self.field[pos.y+1][pos.x], self.field[pos.y+1][pos.x+1]);
    let dx = -(SQRT_2 * self.field[pos.y+1][pos.x+1] +
              self.field[pos.y][pos.x+1] +
              SQRT_2 * self.field[pos.y-1][pos.x+1])
              +
              (SQRT_2 * self.field[pos.y+1][pos.x-1] +
              self.field[pos.y][pos.x-1] +
              SQRT_2 * self.field[pos.y-1][pos.x-1]);

    let dy = -(SQRT_2 *self.field[pos.y+1][pos.x+1] +
              self.field[pos.y+1][pos.x] +
              SQRT_2 * self.field[pos.y+1][pos.x-1])
              +
              (SQRT_2 * self.field[pos.y-1][pos.x+1] +
              self.field[pos.y-1][pos.x] +
              SQRT_2 * self.field[pos.y-1][pos.x-1]);

    return Vector3::new( dx as f32, dy as f32, 0.0);
    //return Vector3::new( 0.0, 0.0, 0.0);
  }

  pub fn height_at(&self, pos: Vector2<f32>) -> f32 {
    let pos = WorldManifold::world_pos_to_field_pos_2d(pos);
    return self.field[pos.y][pos.x].clone();
  }

  fn find_deformation(deform_memo: &mut HashMap<int, Vec<Vec<f32>>>, magnitude: int) -> Vec<Vec<f32>> {

    match deform_memo.get(&magnitude) {
      Some(deformation) => {
          return deformation.clone()
      },
      _ => ()
    };
    //println!("calc deform: {} ", WorldManifold::calculate_deformation(magnitude));
    deform_memo.insert(magnitude, WorldManifold::calculate_deformation(magnitude));
    return deform_memo.get(&magnitude).unwrap().clone();
  }

  fn calculate_deformation(magnitude: int) -> Vec<Vec<f32>> {
    // Sets the "spread"
    let sigma: f32 = 2.0;
    let two_sigma_square: f32 = 18.0*(sigma*sigma);

    // Caps the range of the gaussian
    let maximum_range: uint = 8;

    return Vec::from_fn(maximum_range*2 + 1, |row| {
      Vec::from_fn( maximum_range*2 + 1, |column| {
        //println!("{},{}", column, row);
        let x: f32 =  ((maximum_range as f32) - (column as f32));
        let y: f32 =  ((maximum_range as f32) - (row as f32));
        //println!("      {},{}", x, y);

        let gaussian: f32 = (-(((x*x)/two_sigma_square) + ((y*y)/two_sigma_square))).exp();
        let field_strength: f32 = (magnitude as f32)*gaussian;

        field_strength
      })
    });
  }

  fn world_pos_to_field_pos(pos: Vector3<f32>) -> Vector3<uint> {
    let result = WorldManifold::world_pos_to_field_pos_2d(Vector2::new(pos.x, pos.y));
    Vector3::new(result.x, result.y, 0u)
  }

  fn world_pos_to_field_pos_2d(pos: Vector2<f32>) -> Vector2<uint> {
    let x: f32 = pos.x * 3.0 + 5000.0;
    let y: f32 = pos.y * 3.0 + 5000.0;

    // Keep us from going off the plane
    assert!(x > 0.0 && x < 10000.0);
    assert!(y > 0.0 && y < 10000.0);

    Vector2::new(x as uint, y as uint)
  }
}
