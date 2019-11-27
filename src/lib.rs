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
use scene::{Tracable, Light, Scene, Sphere, Plane, AABB, Triangle};
use std::collections::HashMap;

// Z points INTO the screen. -Z points to the eye

static mut CONFIG : Option< Config > = None;
static mut MESHES : Option< HashMap< u32, Mesh > > = None;

struct Mesh {
  vertices : Vec< Vec3 >,
  normals  : Vec< Vec3 >
}

struct Config {
  viewport_width   : u32,
  viewport_height  : u32,
  is_depth         : bool,
  resultbuffer     : Vec< u8 >,
  pixel_coords     : Vec< ( u32, u32 ) >,
  // Original rays are cached =D
  rays             : Vec< Ray >,
  num_rays         : u32,
  scene            : Scene,
  max_reflect      : u32
}

#[wasm_bindgen]
pub fn init( width : u32, height : u32, is_depth : u32, max_reflect : u32 ) {
  unsafe {
    let dir_temp = Vec3::ZERO;
    let origin = Vec3::new( 0.0, 0.0, -2.0 );
    let rays = vec![Ray::new( origin, dir_temp ); (width*height) as usize];

    CONFIG = Some( Config {
      viewport_width:   width
    , viewport_height:  height
    , is_depth:         is_depth != 0
    , resultbuffer:     vec![0; (width*height*4) as usize]
    , pixel_coords:     vec![(0,0); (width*height) as usize]
    , rays:             rays
    , num_rays:         0
    , scene:            setup_scene( )
    , max_reflect
    } );
  }
}

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

#[wasm_bindgen]
pub fn ray_store_done( ) {
  unsafe {
    let origin = Vec3::new( 0.0, 0.0, -2.0 );

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
        
        let pixel = Vec3::new( fx, fy, 0.0 );
        let dir   = ( pixel - origin ).normalize( );
  
        conf.rays[ i ].dir = dir;
      }
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

// (<any> this._instance.exports ).allocateMesh( idInt, triangles.vertices.length, triangles.normals.length );

// let vPtr = (<any> this._instance.exports ).meshVertices( idInt );
// let nPtr = (<any> this._instance.exports ).meshNormals( idInt );

#[wasm_bindgen]
pub fn allocate_mesh( id : u32, num_vertices : u32, num_normals : u32 ) {
  unsafe {
    let meshmap = meshes( );
    meshmap.insert(
        id
      , Mesh { vertices: vec![Vec3::ZERO; num_vertices as usize]
             , normals: vec![Vec3::ZERO; num_normals as usize]
             }
      );
  }
}

#[wasm_bindgen]
pub fn mesh_vertices( id : u32 ) -> *const Vec3 {
  unsafe {
    let meshmap = meshes( );
    if let Some( ref mut m ) = meshmap.get( &id ) {
      m.vertices.as_ptr( )
    } else {
      panic!( "Mesh not allocated" )
    }
  }
}

#[wasm_bindgen]
pub fn mesh_normals( id : u32 ) -> *const Vec3 {
  unsafe {
    let meshmap = meshes( );
    if let Some( ref mut m ) = meshmap.get( &id ) {
      m.normals.as_ptr( )
    } else {
      panic!( "Mesh not allocated" )
    }
  }
}

// Returns `true` if a scene with the loaded mesh is currently rendering
#[wasm_bindgen]
pub fn notify_mesh_loaded( id : u32 ) -> bool {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
      conf.scene = setup_scene( );
      true
    } else {
      false
    }
  }
}

#[wasm_bindgen]
pub fn compute( ) {
  unsafe {
    if let Some( ref mut conf ) = CONFIG {
 
      if conf.is_depth {
        for i in 0..(conf.num_rays as usize) {
          let (x, y) = conf.pixel_coords[ i ];
  
          let res = trace_original_depth( &conf.scene, &conf.rays[ i ] ).clamp( );
  
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 0 ) as usize ] = ( 255.0 * res.red ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 1 ) as usize ] = ( 255.0 * res.green ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 2 ) as usize ] = ( 255.0 * res.blue ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 3 ) as usize ] = 255;
        }
      } else {
        for i in 0..(conf.num_rays as usize) {
          let (x, y) = conf.pixel_coords[ i ];
  
          let (d, res) = trace_original_color( &conf.scene, &conf.rays[ i ], conf.max_reflect );
  
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 0 ) as usize ] = ( 255.0 * res.red ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 1 ) as usize ] = ( 255.0 * res.green ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 2 ) as usize ] = ( 255.0 * res.blue ) as u8;
          conf.resultbuffer[ ( ( y * conf.viewport_width + x ) * 4 + 3 ) as usize ] = 255;
        }
      }
    } else {
      panic!( "init not called" )
    }
  }
}

unsafe fn meshes( ) -> &'static mut HashMap< u32, Mesh > {
  if let Some( ref mut meshmap ) = MESHES {
    meshmap
  } else {
    MESHES = Some( HashMap::new( ) );
    meshes( )
  }
}

unsafe fn setup_scene( ) -> Scene {
  let light = Light::new( Vec3::new( 0.0, 6.0, 4.5 ), Color3::new( 0.7, 0.7, 0.7 ) );

  // MatDiffuse { color : Color3 },
  // MatReflect { color : Color3, reflection : f32 },
  // MatRefract { reflection : f32, absorption : Color3, refractive_index : f32 }

  let mut shapes: Vec< Box< dyn Tracable > > = Vec::new( );
  shapes.push( Box::new( Sphere::new( Vec3::new(  0.0, 1.0, 5.0 ), 1.0, Material::refract( Vec3::new( 0.1, 0.2, 0.1 ), 1.5 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ) ) ) );
  shapes.push( Box::new( AABB::cube( Vec3::new(  -1.7, 0.0 + math::EPSILON * 2.0, 7.0 ), 1.0, Material::refract( Vec3::new( 0.1, 0.2, 0.2 ), 1.5 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );


  // If the rabbit is loaded
  /*if let Some( rabbit ) = meshes( ).get( &0 ) {
    for i in 0..(rabbit.vertices.len()/3) {
      let mut triangle =
        Triangle::new( rabbit.vertices[ i * 3 + 0 ] * 0.5, rabbit.vertices[ i * 3 + 1 ] * 0.5, rabbit.vertices[ i * 3 + 2 ] * 0.5
                     , rabbit.normals[ i * 3 + 0 ],  rabbit.normals[ i * 3 + 1 ],  rabbit.normals[ i * 3 + 2 ]
                     , Material::diffuse( Color3::new( 1.0, 0.4, 0.4 ) ) );
      triangle = triangle.translate( Vec3::new( 0.0, -0.8, 5.0 ) );
      shapes.push( Box::new( triangle ) );
    }
  }*/

  Scene::new( vec![ light ], shapes )
  /*let sphereLoc  = Vec3::new( 0.0, 0.0, 5.0 );

  let lightLoc   = Vec3::new( -0.5, 2.0, 1.0 );
  let lightColor = Color3::new( 0.7, 0.7, 0.7 );
  let light = Light::new( lightLoc, lightColor );

  let mut shapes: Vec< Box< dyn Tracable > > = Vec::new( );
  shapes.push( Box::new( Sphere::new( sphereLoc, 1.0, Material::diffuse( Color3::new( 0.0, 0.0, 1.0 ) ) ) ) );

  Scene::new( vec![ light ], shapes )*/
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

pub fn hit_sphere( pos : Vec3, r : f32, ray : Ray ) -> Option< Hit > {
  // Using algebraic solution. (Non-geometric)
  // Solve: ((O-P)+D*t)^2 - R^2
  let a = 1_f32; // D^2
  let b = 2_f32 * ray.dir.dot( ray.origin - pos );
  let c = ( ray.origin - pos ).dot( ray.origin - pos ) - r*r;
  let D = b * b - 4_f32 * a * c;

  if D < 0_f32 {
    return None;
  }

  let t0 = ( -b + D.sqrt( ) ) / ( 2_f32 * a );
  let t1 = ( -b - D.sqrt( ) ) / ( 2_f32 * a );

  let mut t = t0.min( t1 );
  if t <= 0_f32 {
    t = t0.max( t1 );

    if t <= 0_f32 { // The sphere is fully behind the "camera"
      return None
    }
  }

  let normal = ( ray.at( t ) - pos ) / r;

  return Some( Hit::new( t, normal, Material::diffuse( Color3::new( 1.0, 0.0, 0.0 ) ), true ) );
}
