use crate::vec3::Vec3;
use crate::ray::{Ray, Hit};

pub fn hit_sphere( pos : Vec3, r : f64, ray : Ray ) -> Option< Hit > {
  // Using algebraic solution. (Non-geometric)
  // Solve: ((O-P)+D*t)^2 - R^2
  let a = 1_f64; // D^2
  let b = 2_f64 * ray.dir.dot( ray.origin - pos );
  let c = ( ray.origin - pos ).dot( ray.origin - pos ) - r*r;
  let D = b * b - 4_f64 * a * c;

  if D < 0_f64 {
    return None;
  }

  let t0 = ( -b + D.sqrt( ) ) / ( 2_f64 * a );
  let t1 = ( -b - D.sqrt( ) ) / ( 2_f64 * a );

  let mut t = t0.min( t1 );
  if t <= 0_f64 {
    t = t0.max( t1 );

    if t <= 0_f64 { // The sphere is fully behind the "camera"
      return None
    }
  }

  let normal = ( ray.at( t ) - pos ) / r;

  return Some( Hit::new( t, normal ) );
}
