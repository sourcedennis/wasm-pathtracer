use crate::math::Vec3;
use crate::graphics::{PointMaterial};

// A module with `Ray` and `Hit` structures, that are useful for raytracing
//
// Exports:
// * Ray
// * Hit
// * Tracable

/// A half-line in 3-dimensional space
///
/// Conceptually, it "shoots" from a origin into a direction
/// The direction should be of unit length
#[derive(Clone,Copy)]
pub struct Ray {
  pub origin : Vec3,
  pub dir    : Vec3
}

impl Ray {
  /// Constructs a new `Ray`
  /// The direction should be of unit length
  pub fn new( origin : Vec3, dir : Vec3 ) -> Ray {
    Ray { origin, dir }
  }

  /// Evaluates the ray at the provided distance from its origin
  pub fn at( self, distance : f32 ) -> Vec3 {
    self.origin + distance * self.dir
  }
}

/// A `Hit` in 3-dimensional space
/// This is typically used as the intersection of a ray with a surface
/// The Hit contains the properties of the intersected surface at the
///   intersection point (e.g. materials)
#[derive(Clone,Copy)]
pub struct Hit {
  /// The distance from the ray origin to the surface intersection
  pub distance    : f32,
  // TODO: Make its computation delayed
  pub normal      : Vec3,
  /// The material of the surface at the intersection point
  pub mat         : PointMaterial,
  /// True if the rays comes from the outside, pointing into the shape
  ///   Defining the "inside" and "outside" of a shape, is the responsibility
  ///   of that particular shape.
  pub is_entering : bool
}

impl Hit {
  /// Constructs a new `Hit` at a distance from its ray origin
  pub fn new( distance : f32, normal : Vec3, mat : PointMaterial, is_entering : bool ) -> Hit {
    Hit { distance, normal, mat, is_entering }
  }
}

/// A trait for physical objects, with which a ray of light can be intersected
pub trait Tracable {
  fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
    if let Some( h ) = self.trace( ray ) {
      Some( h.distance )
    } else {
      None
    }
  }

  fn trace( &self, ray : &Ray ) -> Option< Hit >;
}