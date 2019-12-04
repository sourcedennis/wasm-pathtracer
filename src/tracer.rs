use crate::data::cap_stack::Stack;
use crate::graphics::{Color3, PointMaterial, Scene};
use crate::graphics::ray::{Ray, Tracable};
use crate::math::Vec3;
use crate::math;

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

struct Mesh {
  pub vertices : Vec< Vec3 >,
  pub normals  : Vec< Vec3 >
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
pub fn trace_original_depth( scene : &Scene, ray : &Ray ) -> Color3 {
  if let Some( h ) = scene.trace( ray ) {
    let v = 1.0 - math::clamp( ( h.distance - 5.0 ) / 12.0, 0.0, 1.0 );
    Color3::new( v, v, v )
  } else {
    Color3::new( 0.0, 0.0, 0.0 )
  }
}

/// Traces an original ray, and produces a color for that ray
pub fn trace_original_color( scene : &Scene, ray : &Ray, max_rays : u32, refr_stack : &mut Stack< MatRefract > ) -> Color3 {
  let (_, c) = trace_color( scene, ray, max_rays, refr_stack );
  c
}

/// Traces a ray into the scene, and returns both its distance and color to the
/// first hit. If no hit found it returns a distance of 0 and the color black.
/// This should be of no effect anyway.
fn trace_color( scene : &Scene, ray : &Ray, max_rays : u32, refr_stack : &mut Stack< MatRefract > ) -> (f32, Color3) {
  if let Some( h ) = scene.trace( ray ) {
    let hit_loc = ray.at( h.distance );

    let color =
      match h.mat {
        PointMaterial::Reflect { color, reflection } => {
          let light_color = lights_color( scene, &hit_loc, &h.normal );

          if max_rays > 0 && reflection > 0.0 {
            let refl_dir          = (-ray.dir).reflect( h.normal );
            let refl_ray          = Ray::new( hit_loc + math::EPSILON * refl_dir, refl_dir );
            let (_, refl_diffuse) = trace_color( scene, &refl_ray, max_rays - 1, refr_stack );
            let diffuse_color     = reflection * refl_diffuse + ( 1.0 - reflection ) * color;
            light_color * diffuse_color
          } else { // If it's at the cap, just apply direct illumination
            light_color * color
          }
        },

        PointMaterial::Refract { absorption, refractive_index } => {
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
                  let (d,c) = trace_color( scene, &refr_ray, max_rays - 1, refr_stack );
                  refr_stack.pop_until1( );
                  ( kr, c * ( -absorption * d ).exp( ) )
                } else { // leaving the object
                  // Note that in this case the material was popped before, and is pushed after
                  // Which is done externally
                  let (d,c) = trace_color( scene, &refr_ray, max_rays - 1, refr_stack );

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
              let (_, c) = trace_color( scene, &refl_ray, max_rays - 1, refr_stack );
              c
            } else {
              // This means very little, but happens when the rays don't want to
              // go any further. Instead of black, choose a sensible color
              let habs = absorption.x.max( absorption.y ).max( absorption.z );
              Color3::new( 1.0 - absorption.x / habs, 1.0 - absorption.y / habs, 1.0 - absorption.z / habs )
            };

          refl_color * kr + refr_color * ( 1.0 - kr )
        }
      };

    ( h.distance, color )
  } else {
    ( 0.0, scene.background )
  }
}

/// The sum of the contribution of all lights in the scene toward the `hit_loc`
/// This takes into account:
/// * Occlusion (shadow-rays; occluded sources do not contribute)
/// * Distance
/// * Angle of hit
fn lights_color( scene : &Scene, hit_loc : &Vec3, hit_normal : &Vec3 ) -> Vec3 {
  let mut light_color = Vec3::ZERO;
  for l_id in 0..scene.lights.len( ) {
    if let Some( light_hit ) = scene.shadow_ray( &hit_loc, l_id ) {
      // Note that `scene.shadow_ray(..)` already performs distance attenuation
      light_color += light_hit.color * 0.0_f32.max( hit_normal.dot( light_hit.dir ) );
    }
  }
  light_color
}

/// Returns the amount (in range (0,1)) of reflection, and the angle of refraction
/// If None is returned, total internal reflection applies (and no refraction at all)
/// This applies both fresnel and Snell's law for refraction
fn refract_fresnel( i : Vec3, mut n : Vec3, prev_ior : f32, ior : f32 ) -> Option< ( f32, Vec3 ) > {
  // Borrowed and adjusted from:
  // https://www.scratchapixel.com/lessons/3d-basic-rendering/introduction-to-shading/reflection-refraction-fresnel

  let mut cosi = math::clamp( i.dot( n ), -1.0, 1.0 );
  let mut etai = prev_ior;
  let mut etat = ior;
  if cosi < 0.0 {
    cosi = -cosi;
  } else { // Inside the object
    std::mem::swap( &mut etai, &mut etat );
    n = -n;
  }
  let eta = etai / etat;
  // Compute sint using Snell's law
  let sint = eta * 0.0_f32.max( 1.0 - cosi * cosi ).sqrt( );
  // Total internal reflection
  if sint >= 1.0 {
    None // So, reflection = 1.0
  } else {
    // Because sint < 1.0, k > 0.0
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    // assert!( k > 0.0 );

    let cost = 0.0_f32.max( 1.0 - sint * sint ).sqrt( );
    let rs = ((etat * cosi) - (etai * cost)) / ((etat * cosi) + (etai * cost));
    let rp = ((etai * cosi) - (etat * cost)) / ((etai * cosi) + (etat * cost));
    let frac_refl = (rs * rs + rp * rp) / 2.0;
    let refr_dir  = ( eta * i + (eta * cosi - k.sqrt()) * n ).normalize( );

    Some( ( frac_refl, refr_dir ) )
  }
}
