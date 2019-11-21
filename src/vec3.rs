
use std::ops;

#[derive(Copy,Clone)]
pub struct Vec3 {
  pub x : f32,
  pub y : f32,
  pub z : f32
}

impl Vec3 {
  pub fn new( x : f32, y : f32, z : f32 ) -> Vec3 {
    Vec3 { x, y, z }
  }

  pub fn normalize( self ) -> Vec3 {
    let lenSq = self.dot( self );
    let len = lenSq.sqrt( );
    Vec3::new( self.x / len, self.y / len, self.z / len )
  }

  pub fn dot( self, rhs : Vec3 ) -> f32 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
  }

  pub fn len( self ) -> f32 {
    self.len_sq( ).sqrt( )
  }

  pub fn len_sq( self ) -> f32 {
    self.dot( self )
  }

  pub fn reflect( self, normal : Vec3 ) -> Vec3 {
    2.0 * self.dot( normal ) * normal - self
  }

  pub fn exp( self ) -> Vec3 {
    Vec3::new( self.x.exp( ), self.y.exp( ), self.z.exp( ) )
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

impl ops::Mul< f32 > for Vec3 {
  type Output = Vec3;

  fn mul( self, multiplier: f32 ) -> Vec3 {
    Vec3::new( multiplier * self.x, multiplier * self.y, multiplier * self.z )
  }
}

impl ops::Mul< Vec3 > for f32 {
  type Output = Vec3;

  fn mul( self, v: Vec3 ) -> Vec3 {
    Vec3::new( self * v.x, self * v.y, self * v.z )
  }
}

impl ops::Div< f32 > for Vec3 {
  type Output = Vec3;

  fn div( self, divisor: f32 ) -> Vec3 {
    Vec3::new( self.x / divisor, self.y / divisor, self.z / divisor )
  }
}
