
// External imports
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
// Local imports
use crate::data::cap_stack::{Stack};
use crate::graphics::scene::{Scene};
use crate::graphics::ray::{Ray};
use crate::graphics::primitives::{AARect,Plane,Sphere,Triangle};
use crate::math::vec3::{Vec3};
use crate::scenes::{setup_scene, setup_ball_scene, setup_scene_ballsphere};
use crate::tracer::{MatRefract, Camera, trace_original_color, trace_original_depth};

// TODO
struct Mesh { }

// This file contains all the functions that are exposed through WebAssembly
// Interfacing with JavaScript is a bit annoying, as only primitives (i32, i64, f32, f64)
// can be passed across the "boundary".
// I purposefully avoid "bridging" JavaScript code that is generated by wasm-pack,
// because I'm unsure about performance penalties this may incur. So I write "simple bridges"
// with only primitives.

// The intuition about the tracing work is as follows:
// * This instance is initialised with session information (viewport, camera, ray depth, etc.)
// * This instance is *assigned* (by JavaScript) the pixels for which it should trace rays
//     (Thus JavaScript can spawn multiple webworkers - each with their own rays to compute)
// * The `compute` method is called, which traces the rays for all assigned pixels
//
// General notes:
// * Z points INTO the screen. -Z points to the eye


/// The state of a rendering session
///   (Sessions change upon framebuffer resize)
struct Config {
  // ## Global State
  meshes          : HashMap< u32, Mesh >,

  // ## Session State
  viewport_width  : u32,
  viewport_height : u32,
  // True if rendering a depth buffer. Otherwise a color buffer
  is_depth        : bool,
  resultbuffer    : Vec< u8 >,
  // Pixels that are handled by the renderer
  pixel_coords    : Vec< ( u32, u32 ) >,
  // Cached original rays
  rays            : Vec< Ray >,
  num_rays        : u32,
  scene           : Scene,
  max_ray_depth   : u32,
  camera          : Camera,

  // ## Preallocation Stuff
  //      (avoids dynamic allocation)
  mat_stack       : Stack< RefractMat >,
}

/// This is global state, which it must be. WASM is called through
///   JS which owns the (global) state. Consider this whole WASM
///   module as a single encapsulated entity, with its own state.
static mut CONFIG : Option< Config > = None;

/// Initialises the *Session State*.
#[wasm_bindgen]
pub fn init( width : u32, height : u32, scene_id : u32, is_depth : u32, max_ray_depth : u32
           , cam_x : f32, cam_y : f32, cam_z : f32, cam_rot_x : f32, cam_rot_y : f32 ) {
  unsafe {
    // Preserve global state
    let prev_meshes =
      if let Some( conf ) = CONFIG {
        conf.meshes
      } else {
        HashMap::new( )
      };

    CONFIG = Some( Config {
      // ## Global State
      meshes:           prev_meshes

      // ## Session State
    , viewport_width:   width
    , viewport_height:  height
    , is_depth:         is_depth != 0
    , resultbuffer:      vec![0; (width*height*4) as usize]
      // Note that the actual pixels for this "thread" are distributed by JavaScript
    , pixel_coords:     vec![(0,0); (width*height) as usize]
    , rays:             vec![Ray::new( Vec3::ZERO, Vec3::ZERO ); (width*height) as usize]
    , num_rays:         0
    , scene:            select_scene( scene_id )
    , max_ray_depth
    , camera:           Camera::new( Vec3::new( cam_x, cam_y, cam_z ), cam_rot_x, cam_rot_y )

      // ## Preallocation Stuff
    , mat_stack:        Stack::new1( ( max_ray_depth + 1 ) as usize, RefractMat::AIR )
    } );
  }
}

/// Returns a pointer to the resulting buffer
/// This buffer is always of size `viewport_width * viewport_height`, but only the assigned
///   pixels are filled in.
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

// Called by JavaScript to obtain a pointer to the block in which the ray pixels are stored
// JavaScript fills it with the pixel locations that the raytracer should execute
//   (Such that pixels can be distributed over different workers running this raytracer)
#[wasm_bindgen]
pub fn ray_store( num_rays : u32 ) -> *mut (u32, u32) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.num_rays = num_rays;
      conf.pixel_coords.as_mut_ptr( )
    } else {
      panic!( "init not called" )
    }
  }
}

// After JavaScript has assigned the pixels, the original rays are *precomputed*
// This improves performance when the camera does not move
#[wasm_bindgen]
pub fn ray_store_done( ) {
  unsafe {
    if let Some( ref conf ) = CONFIG {
      let origin = conf.camera.location;

      // For the camera:
      // - First rotate each direction around the x-axis
      // - Then rotate each direction around the y-axis
      // - Then translate the origin

      if let Some( ref mut conf ) = CONFIG {
        let uw = conf.viewport_width as usize;
        let uh = conf.viewport_height as usize;

        for i in 0..(conf.num_rays as usize) {
          let (x,y) = conf.pixel_coords[ i ];

          let w_inv = 1.0 / conf.viewport_width as f32;
          let h_inv = 1.0 / conf.viewport_height as f32;
          let ar = conf.viewport_width as f32 / conf.viewport_height as f32;

          let fx = ( ( x as f32 + 0.5_f32 ) * w_inv - 0.5_f32 ) * ar;
          let fy = 0.5_f32 - ( y as f32 + 0.5_f32 ) * h_inv;

          let pixel = Vec3::new( fx, fy, 1.0 );
          let dir   = pixel.normalize( ).rot_x( conf.camera.rot_x ).rot_y( conf.camera.rot_y );

          conf.rays[ i ].origin = origin;
          conf.rays[ i ].dir = dir;
        }
      }
    } else {
      panic!( "init not called" )
    }
  }
}

/// Updates the rendering session with new parameters
/// Other aspects of the session remain the same
#[wasm_bindgen]
pub fn update_params( is_depth : u32, max_ray_depth : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.is_depth      = ( is_depth != 0 );
      conf.max_ray_depth = max_ray_depth;
      conf.mat_stack     = Stack::new1( ( max_ray_depth + 1 ) as usize, RefractMat::AIR );
    } else {
      panic!( "init not called" )
    }
  }
}

/// Updates the rendered scene
/// Other aspects of the session remain the same
#[wasm_bindgen]
pub fn update_scene( scene_id : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.scene =
        match scene_id {
          0 => setup_scene( ),
          1 => setup_ball_scene( ),
          2 => setup_scene_ballsphere( ),
          _ => setup_scene( ) // Should not happen
        };
    } else {
      panic!( "init not called" )
    }
  }
}

/// Updates the camera in the session
/// Other aspects of the session remain the same
/// Note that the camera first rotates around the x-axis, then around the y-axis, then it translates
#[wasm_bindgen]
pub fn update_camera( cam_x : f32, cam_y : f32, cam_z : f32, cam_rot_x : f32, cam_rot_y : f32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.camera = Camera::new( Vec3::new( cam_x, cam_y, cam_z ), cam_rot_x, cam_rot_y );
      ray_store_done( );
    } else {
      panic!( "init not called" )
    }
  }
}

/// Actually traces all the rays
/// Note that it only traces rays whose pixels are assigned to this instance.
///   (in multi-threading different instances are assigned different pixels)
#[wasm_bindgen]
pub fn compute( ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      // These two loops are extracted (instead of checking for `is_depth` in the body),
      //   because I'm unsure whether the compiler hoists this. So, I hoist it.
      if conf.is_depth {
        compute_depth( &mut conf )
      } else {
        compute_color( &mut conf )
      }
    } else {
      panic!( "init not called" )
    }
  }
}

/// Traces rays to obtain a depth buffer of the scene (for assigned pixels)
fn compute_depth( conf : &mut Config ) {
  let mat_stack = &mut conf.mat_stack;

  for i in 0..(conf.num_rays as usize) {
    let (x, y) = conf.pixel_coords[ i ];

    let res = trace_original_depth( &conf.scene, &conf.rays[ i ] ).clamp( );

    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 0 ) as usize ] = ( 255.0 * res.red ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 1 ) as usize ] = ( 255.0 * res.green ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 2 ) as usize ] = ( 255.0 * res.blue ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 3 ) as usize ] = 255;
  }
}

/// Traces rays to obtain a diffuse buffer of the scene (for assigned pixels)
fn compute_color( conf : &mut Config ) {
  let mat_stack = &mut conf.mat_stack;

  for i in 0..(conf.num_rays as usize) {
    let (x, y) = conf.pixel_coords[ i ];

    // Note that `mat_stack` already contains the "material" for air (so now it's a stack of air)
    let (d, res) = trace_original_color( &conf.scene, &conf.rays[ i ], conf.max_ray_depth, mat_stack );

    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 0 ) as usize ] = ( 255.0 * res.red ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 1 ) as usize ] = ( 255.0 * res.green ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 2 ) as usize ] = ( 255.0 * res.blue ) as u8;
    conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 3 ) as usize ] = 255;
  }
}

// Scenes are numbered in the interface. This functions performs the mapping
fn select_scene( id : u32 ) -> Scene {
  match id {
    0 => setup_scene( ),
    1 => setup_ball_scene( ),
    2 => setup_scene_ballsphere( ),
    _ => setup_scene( )
  };
}
