//extern crate rand;

mod vec3;
mod ray;
mod primitives;

use wasm_bindgen::prelude::*;
use vec3::Vec3;
use ray::{Ray, Hit};
use primitives::sphere::{hit_sphere};

// Z points INTO the screen. -Z points to the eye

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

static mut VIEWPORT_WIDTH  : u32 = 0;
static mut VIEWPORT_HEIGHT : u32 = 0;

#[wasm_bindgen]
pub fn init( width: u32, height: u32 ) {
  unsafe {
    VIEWPORT_WIDTH  = width;
    VIEWPORT_HEIGHT = height;
  }
}

#[wasm_bindgen]
pub fn compute( vp_x: u32, vp_y: u32, width: u32, height: u32 ) -> Vec< u8 > {
  let mut v = Vec::with_capacity( ( width * height * 3 ) as usize );
  for r in rays( vp_x, vp_y, width, height ) {
    let color = trace_ray_color( r ).clamp( );

    v.push( ( color.red * 255.0 ) as u8 );
    v.push( ( color.green * 255.0 ) as u8 );
    v.push( ( color.blue * 255.0 ) as u8 );
  }
  v
}

#[wasm_bindgen]
pub fn compute_depths( vp_x: u32, vp_y: u32, width: u32, height: u32 ) -> Vec< u8 > {
  let mut v = Vec::with_capacity( ( width * height * 3 ) as usize );
  for r in rays( vp_x, vp_y, width, height ) {
    let c_val =
      if let Some( h ) = trace_ray_depth( r ) {
        ( clamp( 1.0 - ( h - 3.0 ) / 4.0, 0.0, 1.0 ) * 255.0 ) as u8
      } else {
        0
      };
    v.push( c_val ); v.push( c_val ); v.push( c_val );
  }
  v
}

fn rays( vp_x: u32, vp_y: u32, width: u32, height: u32 ) -> Vec< Ray > {
  unsafe {
    let mut rays = Vec::with_capacity( ( width * height ) as usize );
    let ar = VIEWPORT_WIDTH as f64 / VIEWPORT_HEIGHT as f64;
    for y in 0..height {
      for x in 0..width {
        let origin = Vec3::new( 0.0, 0.0, -2.0 );
        let x_f64  = ( ( ( vp_x + x ) as f64 / ( ( VIEWPORT_WIDTH - 1 ) as f64 ) ) - 0.5 ) * ar;
        let y_f64  = ( vp_y + y ) as f64 / ( ( VIEWPORT_HEIGHT - 1 ) as f64 ) - 0.5;
        let pixel  = Vec3::new( x_f64, y_f64, 0.0 );
        let dir    = ( pixel - origin ).normalize( );
        rays.push( Ray::new( origin, dir ) );
      }
    }
    rays
  }
}

fn trace_ray_color( ray : Ray ) -> Color3 {
  if let Some( _h ) = trace_ray( ray ) {
    Color3::new( 1.0, 0.0, 0.0 )
  } else {
    Color3::new( 0.0, 0.0, 0.0 )
  }
}

fn trace_ray_depth( ray : Ray ) -> Option< f64 > {
  if let Some( h ) = trace_ray( ray ) {
    Some( h.distance )
  } else {
    None
  }
}

fn trace_ray( ray : Ray ) -> Option< Hit > {
  hit_sphere( Vec3::new( 0.0, 0.0, 5.0 ), 1.0, ray )
}

struct Color3 {
  red   : f64,
  green : f64,
  blue  : f64
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
}

fn clamp( x : f64, min_val : f64, max_val : f64 ) -> f64 {
  max_val.min( min_val.max( x ) )
}
