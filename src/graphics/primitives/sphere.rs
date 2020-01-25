// External imports
use std::f32::consts::PI;
// Local imports
use crate::math::{Vec2, Vec3};
use crate::graphics::{Material, AABB};
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::rng::Rng;

/// A Sphere primitive
#[derive(Debug, Clone)]
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

impl Bounded for Sphere {
  fn location( &self ) -> Option< Vec3 > {
    Some( self.location )
  }

  fn aabb( &self ) -> Option< AABB > {
    let l = self.location;
    let r = self.radius;

    Some( AABB::new1( l.x - r, l.y - r, l.z - r, l.x + r, l.y + r, l.z + r ) )
  }
}

// Finds the closest intersection with the sphere along the ray
// If the ray's origin is inside the sphere, the resulting normal
// also points otherwise. Otherwise the normal points outward.
impl Tracable for Sphere {
  fn pick_random( &self, rng : &mut Rng ) -> (Vec3, Vec3, Vec3) {
    panic!( "Not implemented" );
  }
  
  fn is_emissive( &self ) -> bool {
    self.mat.is_emissive( )
  }
  
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
    let mut normal = ( ray.at( t ) - self.location ) / self.radius;

    let mat =
      if let Some( v ) = self.mat.evaluate_simple( ) {
        v
      } else {
        let u = 0.5 + normal.z.atan2( normal.x ) / ( 2.0 * PI );
        let v = 0.5 - normal.y.asin( ) / PI;
        self.mat.evaluate_at( &Vec2::new( u, v ) )
      };

    normal =
      if is_entering {
        normal
      } else {
        -normal
      };
    
    Some( Hit::new( t, normal, mat, is_entering ) )
  }
  
  fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
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
    if t <= 0_f32 {
      t = t0.max( t1 );

      if t <= 0_f32 { // The sphere is fully behind the "camera"
        return None
      }
    }

    Some( t )
  }
}
