use crate::data::cap_stack::Stack;
use crate::graphics::{Color3, PointMaterial, Scene};
use crate::graphics::ray::{Ray};
use crate::math::Vec3;
use crate::math;
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

/// Traces an original ray, and produces a gray-scale value for that ray
/// White values are close, black are far away
pub fn trace_original_depth( scene : &Scene, ray : &Ray ) -> (usize,f32) {
  let (d, res) = scene.trace_simple( ray );
  if let Some( v ) = res {
    (d, v)
  } else {
    (d, INFINITY)
  }
}

pub fn trace_original_bvh( scene : &Scene, ray : &Ray ) -> usize {
  let (d, _) = scene.trace( ray );
  d
}

/// Traces an original ray, and produces a color for that ray
pub fn trace_original_color( scene : &Scene, ray : &Ray, max_rays : u32, refr_stack : &mut Stack< MatRefract > ) -> (usize,Color3) {
  let (d, _, c) = trace_color( scene, ray, max_rays, refr_stack );
  (d, c)
}

/// Traces a ray into the scene, and returns both its distance and color to the
/// first hit. If no hit found it returns a distance of 0 and the color black.
/// This should be of no effect anyway.
fn trace_color( scene : &Scene, ray : &Ray, max_rays : u32, refr_stack : &mut Stack< MatRefract > ) -> (usize, f32, Color3) {
  if let (u0, Some( h )) = scene.trace( ray ) {
    let hit_loc = ray.at( h.distance );
    let mut u_sum = u0;

    let color =
      match h.mat {
        PointMaterial::Reflect { color, reflection, ks, n } => {
          let (u1,light_color) = lights_color( scene, &hit_loc, &h.normal );
          let (u2,res) = specular_lights_color( scene, &hit_loc, &ray.dir, &h.normal, ks, n );
          let specular_light_color = Color3::from_vec3( res );
          u_sum += u1 + u2;

          if max_rays > 0 && reflection > 0.0 {
            let refl_dir             = (-ray.dir).reflect( h.normal );
            let refl_ray             = Ray::new( hit_loc + math::EPSILON * refl_dir, refl_dir );
            let (u, _, refl_diffuse) = trace_color( scene, &refl_ray, max_rays - 1, refr_stack );
            let diffuse_color        = reflection * refl_diffuse + ( 1.0 - reflection ) * color;
            u_sum += u;
            (light_color * diffuse_color + specular_light_color)
          } else { // If it's at the cap, just apply direct illumination
            (light_color * color + specular_light_color)
          }
        },

        PointMaterial::Refract { absorption, refractive_index, ks, n } => {
          let (u,res) = specular_lights_color( scene, &hit_loc, &ray.dir, &h.normal, ks, n );
          let specular_light_color = Color3::from_vec3( res );
          u_sum += u;

          let (obj_refractive_index, outside_refr_index, is_popped) =
            if h.is_entering {
              let outside_mat = refr_stack.top( ).unwrap( );
              ( refractive_index, outside_mat.refractive_index, false )
            } else {
              let ip = !refr_stack.pop_until1( ).is_none( ); // This is the object's material
              let outside_mat = refr_stack.top( ).unwrap( );
              ( outside_mat.refractive_index, refractive_index, ip )
            };

          let (kr, refr_color) =
            if max_rays > 0 {
              if let Some( ( kr, refr_dir ) ) = refract_fresnel( ray.dir, h.normal, outside_refr_index, obj_refractive_index ) {
                // No total internal reflection (refract(..) returns None if that were so). So kr < 1.0
                // Cast refraction ray
                let refr_ray = Ray::new( hit_loc + refr_dir * math::EPSILON, refr_dir );

                // Beer's law
                // With some additional stuff. Consider the following situation
                // --->| A   |B|   A   |C|    A |--->
                // Here the object A contains two objects B and C
                // but how does the central part of A absorb stuff? It is unknown from leaving B or entering C.
                // Thus keep a stack for these weird cases
                // Note, however, if the original ray starts inside a mesh, stuff goes wrong (so don't do this =D )
                if h.is_entering {
                  // This object is the contained object's outside
                  refr_stack.push( MatRefract::new( absorption, obj_refractive_index ) );
                  let (u,d,c) = trace_color( scene, &refr_ray, max_rays - 1, refr_stack );
                  u_sum += u;
                  refr_stack.pop_until1( );
                  ( kr, c * ( -absorption * d ).exp( ) )
                } else { // leaving the object
                  // Note that in this case the material was popped before, and is pushed after
                  // Which is done externally
                  let (u,d,c) = trace_color( scene, &refr_ray, max_rays - 1, refr_stack );
                  u_sum += u;

                  let a = refr_stack.top( ).unwrap( ).absorption;
                  ( kr, c * ( -a * d ).exp( ) )
                }
              } else { // Total internal reflection
                ( 1.0, Color3::BLACK )
              }
            } else {
              // This means very little, but happens when the rays don't want to
              // go any further. Instead of black, choose a sensible color
              let habs = absorption.x.max( absorption.y ).max( absorption.z );
              ( 1.0, Color3::new( 1.0 - absorption.x / habs, 1.0 - absorption.y / habs, 1.0 - absorption.z / habs ) )
            };

          if is_popped {
            // This was popped before, so put it back. We're inside the object again
            refr_stack.push( MatRefract::new( absorption, refractive_index ) );
          }

          let refl_color =
            if max_rays > 0 && kr > 0.0 {
              let refl_dir = (-ray.dir).reflect( h.normal );
              let refl_ray = Ray::new( hit_loc + refl_dir * math::EPSILON, refl_dir );
              let (u, _, c) = trace_color( scene, &refl_ray, max_rays - 1, refr_stack );
              u_sum += u;
              c
            } else {
              // This means very little, but happens when the rays don't want to
              // go any further. Instead of black, choose a sensible color
              let habs = absorption.x.max( absorption.y ).max( absorption.z );
              let c = Color3::new( 1.0 - absorption.x / habs, 1.0 - absorption.y / habs, 1.0 - absorption.z / habs );
              c
            };

          (refl_color * kr + specular_light_color + refr_color * ( 1.0 - kr ))
        }
      };

    ( u_sum, h.distance, color )
  } else {
    ( 0, 0.0, scene.background )
  }
}

/// The sum of the contribution of all lights in the scene toward the `hit_loc`
/// This takes into account:
/// * Occlusion (shadow-rays; occluded sources do not contribute)
/// * Distance
/// * Angle of hit
fn lights_color( scene : &Scene, hit_loc : &Vec3, hit_normal : &Vec3 ) -> (usize, Vec3) {
  let mut light_color = Vec3::ZERO;
  let mut u_sum = 0;
  for l_id in 0..scene.lights.len( ) {
    let (u, res) = scene.shadow_ray( &hit_loc, l_id );
    if let Some( light_hit ) = res {
      let attenuation =
        if let Some( dis_sq ) = light_hit.distance_sq {
          1.0 / dis_sq
        } else {
          1.0
        };
      light_color += light_hit.color * attenuation * 0.0_f32.max( hit_normal.dot( light_hit.dir ) );
    }
    u_sum += u;
  }
  (u_sum, light_color)
}

/// Computes the specular highlight from the Phong illumination model
/// The "reflects" the light-source onto the object
fn specular_lights_color( scene : &Scene, hit_loc : &Vec3, i : &Vec3, normal : &Vec3, ks : f32, n : f32 ) -> (usize, Vec3) {
  if ks == 0.0 {
    return (0, Vec3::ZERO);
  }

  let mut specular_color = Vec3::ZERO;
  let mut u_sum = 0;
  for l_id in 0..scene.lights.len( ) {
    let (u, res) = scene.shadow_ray( &hit_loc, l_id );
    if let Some( light_hit ) = res {
      // The reflection of the vector to the lightsource
      let refl_l = light_hit.dir.reflect( *normal );
      specular_color += light_hit.color * ks * 0.0_f32.max( refl_l.dot( -*i ) ).powf( n );
    }
    u_sum += u;
  }
  (u_sum, specular_color)
}

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
