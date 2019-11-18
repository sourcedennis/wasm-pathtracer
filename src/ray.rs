use crate::vec3::Vec3;
use crate::material::{Material};

#[derive(Clone,Copy)]
pub struct Ray {
  pub origin : Vec3,
  pub dir    : Vec3
}

impl Ray {
  pub fn new( origin : Vec3, dir : Vec3 ) -> Ray {
    Ray { origin, dir }
  }

  pub fn at( self, distance : f32 ) -> Vec3 {
    self.origin + distance * self.dir
  }
}

#[derive(Clone,Copy)]
pub struct Hit {
  pub distance    : f32,
  pub normal      : Vec3,
  pub mat         : Material,
  pub is_entering : bool
}

impl Hit {
  pub fn new( distance : f32, normal : Vec3, mat : Material, is_entering : bool ) -> Hit {
    Hit { distance, normal, mat, is_entering }
  }
}
