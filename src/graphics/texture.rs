// External imports
use std::fmt;
// Local imports
use crate::graphics::Color3;
use crate::math::Vec2;

/// A RGB texture
#[derive(Clone)]
pub struct Texture {
  pub data   : Vec< (u8,u8,u8) >,
  pub width  : u32,
  pub height : u32
}

impl Texture {
  /// Creates a black RGB texture of the provided size
  pub fn new( width : u32, height : u32 ) -> Texture {
    Texture { width, height, data: vec![(0,0,0); (width * height) as usize] }
  }

  /// Evaluates the texture at the given location in (0,1)x(0,1)
  ///   any value outside that range wraps around to the start again
  pub fn at( &self, v : Vec2 ) -> Color3 {
    let ix = modulo( ( v.x * self.width as f32 ).floor( ) as u32, self.width );
    let iy = modulo( ( v.y * self.height as f32 ).floor( ) as u32, self.height );
    let (r,g,b) = self.data[ ( iy * self.width + ix ) as usize ];
    Color3::new( r as f32 / 255_f32
               , g as f32 / 255_f32
               , b as f32 / 255_f32
               )
  }
}

/// Performs mathematically correct module on `u32`s.
/// Note that this differs from the available "remainder" operator in Rust.
fn modulo( a : u32, m : u32 ) -> u32 {
  ( ( a % m ) + m ) % m
}


impl fmt::Debug for Texture {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!( f, "Texture {{ width: {}, height: {} }}", self.width, self.height )
  }
}
