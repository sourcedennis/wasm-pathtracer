use std::ops;
use std::fmt;

/// A vector in 3-dimensional space
#[derive(Copy,Clone)]
pub struct Vec3 {
  pub x : f32,
  pub y : f32,
  pub z : f32
}

impl Vec3 {
  /// The vector that lies at the origin, which has 0 length; (0,0,0)
  pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };

  /// Constructs a new vector with the provided components
  pub fn new( x : f32, y : f32, z : f32 ) -> Vec3 {
    Vec3 { x, y, z }
  }

  /// Constructs a new unit vector in the provided direction
  pub fn unit( x : f32, y : f32, z : f32 ) -> Vec3 {
    Vec3::new( x, y, z ).normalize( )
  }

  /// Scales the vector such that its length becomes 1
  pub fn normalize( self ) -> Vec3 {
    self * ( 1.0 / self.len( ) )
  }

  /// Computes the dot product with the provided Vec3
  pub fn dot( self, rhs : Vec3 ) -> f32 {
    self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
  }

  /// Computes the crosss product with the provided Vec3
  pub fn cross( self, t : Vec3 ) -> Vec3 {
    Vec3::new(
      self.y * t.z - self.z * t.y,
      self.z * t.x - self.x * t.z,
      self.x * t.y - self.y * t.x )
  }

  /// Returns the length
  pub fn len( self ) -> f32 {
    self.len_sq( ).sqrt( )
  }

  /// Returns the *square* length
  pub fn len_sq( self ) -> f32 {
    self.dot( self )
  }

  /// Reflects the vector along the provided normal
  pub fn reflect( self, normal : Vec3 ) -> Vec3 {
    2.0 * self.dot( normal ) * normal - self
  }

  /// Applies every component as the power of `e`
  /// So, it returns: (e^x, e^y, e^z)
  pub fn exp( self ) -> Vec3 {
    Vec3::new( self.x.exp( ), self.y.exp( ), self.z.exp( ) )
  }

  pub fn rot_y( &self, angle : f32 ) -> Vec3 {
    // [  c 0 s ] [x]
    // [  0 1 0 ] [y]
    // [ -s 0 c ] [z]
    let x = self.x;
    let y = self.y;
    let z = self.z;

    let c = angle.cos( );
    let s = angle.sin( );
    Vec3::new( c * x + s * z, y, -s * x + c * z )
  }

  pub fn rot_x( &self, angle : f32 ) -> Vec3 {
    // [ 1 0  0 ] [x]
    // [ 0 c -s ] [y]
    // [ 0 s  c ] [z]
    let x = self.x;
    let y = self.y;
    let z = self.z;

    let c = angle.cos( );
    let s = angle.sin( );
    Vec3::new( x, c * y - s * z, s * y + c * z )
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

impl ops::AddAssign< Vec3 > for Vec3 {
  fn add_assign( &mut self, v : Vec3 ) {
    self.x += v.x;
    self.y += v.y;
    self.z += v.z;
  }
}

impl fmt::Debug for Vec3 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!( f, "vec3({}, {}, {})", self.x, self.y, self.z )
  }
}
          