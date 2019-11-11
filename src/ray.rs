use crate::vec3::Vec3;

pub struct Ray {
  pub origin : Vec3,
  pub dir    : Vec3
}

impl Ray {
  pub fn new( origin : Vec3, dir : Vec3 ) -> Ray {
    Ray { origin, dir }
  }

  pub fn at( self, distance : f64 ) -> Vec3 {
    self.origin + distance * self.dir
  }
}

pub struct Hit {
  pub distance : f64,
  pub normal   : Vec3
}

impl Hit {
  pub fn new( distance : f64, normal : Vec3 ) -> Hit {
    Hit { distance, normal }
  }
}
