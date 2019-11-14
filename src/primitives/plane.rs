use crate::vec3::Vec3;
use crate::ray::{Ray, Hit};

pub fn hit_plane( location : Vec3, mut normal : Vec3, ray : Ray ) -> Option< Hit > {
  normal = normal.normalize( );

  let n_dot_dir = normal.dot( ray.dir );

  if ( n_dot_dir == 0.0 ) {
    // The normal is orthogonal to the ray, so no hit
    return None;
  }

  let o_distance = normal.dot( location );

  let t = ( o_distance - normal.dot( ray.origin ) ) / n_dot_dir;

  if ( t <= 0.0 ) {
    // The triangle is behind the ray's origin (or equal to)
    return None;
  }

  if ( n_dot_dir > 0.0 ) {
    // Pick the normal that points towards the ray origin, so that it is visible from both sides
    normal = -normal;
  }

  return Some( Hit::new( t, normal ) );
}
