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

  pub fn at( self, distance : f64 ) -> Vec3 {
    self.origin + distance * self.dir
  }
}

#[derive(Clone,Copy)]
pub struct Hit {
  pub distance : f64,
  pub normal   : Vec3
}

impl Hit {
  pub fn new( distance : f64, normal : Vec3 ) -> Hit {
    Hit { distance, normal }
  }
}

#[derive(Clone,Copy)]
pub struct MatHit {
  pub hit : Hit,
  pub mat : Material
}

impl MatHit {
  pub fn new( hit : Hit, mat : Material ) -> MatHit {
    MatHit { hit, mat }
  }

  pub fn fromHit( hit : Option< Hit >, mat : Material ) -> Option< MatHit > {
    if let Some( h ) = hit {
      Some( MatHit::new( h, mat ) )
    } else {
      None
    }
  }
}
