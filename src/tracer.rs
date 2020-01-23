use crate::data::stack::DefaultStack;
use crate::graphics::{Color3, PointMaterial, Scene, LightEnum};
use crate::graphics::ray::{Ray};
use crate::math::Vec3;
use crate::math;
use crate::rng::Rng;
use std::f32::INFINITY;

/// Individual instances of Material constructors
/// Such that work can be split up
#[derive(Copy, Clone)]
pub struct MatReflect {
  pub color     : Color3,
  pub reflection : f32
}

impl MatReflect {
  pub fn new( color : Color3, reflection : f32 ) -> MatReflect {
    MatReflect { color, reflection }
  }
}

/// Another extracted Material constructor
#[derive(Copy, Clone)]
pub struct MatRefract {
  pub absorption       : Vec3,
  pub refractive_index : f32
}

impl MatRefract {
  // The "material" of air (real refractive index might be slightly greater)
  // Note that when `absorption` = (0,0,0); then the multipliers are: e^0.0 = 1.0
  pub const AIR: MatRefract = MatRefract { absorption: Vec3::ZERO, refractive_index: 1.0 };

  pub fn new( absorption : Vec3, refractive_index : f32 ) -> MatRefract {
    MatRefract { absorption, refractive_index }
  }
}

/// The scene camera.
/// It first rotates around the x-axis, then around the y-axis, then it translates
pub struct Camera {
  pub location : Vec3,
  pub rot_x    : f32,
  pub rot_y    : f32
}

impl Camera {
  pub fn new( location : Vec3, rot_x : f32, rot_y : f32 ) -> Camera {
    Camera { location, rot_x, rot_y }
  }
}

struct RenderInstance< 'a > {
  scene        : &'a Scene,
  rng          : &'a Rng,
  refr_stack   : &'a DefaultStack< MatRefract >,
  num_bvh_hits : usize,

  // True if Next-Event-Estimation is enabled
  has_nee : bool
}

impl< 'a > RenderInstance< 'a > {
  pub fn reset( &self ) {
    self.num_bvh_hits = 0;
  }

  /// Traces an original ray, and produces a gray-scale value for that ray
  /// White values are close, black are far away
  pub fn trace_original_depth( &mut self, ray : &Ray ) -> f32 {
    let (d, res) = self.scene.trace_simple( ray );
    self.num_bvh_hits += d;
    if let Some( v ) = res {
      v
    } else {
      INFINITY
    }
  }

  pub fn trace_original_bvh( &mut self, ray : &Ray ) {
    let (d, _) = self.scene.trace( ray );
    self.num_bvh_hits += d;
  }

  /// Traces an original ray, and produces a color for that ray
  pub fn trace_original_color( &mut self, ray : &Ray ) -> Vec3 {
    let (_, c) = self.trace_color( ray, Vec3::new( 1.0, 1.0, 1.0 ), true );
    c
  }

  /// Traces a ray into the scene, and returns both its distance and color to the
  /// first hit. This returns the radiance toward the ray (i.e. it is not
  /// "projected on the eye")
  /// * `keep_direct` - True if it should return the direct light (false after indirect bounce)
  fn trace_color( &mut self, ray : &Ray, throughput : Vec3, keep_direct : bool ) -> (f32, Vec3) {
    let (u0, mh) = self.scene.trace( ray );
    self.num_bvh_hits += u0;

    if let Some( h ) = mh {
      let hit_loc = ray.at( h.distance );
  
      // Chance of sending a *next* ray (Russian Roulette)
      let keep_chance = throughput.x.max( throughput.y ).max( throughput.z ).max( 0.1 ).min( 0.9 );
      let is_ray_kept = self.rng.next( ) <= keep_chance;
      
      match h.mat {
        PointMaterial::Reflect { color, reflection } => {
          if self.rng.next( ) <= reflection {
            // Consider it a reflective surface
            if is_ray_kept {
              let refl_dir = (-ray.dir).reflect( h.normal );
              let refl_ray = Ray::new( hit_loc + math::EPSILON * refl_dir, refl_dir );
              let (_, refl_radiance) = self.trace_color( &refl_ray, throughput, keep_direct );
              ( h.distance, refl_radiance * ( 1.0 / keep_chance ) )
            } else {
              ( h.distance, Color3::BLACK.to_vec3( ) )
            }
          } else {
            // Consider it a diffuse surface

            // Next event estimation
            let (direct_radiance, to_direct_light) = self.trace_random_light( &hit_loc, &h.normal );
            let cos_direct = 0.0_f32.max( to_direct_light.dot( h.normal ) );
            let direct_irradiance = color.to_vec3( ).mul_elem( direct_radiance ) * cos_direct;

            // Find the indirect light with the random bounce
            if is_ray_kept {
              let ( indirect_radiance, to_indirect_light ) = self.trace_random_indirect( &throughput.mul_elem( color.to_vec3( ) ), &hit_loc, &h.normal );
              let cos_indirect = 0.0_f32.max( to_indirect_light.dot( h.normal ) );
              let indirect_irradiance = color.to_vec3( ).mul_elem( indirect_radiance ) * cos_indirect;
              ( h.distance, direct_irradiance + indirect_irradiance * ( 1.0 / keep_chance ) )
            } else {
              ( h.distance, direct_irradiance )
            }
          }
        },

        PointMaterial::Refract { absorption, refractive_index } => {
          let (obj_refractive_index, outside_refr_index, is_popped) =
            if h.is_entering {
              let outside_mat = self.refr_stack.top( ).unwrap( );
              ( refractive_index, outside_mat.refractive_index, false )
            } else {
              let ip = !self.refr_stack.pop_until1( ).is_none( ); // This is the object's material
              let outside_mat = self.refr_stack.top( ).unwrap( );
              ( outside_mat.refractive_index, refractive_index, ip )
            };

          let res =
            if is_ray_kept {
              if let Some( ( kr, refr_dir ) ) = refract_fresnel( ray.dir, h.normal, outside_refr_index, obj_refractive_index ) {
                if self.rng.next( ) < kr { // Send a specular reflection ray
                  let refl_dir = (-ray.dir).reflect( h.normal );
                  let refl_ray = Ray::new( hit_loc + refl_dir * math::EPSILON, refl_dir );
                  let (_, c)   = self.trace_color( &refl_ray, throughput, keep_direct );
                  ( h.distance, c * ( 1.0 / keep_chance ) )
                } else { // Send a refraction ray
                  let refr_ray = Ray::new( hit_loc + refr_dir * math::EPSILON, refr_dir );
                  let c = self.trace_refract( &refr_ray, throughput, absorption, obj_refractive_index, h.is_entering, keep_direct );
                  ( h.distance, c * ( 1.0 / keep_chance ) )
                }
              } else { // Total internal reflection
                let refl_dir = (-ray.dir).reflect( h.normal );
                let refl_ray = Ray::new( hit_loc + refl_dir * math::EPSILON, refl_dir );
                let (_, c)   = self.trace_color( &refl_ray, throughput, keep_direct );
                ( h.distance, c * ( 1.0 / keep_chance ) )
              }
            } else {
              ( INFINITY, Color3::BLACK )
            };

          if is_popped {
            // This was popped before, so put it back. We're inside the object again
            self.refr_stack.push( MatRefract::new( absorption, refractive_index ) );
          }

          res
        }
      }
    } else { // No object hit
      // TODO: Skybox
      (INFINITY, self.scene.background.to_vec3( ) )
    }
  }

  fn trace_refract( &mut self, refr_ray : &Ray, throughput : Vec3, absorption : Vec3, obj_refractive_index : f32, is_entering : bool, keep_direct : bool ) -> Vec3 {
    // Beer's law
    // With some additional stuff. Consider the following situation
    // --->| A   |B|   A   |C|    A |--->
    // Here the object A contains two objects B and C
    // but how does the central part of A absorb stuff? It is unknown from leaving B or entering C.
    // Thus keep a stack for these weird cases
    // Note, however, if the original ray starts inside a mesh, stuff goes wrong (so don't do this =D )
    if is_entering {
      // This object is the contained object's outside
      self.refr_stack.push( MatRefract::new( absorption, obj_refractive_index ) );
      let (d,c) = self.trace_color( &refr_ray, throughput, keep_direct );
      self.refr_stack.pop_until1( );
      c.mul_elem( ( -absorption * d ).exp( ) )
    } else { // leaving the object
      // Note that in this case the material was popped before, and is pushed after
      // Which is done externally
      let (d,c) = self.trace_color( &refr_ray, throughput, keep_direct );

      let a = self.refr_stack.top( ).unwrap( ).absorption;
      c.mul_elem( ( -a * d ).exp( ) )
    }
  }

  /// Traces some direct light connection
  /// Returns the radiance and direction to the light
  fn trace_random_light( &mut self, hit_loc : &Vec3, hit_normal : &Vec3 ) -> Option< ( Vec3, Vec3 ) > {
    // Pick (uniformly) a random light source
    let num_lights = self.scene.lights.len( );
    let light_id = self.rng.nextInRange( 0, num_lights );

    match self.scene.lights[ light_id ] {
      LightEnum::Point( p ) => {
        let (u, mLightHit) = self.scene.shadow_ray_point( hit_loc, light_id );
        self.num_bvh_hits += u;
        if let Some( light_hit ) = mLightHit {
          // Note that `color` is already distance attenuated
          Some( ( light_hit.color, light_hit.dir ) )
        } else {
          None
        }
      },
      LightEnum::Area( shape_id ) => {
        let shape = self.scene.shapes[ shape_id ];
        let point = shape.pick_random( &mut self.rng, hit_loc );

        if self.scene.trace_simple(ray: &Ray)
        
        let area  = shape.project_hemisphere( hit_loc );

        None
      }
    }
  }

  /// Traces some indirect direction over the hemisphere
  /// Returns the radiance and direction to wherever the light came from
  fn trace_random_indirect( &mut self, throughput : &Vec3, hit_loc : &Vec3, hit_normal : &Vec3 ) -> (Vec3, Vec3) {

  }
}

/// The sum of the contribution of all lights in the scene toward the `hit_loc`
/// This takes into account:
/// * Occlusion (shadow-rays; occluded sources do not contribute)
/// * Distance
/// * Angle of hit
// fn lights_color( scene : &Scene, hit_loc : &Vec3, hit_normal : &Vec3 ) -> (usize, Vec3) {
//   let mut light_color = Vec3::ZERO;
//   let mut u_sum = 0;
//   for l_id in 0..scene.lights.len( ) {
//     let (u, res) = scene.shadow_ray( &hit_loc, l_id );
//     if let Some( light_hit ) = res {
//       let attenuation =
//         if let Some( dis_sq ) = light_hit.distance_sq {
//           1.0 / dis_sq
//         } else {
//           1.0
//         };
//       light_color += light_hit.color * attenuation * 0.0_f32.max( hit_normal.dot( light_hit.dir ) );
//     }
//     u_sum += u;
//   }
//   (u_sum, light_color)
// }

// /// Computes the specular highlight from the Phong illumination model
// /// The "reflects" the light-source onto the object
// fn specular_lights_color( scene : &Scene, hit_loc : &Vec3, i : &Vec3, normal : &Vec3, ks : f32, n : f32 ) -> (usize, Vec3) {
//   if ks == 0.0 {
//     return (0, Vec3::ZERO);
//   }

//   let mut specular_color = Vec3::ZERO;
//   let mut u_sum = 0;
//   for l_id in 0..scene.lights.len( ) {
//     let (u, res) = scene.shadow_ray( &hit_loc, l_id );
//     if let Some( light_hit ) = res {
//       // The reflection of the vector to the lightsource
//       let refl_l = light_hit.dir.reflect( *normal );
//       specular_color += light_hit.color * ks * 0.0_f32.max( refl_l.dot( -*i ) ).powf( n );
//     }
//     u_sum += u;
//   }
//   (u_sum, specular_color)
// }

/// Returns the amount (in range (0,1)) of reflection, and the angle of refraction
/// If None is returned, total internal reflection applies (and no refraction at all)
/// This applies both fresnel and Snell's law for refraction
fn refract_fresnel( i : Vec3, n : Vec3, prev_ior : f32, ior : f32 ) -> Option< ( f32, Vec3 ) > {
  let cosi    = math::clamp( -i.dot( n ), -1.0, 1.0 );
  let cosi_sq = cosi * cosi;
  // "Real squares cannot be less than 0" -Dennis
  let sini_sq = 0.0_f32.max( 1.0 - cosi_sq );
  let sini    = sini_sq.sqrt( );

  let snell   = prev_ior / ior;

  let sint = snell * sini;

  if sint >= 1.0 { // Total internal reflection
    None // So, reflection = 1.0
  } else {
    // Because sint < 1.0, k > 0.0
    let k = 1.0 - snell * snell * sini_sq;

    let cost = 0.0_f32.max( 1.0 - sint * sint ).sqrt( );

    // s-polarized light
    let spol = (prev_ior * cosi - ior * cost) / (prev_ior * cosi + ior * cost);
    // p-polarized light
    let ppol = (prev_ior * cost - ior * cosi) / (prev_ior * cost + ior * cosi);

    let frac_refl = 0.5 * (spol * spol + ppol * ppol);
    let refr_dir  = ( snell * i + (snell * cosi - k.sqrt()) * n ).normalize( );

    Some( ( frac_refl, refr_dir ) )
  }
}
