use crate::graphics::{Color3};
use crate::graphics::lights::Light;
use crate::graphics::ray::{Ray, Marchable};
use crate::graphics::scene::{LightHit};
use crate::math::{EPSILON, Vec3};
use std::rc::Rc;

pub struct MarchScene {
  pub background : Color3,
  pub lights     : Vec< Light >,
  pub shapes     : Vec< Rc< dyn Marchable > >
}

impl MarchScene {
  // Constructs a new scene with the specified lights and shapes
  pub fn new( background : Color3
            , lights     : Vec< Light >
            , shapes     : Vec< Rc< dyn Marchable > >
            ) -> MarchScene {
    MarchScene { background, lights, shapes }
  }
  
  // Casts a shadow ray from the `hit_loc` to all lights in the scene
  // All non-occluded lights are returned by this function
  // WARNING: Shadow-biassing should be applied externally
  pub fn shadow_ray( &self, hit_loc : &Vec3, light_id : usize ) -> Option< LightHit > {
    match &self.lights[ light_id ] {
      Light::Point( ref l ) => {
        let mut to_light : Vec3 = l.location - *hit_loc;
        let distance_sq = to_light.len_sq( );
        let distance = distance_sq.sqrt( );
        to_light = to_light / distance;

        let shadow_ray = Ray::new( *hit_loc, to_light );
        if self.march( &shadow_ray, distance ).is_none( ) {
          Some( LightHit { dir: to_light, color: l.color, distance_sq: Some( distance_sq ) } )
        } else {
          None
        }
      },
      Light::Directional( ref l ) => {
        let to_light   = -l.direction;
        let shadow_ray = Ray::new( *hit_loc, to_light );
        if self.march( &shadow_ray, 30.0 /* TEMP */ ).is_some( ) {
          // A shadow occludes the lightsource
          None
        } else {
          // Note that no attenuation applies here, as the lightsource is at an
          // infinite distance anyway
          Some( LightHit { dir: to_light, color: l.color.to_vec3( ), distance_sq: None } )
        }
      },
      Light::Spot( ref l ) => {
        let mut to_light : Vec3 = l.location - *hit_loc;
        let distance_sq = to_light.len_sq( );
        let distance    = distance_sq.sqrt( );
        to_light = to_light / distance;
        let from_light : Vec3 = -to_light;

        let angle_diff = from_light.dot( l.direction ).acos( );

        if angle_diff < l.angle {
          let shadow_ray = Ray::new( *hit_loc, to_light );
          if self.march( &shadow_ray, distance ).is_none( ) {
            Some( LightHit { dir: to_light, color: l.color, distance_sq: Some( distance_sq ) } )
          } else {
            // It's occluded
            None
          }
        } else {
          // Outside the spot area
          None
        }
      }
    }
  }

  pub fn march< 'a >( &'a self, ray : &Ray, max_depth : f32 ) -> Option< ( f32, &'a Rc< dyn Marchable >) > {
    let mut depth = 0.0;

    for _j in 0..128 {
      let (dist, obj) = self.sdf( &ray.at( depth ) );
      if dist < EPSILON {
        return Some( ( depth, obj ) );
      }

      depth += dist;

      if depth >= max_depth {
        return None;
      }
    }

    None
  }

  pub fn sdf< 'a >( &'a self, p : &Vec3 ) -> ( f32, &'a Rc< dyn Marchable >) {
    // assert( shapes.len( ) > 0 )
    let mut x = ( self.shapes[ 0 ].sdf( p ), &self.shapes[ 0 ] );

    for i in 1..self.shapes.len( ) {
      let d = self.shapes[ i ].sdf( p );

      if d < x.0 {
        x = ( d, &self.shapes[ i ] );
      }
    }

    x
  }
}
