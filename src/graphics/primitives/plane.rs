use crate::math::{Vec2, Vec3};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::graphics::AABB;

/// An infinite plane in 3d
///
/// It is characterised by a location and normal
#[derive(Debug, Clone)]
pub struct Plane {
  location : Vec3,
  normal   : Vec3,
  mat      : Material
}

impl Plane {
  pub fn new( location : Vec3, normal : Vec3, mat : Material ) -> Plane {
    Plane { location, normal, mat }
  }
}

impl Bounded for Plane {
  /// See `Plane::location()`
  fn location( &self ) -> Option< Vec3 > {
    // Planes are infinite, and thus have no location
    None
  }

  /// See `Plane::aabb()`
  fn aabb( &self ) -> Option< AABB > {
    // Planes are infinite, and thus have no AABB
    None
  }
}

impl Tracable for Plane {
  /// See `Tracable::is_emissive()`
  fn is_emissive( &self ) -> bool {
    self.mat.is_emissive( )
  }
  
  /// See `Tracable::trace()`
  /// Copied and adjusted from BSc ray-tracer:
  /// https://github.com/dennis-school/raytrace_city/blob/master/Code/shapes/plane.cpp
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    let mut normal = self.normal;
    let n_dot_dir = normal.dot( ray.dir );

    if n_dot_dir == 0.0 {
      // The normal is orthogonal to the ray, so no hit
      return None;
    }

    let o_distance = normal.dot( self.location );

    let t = ( o_distance - normal.dot( ray.origin ) ) / n_dot_dir;

    if t <= 0.0 {
      // The triangle is behind the ray's origin (or equal to)
      return None;
    }

    if n_dot_dir > 0.0 {
      // Pick the normal that points towards the ray origin, so that it is visible from both sides
      normal = -normal;
    }
    
    let mat =
      if let Some( v ) = self.mat.evaluate_simple( ) {
        v
      } else {
        // TODO: UV mapping
        self.mat.evaluate_at( &Vec2::ZERO )
      };
    
    Some( Hit::new( t, normal, mat, true ) )
  }
  
  /// See `Tracable::trace_simple()`
  fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
    let normal = self.normal;
    let n_dot_dir = normal.dot( ray.dir );

    if n_dot_dir == 0.0 {
      // The normal is orthogonal to the ray, so no hit
      return None;
    }

    let o_distance = normal.dot( self.location );

    let t = ( o_distance - normal.dot( ray.origin ) ) / n_dot_dir;

    if t <= 0.0 {
      // The triangle is behind the ray's origin (or equal to)
      None
    } else {
      Some( t )
    }
  }
}
