use crate::graphics::material::{Material};
use crate::graphics::color3::{Color3};
use crate::graphics::ray::{Ray, Hit};
use crate::graphics::lights::PointLight;
use crate::math::vec3::{Vec3};
use crate::math::EPSILON;

// A Scene consists of shapes and lights
// The camera is *not* part of the scene
//
// (For specific scenes, look at the `/scenes.rs` file)
pub struct Scene {
  pub lights : Vec< PointLight >,
  pub shapes : Vec< Box< dyn Tracable > >
}

// A "hit" for a pointlight source
// If such a hit exists, there is a non-occluded ray from a surface point to
//   the light source. (This is used for casting shadow rays)
pub struct LightHit {
  pub dir      : Vec3,
  pub distance : f32,
  pub color    : Color3,
}

impl Scene {
  // Constructs a new scene with the specified lights and shapes
  pub fn new( lights : Vec< Light >, shapes : Vec< Box< dyn Tracable > > ) -> Scene {
    Scene { lights, shapes }
  }

  // Casts a shadow ray from the `hit_loc` to all lights in the scene
  // All non-occluded lights are returned by this function
  pub fn shadow_ray( &self, hit_loc : &Vec3, light_id : usize ) -> Option< LightHit > {
    let l = &self.lights[ light_id ];
    let mut to_light = l.location - *hit_loc;
    let distance = to_light.len( );
    to_light = to_light / distance;

    let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
    if !is_hit_within_sq( self.trace( &shadow_ray ), ( l.location - *hit_loc ).len_sq( ) ) {
      Some( LightHit { dir: to_light, distance, color: l.color } )
    } else {
      None
    }
  }
}

// Returns only true if a hit occurs and it occurs within at most `sqrt(d_sq)` units
// `d_sq` is the square of the distance - for efficiency reasons
fn is_hit_within_sq( m_hit : Option< Hit >, d_sq : f32 ) -> bool {
  if let Some( h ) = m_hit {
    h.distance * h.distance < d_sq
  } else {
    false
  }
}

impl Tracable for Scene {
  fn trace( &self, ray : &Ray ) -> Option< Hit > {
    let mut best_hit: Option< Hit > = None;

    for s in &self.shapes {
      let new_hit: Option< Hit > = s.trace( ray );

      if let Some( nh ) = new_hit {
        if let Some( bh ) = best_hit {
          if nh.distance < bh.distance {
            best_hit = new_hit;
          }
        } else {
          best_hit = new_hit;
        }
      }
    }

    if let Some( bh ) = best_hit {
      if bh.distance <= 0.0 {
        None
      } else {
        best_hit
      }
    } else {
      None
    }
  }
}
