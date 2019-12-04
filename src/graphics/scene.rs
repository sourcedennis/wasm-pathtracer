use crate::graphics::{Color3};
use crate::graphics::ray::{Ray, Hit, Tracable};
use crate::graphics::lights::Light;
use crate::math::{Vec3, EPSILON};

// A Scene consists of shapes and lights
// The camera is *not* part of the scene
//
// (For specific scenes, look at the `/scenes.rs` file)
pub struct Scene {
  pub background : Color3,
  pub lights     : Vec< Light >,
  pub shapes     : Vec< Box< dyn Tracable > >
}

// A "hit" for a light source
// If such a hit exists, there is a non-occluded ray from a surface point to
//   the light source. (This is used for casting shadow rays)
pub struct LightHit {
  pub dir      : Vec3,
  // The color of the distance-attenuated light source
  pub color    : Vec3,
}

impl Scene {
  // Constructs a new scene with the specified lights and shapes
  pub fn new( background : Color3
            , lights     : Vec< Light >
            , shapes     : Vec< Box< dyn Tracable > >
            ) -> Scene {
    Scene { background, lights, shapes }
  }

  // Casts a shadow ray from the `hit_loc` to all lights in the scene
  // All non-occluded lights are returned by this function
  pub fn shadow_ray( &self, hit_loc : &Vec3, light_id : usize ) -> Option< LightHit > {
    match &self.lights[ light_id ] {
      Light::Point( ref l ) => {
        let mut to_light : Vec3 = l.location - *hit_loc;
        let distance_sq = to_light.len_sq( );
        to_light = to_light / distance_sq.sqrt( );

        let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
        if !is_hit_within_sq( self.trace_simple( &shadow_ray ), distance_sq ) {
          let attenuation = 1.0 / distance_sq;
          Some( LightHit { dir: attenuation * to_light, color: l.color } )
        } else {
          None
        }
      },
      Light::Directional( ref l ) => {
        let to_light   = -l.direction;
        let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
        if let Some( _h ) = self.trace_simple( &shadow_ray ) {
          // A shadow occludes the lightsource
          None
        } else {
          // Note that no attenuation applies here, as the lightsource is at an
          // infinite distance anyway
          Some( LightHit { dir: to_light, color: l.color.to_vec3( ) } )
        }
      },
      Light::Spot( ref l ) => {
        let mut to_light : Vec3 = ( l.location - *hit_loc );
        let distance_sq = to_light.len_sq( );
        to_light = to_light / distance_sq.sqrt( );
        let from_light : Vec3 = -to_light;

        let angle_diff = from_light.dot( l.direction ).acos( );

        if angle_diff < l.angle {
          let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
          if !is_hit_within_sq( self.trace_simple( &shadow_ray ), distance_sq ) {
            let attenuation = 1.0 / distance_sq;
            Some( LightHit { dir: attenuation * to_light, color: l.color } )
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
}

// Returns only true if a hit occurs and it occurs within at most `sqrt(d_sq)` units
// `d_sq` is the square of the distance - for efficiency reasons
fn is_hit_within_sq( m_hit : Option< f32 >, d_sq : f32 ) -> bool {
  if let Some( h ) = m_hit {
    h * h < d_sq
  } else {
    false
  }
}

impl Tracable for Scene {
  fn trace( &self, ray : &Ray ) -> Option< Hit > {
    let mut best_hit: Option< (f32, &Box< dyn Tracable >) > = None;

    for s in &self.shapes {
      if let Some( new_dis ) = s.trace_simple( ray ) {
        if let Some( (bhd, _) ) = best_hit {
          if 0.0_f32 < new_dis && new_dis < bhd {
            best_hit = Some( (new_dis, &s) );
          }
        } else {
          best_hit = Some( (new_dis, &s) );
        }
      }
    }

    if let Some( ( _, best_object ) ) = best_hit {
      best_object.trace( ray )
    } else {
      None
    }
  }
}
