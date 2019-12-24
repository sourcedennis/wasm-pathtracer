use crate::math::Vec3;
use crate::graphics::{PointMaterial, AABB, Color3};
use std::fmt;

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
  pub origin  : Vec3,
  pub dir     : Vec3,
  pub inv_dir : Vec3
}

impl Ray {
  /// Constructs a new `Ray`
  /// The direction should be of unit length
  pub fn new( origin : Vec3, dir : Vec3 ) -> Ray {
    Ray { origin, dir, inv_dir: Vec3::new( 1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z ) }
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

pub trait Bounded : fmt::Debug {
  fn location( &self ) -> Option< Vec3 > {
    if let Some( b ) = self.aabb( ) {
      Some( b.center( ) )
    } else {
      None
    }
  }

  // Returns None if the primitive has no bounding-box, which happens when it is
  // infinite. (Such as planes)
  fn aabb( &self ) -> Option< AABB >;
}

/// A trait for physical objects, with which a ray of light can be intersected
pub trait Tracable : Bounded {
  /// Traces a ray with limited properties evaluated at the hit.
  /// That is, no normal or materials are included. Only its distance from the
  ///   ray origin.
  fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
    if let Some( h ) = self.trace( ray ) {
      Some( h.distance )
    } else {
      None
    }
  }

  /// Traces a ray. At the hit point the normal and material are evaluated and
  ///   included in the returned hit.
  fn trace( &self, ray : &Ray ) -> Option< Hit >;
}

pub trait Marchable : Bounded {
  fn sdf( &self, p : &Vec3 ) -> f32;

  fn color( &self, p : &Vec3 ) -> Color3;
}
