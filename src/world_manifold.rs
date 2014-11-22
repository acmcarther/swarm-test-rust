extern crate cgmath;

use cgmath::Vector3;

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

  pub fn deform(&mut self, pos: Vector3<f32>, magnitude: f32, diameter: f32) -> () {
    // TODO: Something. At all.
    // Probably apply a gaussian deformation onto a 2d array
    //println!("DEFORM");
  }

  pub fn gradient_at(&self, pos: Vector3<f32>) -> Vector3<f32> {
    // TODO: When we get a model, get a gradient
    return Vector3::new(0.0,0.0,0.0);
  }
}
