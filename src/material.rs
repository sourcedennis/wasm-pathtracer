use crate::math::{clamp};
use crate::vec3::{Vec3};
use std::ops;

// #[derive(Clone,Copy)]
// pub struct Material {
//   pub color      : Color3,
//   // Reflection. 0 is none. 1 is full
//   pub reflection : f32,
//   // Shininess for specular highlight. >= 0
//   pub specular   : f32,
//   pub shininess  : f32,
//   pub refraction : Option< Refraction >
// }

#[derive(Clone,Copy)]
pub enum Material {
  Diffuse { color : Color3 },
  Reflect { color : Color3, reflection : f32 },
  Refract { absorption : Vec3, refractive_index : f32 }
}

impl Material {
  pub fn diffuse( color : Color3 ) -> Material {
    Material::Diffuse { color }
  }

  pub fn reflect( color : Color3, reflection : f32 ) -> Material {
    Material::Reflect { color, reflection }
  }

  pub fn refract( absorption : Vec3, refractive_index : f32 ) -> Material {
    Material::Refract { absorption, refractive_index }
  }
}

#[derive(Clone,Copy)]
pub struct Color3 {
  pub red   : f32,
  pub green : f32,
  pub blue  : f32
}

impl Color3 {
  pub const BLACK: Color3 = Color3 { red: 0.0, green: 0.0, blue: 0.0 };

  pub fn new( red : f32, green : f32, blue : f32 ) -> Color3 {
    ( Color3 { red, green, blue } ).clamp( )
  }

  pub fn clamp( &self ) -> Color3 {
    let red   = clamp( self.red,   0.0_f32, 1.0_f32 );
    let green = clamp( self.green, 0.0_f32, 1.0_f32 );
    let blue  = clamp( self.blue,  0.0_f32, 1.0_f32 );
    Color3 { red, green, blue }
  }

  pub fn avg( colors : &Vec< Color3 > ) -> Color3 {
    let mut red   = 0.0_f32;
    let mut green = 0.0_f32;
    let mut blue  = 0.0_f32;

    for c in colors {
      red   += c.red;
      green += c.green;
      blue  += c.blue;
    }

    Color3::new( red   / colors.len( ) as f32
               , green / colors.len( ) as f32
               , blue  / colors.len( ) as f32
               )
  }
}

impl ops::Mul< f32 > for Color3 {
  type Output = Color3;

  fn mul( self, multiplier: f32 ) -> Color3 {
    Color3::new( multiplier * self.red, multiplier * self.green, multiplier * self.blue )
  }
}

impl ops::Mul< Color3 > for f32 {
  type Output = Color3;

  fn mul( self, v: Color3 ) -> Color3 {
    Color3::new( self * v.red, self * v.green, self * v.blue )
  }
}

impl ops::Mul< Vec3 > for Color3 {
  type Output = Color3;

  fn mul( self, v: Vec3 ) -> Color3 {
    Color3::new( self.red * v.x, self.green * v.y, self.blue * v.z )
  }
}

impl ops::Mul< Color3 > for Vec3 {
  type Output = Color3;

  fn mul( self, c : Color3 ) -> Color3 {
    Color3::new( self.x * c.red, self.y * c.green, self.z * c.blue )
  }
}

impl ops::Add< Color3 > for Color3 {
  type Output = Color3;

  fn add( self, v: Color3 ) -> Color3 {
    Color3::new( self.red + v.red, self.green + v.green, self.blue + v.blue )
  }
}

impl ops::Mul< Color3 > for Color3 {
  type Output = Color3;

  fn mul( self, v: Color3 ) -> Color3 {
    Color3::new( self.red * v.red, self.green * v.green, self.blue * v.blue )
  }
}
