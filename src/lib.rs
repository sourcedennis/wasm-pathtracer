//extern crate rand;

mod vec3;
mod ray;
mod primitives;
mod math;
mod material;

use wasm_bindgen::prelude::*;
use vec3::Vec3;
use ray::{Ray, Hit, MatHit};
use primitives::sphere::{hit_sphere};
use primitives::plane::{hit_plane};
use material::{Color3, Material};
use math::{clamp};

// Z points INTO the screen. -Z points to the eye

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

static mut VIEWPORT_WIDTH  : u32 = 0;
static mut VIEWPORT_HEIGHT : u32 = 0;

static EPSILON : f64 = 0.00001;

#[wasm_bindgen]
pub fn init( width: u32, height: u32 ) {
  unsafe {
    VIEWPORT_WIDTH  = width;
    VIEWPORT_HEIGHT = height;
  }
}

#[wasm_bindgen]
pub fn compute( vp_x: u32, vp_y: u32, width: u32, height: u32, anti_alias: u32 ) -> Vec< u8 > {
  let mut v = Vec::with_capacity( ( width * height * 3 ) as usize );
  /*for bundle in rays( vp_x, vp_y, width, height, anti_alias ) {
    let mut colors = Vec::new( );
    for r in bundle {
      colors.push( trace_ray_color( r ).clamp( ) );
    }
    let avg_color = Color3::avg( &colors );

    v.push( ( avg_color.red * 255.0 ) as u8 );
    v.push( ( avg_color.green * 255.0 ) as u8 );
    v.push( ( avg_color.blue * 255.0 ) as u8 );
  }*/
  v
}

#[wasm_bindgen]
pub fn compute_depths( vp_x: u32, vp_y: u32, width: u32, height: u32, anti_alias: u32 ) -> Vec< u8 > {
  let mut v = Vec::with_capacity( ( width * height * 3 ) as usize );
  for bundle in rays( vp_x, vp_y, width, height, anti_alias ) {
    let mut sum = 0.0_f64;
    for r in &bundle {
      let c_val =
        if let Some( h ) = trace_ray_depth( *r ) {
          clamp( 1.0 - ( h - 3.0 ) / 4.0, 0.0, 1.0 )
        } else {
          0.0
        };
      sum += c_val;
    }
    let c_val = ( ( sum / bundle.len( ) as f64 ) * 255.0 ) as u8;
    v.push( c_val ); v.push( c_val ); v.push( c_val );
  }
  v
}

fn rays( vp_x: u32, vp_y: u32, width: u32, height: u32, anti_alias : u32 ) -> Vec< Vec< Ray > > {
  unsafe {
    let mut rays = Vec::with_capacity( ( width * height ) as usize );
    let ar = VIEWPORT_WIDTH as f64 / VIEWPORT_HEIGHT as f64;
    let alias_offsets = ray_alias( anti_alias );
    for y in 0..height {
      for x in 0..width {
        let mut bundle = Vec::new( );
        for off in &alias_offsets {
          let off_x = off.0;
          let off_y = off.1;

          let origin = Vec3::new( 0.0, 0.0, -2.0 );
          let x_f64  = ( ( ( ( vp_x + x ) as f64 + off_x ) / ( ( VIEWPORT_WIDTH - 1 ) as f64 ) ) - 0.5 ) * ar;
          let y_f64  = ( ( VIEWPORT_HEIGHT - ( vp_y + y ) ) as f64 + off_y ) / ( ( VIEWPORT_HEIGHT - 1 ) as f64 ) - 0.5;
          let pixel  = Vec3::new( x_f64, y_f64, 0.0 );
          let dir    = ( pixel - origin ).normalize( );
          bundle.push( Ray::new( origin, dir ) );
        }
        rays.push( bundle );
      }
    }
    rays
  }
}

fn ray_alias( count : u32 ) -> Vec< ( f64, f64 ) > {
  let step    = 1.0 / count as f64;
  let first   = -0.5 + step * 0.5;
  let mut dst = Vec::new( );

  for y in 0..count {
    for x in 0..count {
      dst.push( ( first + step * x as f64, first + step * y as f64 ) );
    }
  }
  // vec![ (0.25,0.25), (0.25,-0.25), (-0.25,-0.25), (-0.25,0.25) ]
  dst
}

fn trace_ray_color( ray : Ray ) -> Color3 {
  let light_loc = Vec3::new( 0.0, 2.0, 1.5 );
  let light_color = 1.0;

  if let Some( h ) = trace_ray( ray ) {
    let hit_loc  = ray.at( h.hit.distance );
    let to_light = ( light_loc - hit_loc ).normalize( );

    let shadow_ray = Ray::new( hit_loc + EPSILON * to_light, to_light );
    if is_hit_within_sq( trace_ray( shadow_ray ), ( light_loc - hit_loc ).len_sq( ) ) {
      Color3::new( 0.0, 0.0, 0.0 )
    } else {
      // Immediate ray to light source
      h.mat.color * light_color * 0.0_f64.max( h.hit.normal.dot( to_light ) )
    }
  } else {
    Color3::new( 0.0, 0.0, 0.0 )
  }
}

fn is_hit_within_sq( m_hit : Option< MatHit >, d_sq : f64 ) -> bool {
  if let Some( h ) = m_hit {
    h.hit.distance * h.hit.distance < d_sq
  } else {
    false
  }
}

fn trace_ray_depth( ray : Ray ) -> Option< f64 > {
  if let Some( h ) = trace_ray( ray ) {
    Some( h.hit.distance )
  } else {
    None
  }
}

fn trace_ray( ray : Ray ) -> Option< MatHit > {
  let h1 = hit_sphere( Vec3::new(  0.0, 0.0, 5.0 ), 1.0, ray );
  let h2 = hit_sphere( Vec3::new( -1.0, 0.0, 5.0 ), 1.0, ray );
  let h3 = hit_sphere( Vec3::new(  1.0, 0.0, 5.0 ), 1.0, ray );
  let plane = hit_plane( Vec3::new( 0.0, -1.0, 5.0 ), Vec3::new( 0.0, 1.0, 0.0 ), ray );

  best_hit(
    vec![ MatHit::fromHit( h1, Material::new( Color3::new( 1.0, 0.0, 0.0 ) ) )
        , MatHit::fromHit( h2, Material::new( Color3::new( 0.0, 1.0, 0.0 ) ) )
        , MatHit::fromHit( h3, Material::new( Color3::new( 0.0, 0.0, 1.0 ) ) )
        , MatHit::fromHit( plane, Material::new( Color3::new( 0.6, 0.6, 0.6 ) ) )
        ]
  )
}

fn best_hit( hits : Vec< Option< MatHit > > ) -> Option< MatHit > {
  let mut best: Option< MatHit > = None;

  for h in hits {
    if let Some( bh ) = best {
      if let Some( new_h ) = h {
        if new_h.hit.distance < bh.hit.distance {
          best = h;
        }
      }
    } else {
      best = h;
    }
  }

  best
}
