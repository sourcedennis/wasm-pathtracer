// External imports
use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
// Local imports
use crate::graphics::{Scene};
use crate::graphics::ray::{Tracable};
use crate::graphics::primitives::{Triangle};
use crate::graphics::{Mesh, Texture, Color3};
use crate::math::{Vec3};
use crate::scenes::{setup_scene_museum, setup_scene_bunny_high};
use crate::tracer::{RenderInstance, RenderType, Camera};
use crate::graphics::{Material};
use crate::rng::Rng;
use crate::render_target::{RenderTarget, SimpleRenderTarget};
use crate::graphics::{SamplingStrategy, RandomSamplingStrategy, AdaptiveSamplingStrategy};

// This file contains all the functions that are exposed through WebAssembly
// Interfacing with JavaScript is a bit annoying, as only primitives (i32, i64, f32, f64)
// can be passed across the "boundary".
// I purposefully avoid "bridging" JavaScript code that is generated by wasm-pack,
// because I'm unsure about performance penalties this may incur. So I write "simple bridges"
// with only primitives.

// The intuition about the tracing work is as follows:
// * This instance is initialised with session information (viewport, camera, etc.)
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
  textures        : HashMap< u32, Texture >,
  rng             : Rc< RefCell< Rng > >,

  // ## Session State
  // The actual produced diffuse buffer
  target          : Rc< RefCell< RenderTarget > >,
  // A buffer that shows the pixels that are most likely to be sampled
  sampling_target : Rc< RefCell< SimpleRenderTarget > >,

  scene_id        : u32,
  scene           : Rc< Scene >,
  camera          : Rc< RefCell< Camera > >,

  // The viewport is split into two halves. The different parts can have
  // different rendering settings. Which is mainly useful for debugging.
  left_instance   : RenderInstance,
  right_instance  : RenderInstance
}

/// This is global state, which it must be. WASM is called through
///   JS which owns the (global) state. Consider this whole WASM
///   module as a single encapsulated entity, with its own state.
static mut CONFIG : Option< Config > = None;

/// Initialises the *Session State*.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn init( width : u32, height : u32, scene_id : u32, render_type : u32
           , cam_x : f32, cam_y : f32, cam_z : f32, cam_rot_x : f32, cam_rot_y : f32 ) {
  unsafe {
    // Here is quite some code duplication, but this is hard to avoid as global state needs
    // to remain preserved. Doing this otherwise causes Rust to allocate a copy of this global
    // state, which is too expensive. (It contains all triangle meshes)
    
    if !CONFIG.is_none( ) {
      panic!( "Cannot init again" );
    }

    let left_width = ( width / 2 ) as usize;

    let camera          = Rc::new( RefCell::new( Camera::new( Vec3::new( cam_x, cam_y, cam_z ), cam_rot_x, cam_rot_y ) ) );
    let target          = Rc::new( RefCell::new( RenderTarget::new( width as usize, height as usize ) ) );
    let sampling_target = Rc::new( RefCell::new( SimpleRenderTarget::new( width as usize, height as usize ) ) );
    
    let meshes   = HashMap::new( );
    let textures = HashMap::new( );
    let scene    = Rc::new( select_scene( scene_id, &meshes, &textures ) );
    let rng      = Rc::new( RefCell::new( Rng::new( ) ) );

    // The initial settings in the Elm panel are reflected here.
    let left_sampling  = Box::new( RandomSamplingStrategy::new( 0, 0, left_width, height as usize, rng.clone( ), sampling_target.clone( ) ) );
    let right_sampling = Box::new( AdaptiveSamplingStrategy::new( left_width, 0, width as usize - left_width, height as usize, target.clone( ), rng.clone( ), sampling_target.clone( ) ) );

    let left_instance  = RenderInstance::new( scene.clone( ), camera.clone( ), rng.clone( ), left_sampling,  false, target.clone( ), RenderType::NormalNEE );
    let right_instance = RenderInstance::new( scene.clone( ), camera.clone( ), rng.clone( ), right_sampling, false, target.clone( ), RenderType::PNEE );

    CONFIG = Some( Config {
      // ## Global State
      meshes
    , textures
    , rng:              rng.clone( )

      // ## Session State
    , target
    , sampling_target
    , scene_id
    , scene:            scene.clone( )
    , camera

    , left_instance
    , right_instance
    } );
  }
}

/// Returns a pointer to the resulting buffer
/// This buffer is of size `viewport_width * viewport_height`
/// If `is_show_sampling` is 1, the pixel sampling frequency is shown instead
#[wasm_bindgen]
#[allow(dead_code)]
pub fn results( is_show_sampling : u32 ) -> *const u8 {
  unsafe {
    if let Some( ref conf ) = CONFIG {
      if is_show_sampling == 1 {
        let sampling_target = conf.sampling_target.borrow( );
        sampling_target.results( ).as_ptr( )
      } else {
        let target = conf.target.borrow( );
        target.results( ).as_ptr( )
      }
    } else {
      panic!( "init not called" )
    }
  }
}

pub fn reset( ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.target.borrow_mut( ).clear( );
      conf.sampling_target.borrow_mut( ).clear( );
      conf.left_instance.reset( );
      conf.right_instance.reset( );
    } else {
      panic!( "init not called" )
    }
  }
}

/// Updates the rendered scene
/// Other aspects of the session remain the same
#[wasm_bindgen]
#[allow(dead_code)]
pub fn update_scene( scene_id : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.scene_id = scene_id;
      conf.scene    = Rc::new( select_scene( scene_id, &conf.meshes, &conf.textures ) );
      conf.target.borrow_mut( ).clear( );
      conf.sampling_target.borrow_mut( ).clear( );

      conf.left_instance.update_scene( conf.scene.clone( ) );
      conf.right_instance.update_scene( conf.scene.clone( ) );
    } else {
      panic!( "init not called" )
    }
  }
}

#[wasm_bindgen]
#[allow(dead_code)]
pub fn update_settings( left_type : u32, right_type : u32, is_left_adaptive : u32, is_right_adaptive : u32, is_light_debug : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      let mut target = conf.target.borrow_mut( );

      let width  = target.viewport_width as usize;
      let height = target.viewport_height as usize;

      let left_width = ( width / 2 ) as usize;
    
      let left_sampling : Box< dyn SamplingStrategy > =
        if is_left_adaptive == 1 {
          Box::new( AdaptiveSamplingStrategy::new( 0, 0, left_width, height, conf.target.clone( ), conf.rng.clone( ), conf.sampling_target.clone( ) ) )
        } else {
          Box::new( RandomSamplingStrategy::new( 0, 0, left_width, height, conf.rng.clone( ), conf.sampling_target.clone( ) ) )
        };
      let right_sampling : Box< dyn SamplingStrategy >  =
        if is_right_adaptive == 1 {
          Box::new( AdaptiveSamplingStrategy::new( left_width, 0, width as usize - left_width, height as usize, conf.target.clone( ), conf.rng.clone( ), conf.sampling_target.clone( ) ) )
        } else {
          Box::new( RandomSamplingStrategy::new( left_width, 0, width as usize - left_width, height as usize, conf.rng.clone( ), conf.sampling_target.clone( ) ) )
        };
    
      target.clear( );
      conf.sampling_target.borrow_mut( ).clear( );
      conf.left_instance  = RenderInstance::new( conf.scene.clone( ), conf.camera.clone( ), conf.rng.clone( ), left_sampling,  is_light_debug == 1, conf.target.clone( ), to_render_type( left_type ) );
      conf.right_instance = RenderInstance::new( conf.scene.clone( ), conf.camera.clone( ), conf.rng.clone( ), right_sampling, is_light_debug == 1, conf.target.clone( ), to_render_type( right_type ) );
    } else {
      panic!( "init not called" )
    }
  }
}

fn to_render_type( t : u32 ) -> RenderType {
  match t {
    0 => RenderType::NoNEE,
    1 => RenderType::NormalNEE,
    2 => RenderType::PNEE,
    _ => panic!( "Invalid RenderType magic number" )
  }
}

/// Updates the viewport, and thus the render buffer
#[wasm_bindgen]
#[allow(dead_code)]
pub fn update_viewport( width : u32, height : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      *conf.target.borrow_mut( )          = RenderTarget::new( width as usize, height as usize );
      *conf.sampling_target.borrow_mut( ) = SimpleRenderTarget::new( width as usize, height as usize );
      let left_width = width / 2;
      conf.left_instance.resize( 0, 0, left_width as usize, height as usize );
      conf.right_instance.resize( left_width as usize, 0, ( width - left_width ) as usize, height as usize );
      reset( );
    } else {
      panic!( "init not called" )
    }
  }
}

/// Updates the camera in the session
/// Other aspects of the session remain the same
/// Note that the camera first rotates around the x-axis, then around the y-axis, then it translates
#[wasm_bindgen]
#[allow(dead_code)]
pub fn update_camera( cam_x : f32, cam_y : f32, cam_z : f32, cam_rot_x : f32, cam_rot_y : f32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      *conf.camera.borrow_mut( ) = Camera::new( Vec3::new( cam_x, cam_y, cam_z ), cam_rot_x, cam_rot_y );
      reset( );
    } else {
      panic!( "init not called" )
    }
  }
}

// Mesh allocation happens in three stages:
// * First the space for the vertices is allocated
// * Then TypeScript stores the vertices in WASM's memory
// * Then, if the current scene is supposed to contain that mesh,
//     it is rebuilt with the mesh
//
// This is the first stage
#[wasm_bindgen]
#[allow(dead_code)]
pub fn allocate_mesh( id : u32, num_vertices : u32 ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.meshes.insert(
          id
        , Mesh::Preload( vec![Vec3::ZERO; num_vertices as usize] )
        );
    } else {
      panic!( "init not called" )
    }
  }
}

/// Obtains a pointer to the mesh vertices
#[wasm_bindgen]
#[allow(dead_code)]
pub fn mesh_vertices( id : u32 ) -> *mut Vec3 {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      if let Some( Mesh::Preload( ref mut m ) ) = conf.meshes.get_mut( &id ) {
        m.as_mut_ptr( )
      } else {
        panic!( "Mesh not allocated" )
      }
    } else {
      panic!( "init not called" )
    }
  }
}

/// Notifies the raytracer that all the mesh vertices are placed in WASM
/// memory. Returns `true` if a scene with the loaded mesh is currently rendering
#[wasm_bindgen]
#[allow(dead_code)]
pub fn notify_mesh_loaded( id : u32 ) -> bool {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      if let Some( Mesh::Preload( ref m ) ) = conf.meshes.get_mut( &id ) {
        let num_triangles = m.len( ) / 3;
        let mut triangles : Vec< Rc< dyn Tracable > > = Vec::with_capacity( num_triangles );

        let mat = Material::diffuse( Color3::new( 1.0, 0.4, 0.4 ) );

        for i in 0..num_triangles {
          // These are actually transformations within the scene
          // But do perform them here, instead of upon each scene construction
          let mut triangle =
            Triangle::new( m[ i * 3 + 0 ] * 0.5, m[ i * 3 + 1 ] * 0.5, m[ i * 3 + 2 ] * 0.5
                , mat.clone( ) );
          triangle = triangle.translate( Vec3::new( 0.0, 0.0, 5.0 ) );

          triangles.push( Rc::new( triangle ) );
        }

        conf.meshes.insert( id, Mesh::Triangled( triangles ) );
      }

      // Scene 1 uses mesh 0. Scene 2 uses mesh 1. Scene 3 uses mesh 2
      if ( id == 0 && conf.scene_id == 1 ) ||
         ( id == 1 && conf.scene_id == 2 ) ||
         ( id == 2 && conf.scene_id == 3 ) {
        update_scene( conf.scene_id );
        true
      } else {
        false
      }
    } else {
      panic!( "init not called" )
    }
  }
}

/// Allocates a texture identifier by the provided `id` with the provided size
/// Returns a pointer to the u8 RGB store location
#[wasm_bindgen]
#[allow(dead_code)]
pub fn allocate_texture( id : u32, width : u32, height : u32 ) -> *mut (u8,u8,u8) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.textures.insert(
          id
        , Texture::new( width, height )
        );
      if let Some( t ) = conf.textures.get_mut( &id ) {
        t.data.as_mut_ptr( )
      } else {
        // Shouldn't happen
        panic!( "HashMap error" )
      }
    } else {
      panic!( "init not called" )
    }
  }
}

/// Notifies the raytracer that the texture RGB data has been put into WASM's
/// memory. If the current scene is using that texture, the scene is updated
#[wasm_bindgen]
#[allow(dead_code)]
pub fn notify_texture_loaded( _id : u32 ) -> bool {
  unsafe {
    if let Some( ref mut _conf ) = CONFIG {
      false
    } else {
      panic!( "init not called" )
    }
  }
}

/// Actually traces the rays
/// Note that it only traces rays whose pixels are assigned to this instance.
///   (in multi-threading different instances are assigned different pixels)
/// Returns the number of intersected BVH nodes
#[wasm_bindgen]
#[allow(dead_code)]
pub fn compute( num_samples : usize ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      let num_samples_left = num_samples / 2;
      conf.left_instance.compute( num_samples_left );
      conf.right_instance.compute( num_samples - num_samples_left );
    } else {
      panic!( "init not called" )
    }
  }
}

// Scenes are numbered in the interface. This functions performs the mapping
// Note that some scenes require externally obtained meshes, that's why these
//   are passed along as well
fn select_scene( id       : u32
               , meshes   : &HashMap< u32, Mesh >
               , _textures : &HashMap< u32, Texture >
               ) -> Scene {
  match id {
    0 => setup_scene_museum( ),
    2 => setup_scene_bunny_high( meshes ),
    _ => panic!( "Invalid scene" )
  }
}
