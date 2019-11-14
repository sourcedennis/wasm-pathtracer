use crate::math::{clamp};
use std::ops;

#[derive(Clone,Copy)]
pub struct Material {
  pub color : Color3
}

impl Material {
  pub fn new( color : Color3 ) -> Material {
    Material { color }
  }
}

#[derive(Clone,Copy)]
pub struct Color3 {
  pub red   : f64,
  pub green : f64,
  pub blue  : f64
}

impl Color3 {
  pub fn new( red : f64, green : f64, blue : f64 ) -> Color3 {
    Color3 { red, green, blue }
  }

  pub fn clamp( &self ) -> Color3 {
    let red   = clamp( self.red,   0.0_f64, 1.0_f64 );
    let green = clamp( self.green, 0.0_f64, 1.0_f64 );
    let blue  = clamp( self.blue,  0.0_f64, 1.0_f64 );
    Color3 { red, green, blue }
  }

  pub fn avg( colors : &Vec< Color3 > ) -> Color3 {
    let mut red   = 0.0_f64;
    let mut green = 0.0_f64;
    let mut blue  = 0.0_f64;

    for c in colors {
      red   += c.red;
      green += c.green;
      blue  += c.blue;
    }

    Color3::new( red   / colors.len( ) as f64
               , green / colors.len( ) as f64
               , blue  / colors.len( ) as f64
               )
  }
}

impl ops::Mul< f64 > for Color3 {
  type Output = Color3;

  fn mul( self, multiplier: f64 ) -> Color3 {
    Color3::new( multiplier * self.red, multiplier * self.green, multiplier * self.blue )
  }
}

impl ops::Mul< Color3 > for f64 {
  type Output = Color3;

  fn mul( self, v: Color3 ) -> Color3 {
    Color3::new( self * v.red, self * v.green, self * v.blue )
  }
}
