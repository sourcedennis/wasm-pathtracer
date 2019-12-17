use crate::graphics::{MarchScene, Color3};
use crate::graphics::ray::{Ray, Marchable};
use crate::math::{EPSILON, Vec3};
use crate::math;

pub fn march_original_color( scene : &MarchScene, ray : &Ray ) -> Color3 {
  if let Some( ( d, h ) ) = scene.march( ray, None ) {
    let p = ray.at( d );
    let normal = march_normal( h, &p );
    // Apply the bias here, along the normal. And 2 * EPSILON.
    // Otherwise `p` is identified as the closest SDF again
    let p_bias = p + normal * ( 2.0 * EPSILON );
    let light_color = lights_color( scene, &p_bias, &normal );

    light_color * h.color( &p )
  } else {
    scene.background
  }
}

pub fn march_original_depth( scene : &MarchScene, ray : &Ray ) -> Color3 {
  if let Some( ( d, _h ) ) = scene.march( ray, None ) {
    let v = 1.0 - math::clamp( ( d - 5.0 ) / 4.0, 0.0, 1.0 );
    Color3::new( v, v, v )
  } else {
    Color3::new( 0.0, 0.0, 0.0 )
  }
}

fn march_normal( s : &std::rc::Rc<dyn Marchable>, p : &Vec3 ) -> Vec3 {
  let e = EPSILON;
  ( Vec3::new(e,-e,-e)*s.sdf( &( *p + Vec3::new(e,-e,-e) ) ) + 
    Vec3::new(-e,-e,e)*s.sdf( &( *p + Vec3::new(-e,-e,e) ) ) + 
    Vec3::new(-e,e,-e)*s.sdf( &( *p + Vec3::new(-e,e,-e) ) ) + 
    Vec3::new(e,e,e)  *s.sdf( &( *p + Vec3::new(e,e,e) ) ) ).normalize( )
}

/// The sum of the contribution of all lights in the scene toward the `hit_loc`
/// This takes into account:
/// * Occlusion (shadow-rays; occluded sources do not contribute)
/// * Distance
/// * Angle of hit
fn lights_color( scene : &MarchScene, hit_loc : &Vec3, hit_normal : &Vec3 ) -> Vec3 {
  let mut light_color = Vec3::ZERO;
  for l_id in 0..scene.lights.len( ) {
    if let Some( light_hit ) = scene.shadow_ray( &hit_loc, l_id ) {
      let attenuation =
        if let Some( dis_sq ) = light_hit.distance_sq {
          1.0 / dis_sq
        } else {
          1.0
        };
      light_color += light_hit.color * attenuation * 0.0_f32.max( hit_normal.dot( light_hit.dir ) );
    }
  }
  light_color
}
