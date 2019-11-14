
use std::ops;

#[derive(Copy,Clone)]
pub struct Vec3 {
  pub x : f64,
  pub y : f64,
  pub z : f64
}

impl Vec3 {
  pub fn new( x : f64, y : f64, z : f64 ) -> Vec3 {
    Vec3 { x, y, z }
  }

  pub fn normalize( self ) -> Vec3 {
    let lenSq = self.dot( self );
    let len = lenSq.sqrt( );
    Vec3::new( self.x / len, self.y / len, self.z / len )
  }

  pub fn dot( self, rhs : Vec3 ) -> f64 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
  }

  pub fn len( self ) -> f64 {
    self.len_sq( ).sqrt( )
  }

  pub fn len_sq( self ) -> f64 {
    self.dot( self )
  }
}

impl ops::Neg for Vec3 {
  type Output = Vec3;

  fn neg( self ) -> Vec3 {
    Vec3::new( -self.x, -self.y, -self.z )
  }
}

impl ops::Add< Vec3 > for Vec3 {
  type Output = Vec3;

  fn add( self, addend: Vec3 ) -> Vec3 {
    Vec3::new( self.x + addend.x, self.y + addend.y, self.z + addend.z )
  }
}

impl ops::Sub< Vec3 > for Vec3 {
  type Output = Vec3;

  fn sub( self, subtrahend: Vec3 ) -> Vec3 {
    Vec3::new( self.x - subtrahend.x, self.y - subtrahend.y, self.z - subtrahend.z )
  }
}

impl ops::Mul< f64 > for Vec3 {
  type Output = Vec3;

  fn mul( self, multiplier: f64 ) -> Vec3 {
    Vec3::new( multiplier * self.x, multiplier * self.y, multiplier * self.z )
  }
}

impl ops::Mul< Vec3 > for f64 {
  type Output = Vec3;

  fn mul( self, v: Vec3 ) -> Vec3 {
    Vec3::new( self * v.x, self * v.y, self * v.z )
  }
}

impl ops::Div< f64 > for Vec3 {
  type Output = Vec3;

  fn div( self, divisor: f64 ) -> Vec3 {
    Vec3::new( self.x / divisor, self.y / divisor, self.z / divisor )
  }
}
