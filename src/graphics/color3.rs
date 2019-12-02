use crate::math::{clamp, Vec3};
use std::ops;

/// A floating point Color class with operations
/// Note that Color3's and Vec3's are semantically different
///   (though both contain 3 f32's each)
/// To avoid accidental errors, let the type system enforce their
///   distinction. (Which causes some duplicate code, sadly)
/// Also, color channel values are *always* within the range [0-1]
///   which sets them apart from Vec3's
#[derive(Clone,Copy)]
pub struct Color3 {
  pub red   : f32,
  pub green : f32,
  pub blue  : f32
}

impl Color3 {
  pub const BLACK: Color3 = Color3 { red: 0.0, green: 0.0, blue: 0.0 };

  /// Constructs a new Color3
  ///
  /// All channels are clamped within the range [0-1]
  ///   (as colors outside this range are semantically invalid)
  pub fn new( red : f32, green : f32, blue : f32 ) -> Color3 {
    let c_red   = clamp( red,   0.0_f32, 1.0_f32 );
    let c_green = clamp( green, 0.0_f32, 1.0_f32 );
    let c_blue  = clamp( blue,  0.0_f32, 1.0_f32 );

    ( Color3 { red: c_red, green: c_green, blue: c_blue } )
  }

  // Converts the (r,g,b) channels to a (x,y,z) vector
  // This is convienient when clamped values are undesired
  pub fn to_vec3( self ) -> Vec3 {
    Vec3::new( self.red, self.green, self.blue )
  }
}

/// Multiply a color by a constant: Color3 * f32 = Color3
impl ops::Mul< f32 > for Color3 {
  type Output = Color3;

  fn mul( self, multiplier: f32 ) -> Color3 {
    Color3::new( multiplier * self.red, multiplier * self.green, multiplier * self.blue )
  }
}

/// Multiply a color by a constant: f32 * Color3 = Color3
impl ops::Mul< Color3 > for f32 {
  type Output = Color3;

  fn mul( self, v: Color3 ) -> Color3 {
    Color3::new( self * v.red, self * v.green, self * v.blue )
  }
}

/// Elementwise multiplication of a color and a Vec3: Color3 * Vec3 = Color3
impl ops::Mul< Vec3 > for Color3 {
  type Output = Color3;

  fn mul( self, v: Vec3 ) -> Color3 {
    Color3::new( self.red * v.x, self.green * v.y, self.blue * v.z )
  }
}

/// Elementwise multiplication of a color and a Vec3: Vec3 * Color3 = Color3
impl ops::Mul< Color3 > for Vec3 {
  type Output = Color3;

  fn mul( self, c : Color3 ) -> Color3 {
    Color3::new( self.x * c.red, self.y * c.green, self.z * c.blue )
  }
}

/// Addition of the channels of two Color3's
impl ops::Add< Color3 > for Color3 {
  type Output = Color3;

  fn add( self, v: Color3 ) -> Color3 {
    Color3::new( self.red + v.red, self.green + v.green, self.blue + v.blue )
  }
}

impl ops::AddAssign< Color3 > for Color3 {
  fn add_assign( &mut self, v : Color3 ) {
    self.red   = clamp( self.red   + v.red,   0.0_f32, 1.0_f32 );
    self.green = clamp( self.green + v.green, 0.0_f32, 1.0_f32 );
    self.blue  = clamp( self.blue  + v.blue,  0.0_f32, 1.0_f32 );
  }
}

// impl ops::Mul< Color3 > for Color3 {
//   type Output = Color3;

//   fn mul( self, v: Color3 ) -> Color3 {
//     Color3::new( self.red * v.red, self.green * v.green, self.blue * v.blue )
//   }
// }
