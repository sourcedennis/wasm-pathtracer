use crate::math::vec3::{Vec3};
use crate::graphics::material::Material;
use crate::graphics::ray::{Ray, Tracable, Hit};

/// A Sphere primitive
pub struct Sphere {
  /// The center of the sphere
  location : Vec3,
  radius   : f32,
  mat      : Material
}

impl Sphere {
  pub fn new( location : Vec3, radius : f32, mat : Material ) -> Sphere {
    Sphere { location, radius, mat }
  }
}

// Finds the closest intersection with the sphere along the ray
// If the ray's origin is inside the sphere, the resulting normal
// also points otherwise. Otherwise the normal points outward.
impl Tracable for Sphere {
  fn trace( &self, ray : &Ray ) -> Option< Hit > {
    // Copied and adjusted from BSc ray-tracer:
    // https://github.com/dennis-school/raytrace_city/blob/master/Code/shapes/sphere.cpp

    // Using algebraic solution. (Non-geometric)
    // Solve: ((O-P)+D*t)^2 - R^2
    let a = 1_f32; // D^2
    let b = 2_f32 * ray.dir.dot( ray.origin - self.location );
    let c = ( ray.origin - self.location ).dot( ray.origin - self.location ) - self.radius*self.radius;
    let d = b * b - 4_f32 * a * c;

    if d < 0_f32 { // There is no intersection
      return None;
    }

    // Find both sphere intersections
    let d_sqrt = d.sqrt( );
    let t0 = ( -b + d_sqrt ) / ( 2_f32 * a );
    let t1 = ( -b - d_sqrt ) / ( 2_f32 * a );

    let mut t = t0.min( t1 );
    let mut is_entering = true;
    if t <= 0_f32 {
      t = t0.max( t1 );

      if t <= 0_f32 { // The sphere is fully behind the "camera"
        return None
      } else { // The camera is inside the sphere
        is_entering = false;
      }
    }

    // Computing this normal is cheap, so do it here
    let normal =
      if is_entering {
        ( ray.at( t ) - self.location ) / self.radius
      } else {
        -( ray.at( t ) - self.location ) / self.radius
      };

    return Some( Hit::new( t, normal, self.mat, is_entering ) );
  }
}
