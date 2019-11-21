mod vec3;
mod ray;
mod scene;
mod math;
mod material;

use wasm_bindgen::prelude::*;
use vec3::Vec3;
use ray::{Ray, Hit};
use material::{Color3, Material};
use math::{clamp};
use std::ptr;
use scene::{Tracable, Light, Scene, Sphere, Plane, AABB};

// Z points INTO the screen. -Z points to the eye


static mut CONFIG : Option< Config > = None;

struct Config {
  viewport_width   : u32,
  viewport_height  : u32,
  aspect_ratio     : f32,
  is_depth         : bool,
  resultbuffer     : Vec< u8 >,
  rays             : Vec< ( u32, u32 ) >,
  scene            : Scene,
  num_indices_done : usize,
  max_reflect      : u32
}

#[wasm_bindgen]
pub fn init( width : u32, height : u32, is_depth : u32, max_reflect : u32 ) {
  unsafe {
    CONFIG = Some( Config {
      viewport_width:   width
    , viewport_height:  height
    , aspect_ratio:     width as f32 / height as f32
    , is_depth:         is_depth != 0
    , resultbuffer:     vec![0; (width*height*4) as usize]
    , rays:             vec![(0,0); (width*height) as usize]
    , scene:            setup_scene( )
    , num_indices_done: 0
    , max_reflect
    } );
  }
}

#[wasm_bindgen]
pub fn ray_store( ) -> *mut (u32, u32) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.rays.as_mut_ptr( )
    } else {
      panic!( "init not called" )
    }
  }
}

#[wasm_bindgen]
pub fn results( ) -> *const u8 {
  unsafe {
    if let Some( ref conf ) = CONFIG {
      conf.resultbuffer.as_ptr( )
    } else {
      panic!( "init not called" )
    }
  }
}

#[wasm_bindgen]
pub fn reset( ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.num_indices_done = 0;
    } else {
      panic!( "init not called" )
    }
  }
}

#[wasm_bindgen]
pub fn compute( count : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      let origin = Vec3::new( 0.0, 0.0, -2.0 );

      for _i in 0..count {
        let (x, y) = conf.rays[ conf.num_indices_done ];

        let x_f32 = ( ( ( x as f32 ) / ( ( conf.viewport_width - 1 ) as f32 ) ) - 0.5 ) * conf.aspect_ratio;
        let y_f32 = ( ( conf.viewport_height - y ) as f32 ) / ( ( conf.viewport_height - 1 ) as f32 ) - 0.5;
        let pixel = Vec3::new( x_f32, y_f32, 0.0 );
        let dir   = ( pixel - origin ).normalize( );

        let res =
          if conf.is_depth {
            trace_original_depth( &conf.scene, &Ray::new( origin, dir ) ).clamp( )
          } else {
            let (_,c) = trace_original_color( &conf.scene, &Ray::new( origin, dir ), conf.max_reflect );
            c.clamp( )
          };

        conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 0 ) as usize ] = ( 255.0 * res.red ) as u8;
        conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 1 ) as usize ] = ( 255.0 * res.green ) as u8;
        conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 2 ) as usize ] = ( 255.0 * res.blue ) as u8;
        conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 3 ) as usize ] = 255;

        conf.num_indices_done += 1;
      }
    } else {
      panic!( "init not called" )
    }
  }
}

pub fn setup_scene( ) -> Scene {
  let light = Light::new( Vec3::new( 0.0, 6.0, 4.5 ), Color3::new( 0.7, 0.7, 0.7 ) );

  // MatDiffuse { color : Color3 },
  // MatReflect { color : Color3, reflection : f32 },
  // MatRefract { reflection : f32, absorption : Color3, refractive_index : f32 }

  let mut shapes: Vec< Box< Tracable > > = Vec::new( );
  //Material::new( Color3::new( 1.0, 0.0, 0.0 ), 0.4, 0.3, 20.0, Some( Refraction::new( Color3::new( 0.7, 0.7, 0.7 ), 1.5 ) ) ) )
  shapes.push( Box::new( Sphere::new( Vec3::new(  0.0, 1.0, 5.0 ), 1.0, Material::refract( Vec3::new( 0.1, 0.2, 0.1 ), 1.5 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ) ) ) );
  shapes.push( Box::new( AABB::cube( Vec3::new(  -1.7, 0.0 + math::EPSILON * 2.0, 7.0 ), 1.0, Material::refract( Vec3::new( 0.1, 0.2, 0.2 ), 1.5 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );

  Scene::new( vec![ light ], shapes )
}

// Borrowed from:
// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/reflection-refraction-fresnel
fn refract( i : Vec3, mut n : Vec3, prev_ior : f32, ior : f32 ) -> Option< Vec3 > {
  let mut cosi = clamp( i.dot( n ), -1.0, 1.0 ); 
  let mut etai = prev_ior;
  let mut etat = ior; 
  //let mut n = N; 
  if cosi < 0.0 { cosi = -cosi; } else { swap( &mut etai, &mut etat ); n = -n; } 
  let eta = etai / etat; 
  let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
  if k < 0.0 {
    None
  } else {
    Some( ( eta * i + (eta * cosi - k.sqrt()) * n ).normalize( ) )
  }
}

fn swap< T: Copy >( a : &mut T, b : &mut T ) {
  let tmp = *a;
  *a = *b;
  *b = tmp;
}

// Borrowed from:
// https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/reflection-refraction-fresnel
fn fresnel( i : Vec3, n : Vec3, prev_ior : f32, ior : f32 ) -> f32 {
  let cosi = clamp( i.dot( n ), -1.0, 1.0 ); 
  let mut etai = prev_ior;
  let mut etat = ior; 
  if cosi > 0.0 {
    swap( &mut etai, &mut etat );
  } 
  // Compute sini using Snell's law
  let sint = etai / etat * 0.0_f32.max( 1.0 - cosi * cosi ).sqrt( ); 
  // Total internal reflection
  if sint >= 1.0 { 
    1.0
  } else { 
    let cost = 0.0_f32.max( 1.0 - sint * sint ).sqrt( );
    let cosi = cosi.abs( ); //fabsf(cosi); 
    let rs = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost)); 
    let rp = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost)); 
    (rs * rs + rp * rp) / 2.0
  } 
} 

fn trace_original_color( scene : &Scene, ray : &Ray, max_rays : u32 ) -> (f32, Color3) {
  if let Some( h ) = scene.trace( ray ) {
    let hit_loc = ray.at( h.distance );
    let lights  = scene.lights_at( &hit_loc );

    // Cumulative light color of all sources, scaled for their angle on the hit
    let mut light_color = Color3::BLACK; // Color3::new( 0.2, 0.2, 0.2 ); //Color3::BLACK;
    for l in &lights {
      light_color  = light_color + l.color * 0.0_f32.max( h.normal.dot( l.dir ) );
    }

    let color =
      match h.mat {
        Material::Diffuse { color } => light_color * color,
        Material::Reflect { color, reflection } => {
          if max_rays > 0 {
            let refl_dir     = (-ray.dir).reflect( h.normal );
            let refl_ray     = Ray::new( hit_loc + math::EPSILON * refl_dir, refl_dir );
            let (_, refl_diffuse) = trace_original_color( scene, &refl_ray, max_rays - 1 );
            let mut diffuse_color = Color3::BLACK;
            if reflection > 0.0 {
              diffuse_color = diffuse_color + reflection * refl_diffuse;
            }
            if reflection < 1.0 {
              diffuse_color = diffuse_color + ( 1.0 - reflection ) * color;
            }
            light_color * diffuse_color
          } else { // If it's at the cap, just apply direct illumination
            light_color * color
          }
        },
        Material::Refract { absorption, refractive_index } => {
          let air_refraction = 1.0; // TODO: What if a glass object is inside water?
          let mut kr = fresnel( ray.dir, h.normal, air_refraction, refractive_index );

          let refr_color =
            if max_rays > 0 {
              if let Some( refr_dir ) = refract( ray.dir, h.normal, air_refraction, refractive_index ) {
                // No total internal reflection (refract(..) returns None if that were so). So kr < 1.0
                // Cast refraction ray
                let refr_ray = Ray::new( hit_loc + refr_dir * math::EPSILON, refr_dir );
                let (d,c) = trace_original_color( scene, &refr_ray, max_rays - 1 );
                c * ( -absorption * d ).exp( ) // Beer's Law
              } else {
                Color3::BLACK
              }
            } else {
              kr = 1.0; // Assume full reflection
              Color3::BLACK
            };

          let refl_color =
            if max_rays > 0 && kr > 0.0 {
              let refl_dir = (-ray.dir).reflect( h.normal ); // TODO: Remove normalize()
              let refl_ray = Ray::new( hit_loc + refl_dir * math::EPSILON, refl_dir );
              let (_, c) = trace_original_color( scene, &refl_ray, max_rays - 1 );
              c
            } else {
              kr = 1.0; // Assume full reflection
              Color3::BLACK // And paint it black
            };

          refl_color * kr + refr_color * ( 1.0 - kr )
        }
      };

    ( h.distance, color )
  } else {
    ( 0.0, Color3::BLACK )
  }
}

fn trace_original_depth( scene : &Scene, ray : &Ray ) -> Color3 {
  if let Some( h ) = scene.trace( ray ) {
    let v = 1.0 - clamp( ( h.distance - 5.0 ) / 12.0, 0.0, 1.0 );
    Color3::new( v, v, v )
  } else {
    Color3::new( 0.0, 0.0, 0.0 )
  }
}
