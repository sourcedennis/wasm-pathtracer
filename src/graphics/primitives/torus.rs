use crate::math::{Vec2, Vec3};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Hit};

// A torus that lies flat
pub struct Torus {
  location : Vec3,
  big_r    : f32,
  small_r  : f32
}

impl Torus {
  pub fn new( location : Vec3, big_r : f32, small_r : f32 ) -> Torus {
    Torus { location, big_r, small_r }
  }
}

impl Tracable for Torus {
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    None // TODO
  }
}
