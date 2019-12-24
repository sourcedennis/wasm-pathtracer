// External imports
use std::fmt;
// Local imports
use crate::math::Vec3;
use crate::graphics::{PointMaterial, AABB, Color3};

// A module with `Ray` and `Hit` structures, that are useful for raytracing
//
// Exports:
// * Ray
// * Hit
// * Bounded
// * Tracable
// * Marchable

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

/// A shape which has a location and bounds in 3D space
/// Also objects that are knowingly infinite have this trait,
///   but should return `None` for both location and AABB.
/// (So, basically, only immaterial objects lack this trait)
pub trait Bounded : fmt::Debug {
  /// Returns the location of the object in 3D space
  /// 
  /// The location of an object is arbitrary, and is up to the implementer to
  ///   determine. However, it is advised to implement such that mass is
  ///   appropriately divided over the 3 axes.
  /// Thus, the default implementation takes the center of the object's AABB,
  ///   which should reasonably approximate it.
  fn location( &self ) -> Option< Vec3 > {
    if let Some( b ) = self.aabb( ) {
      Some( b.center( ) )
    } else {
      None
    }
  }

  /// Returns None if the primitive has no bounding-box, which happens when it is
  /// infinite. (Such as planes)
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

/// A trait for objects that can be ray-marched
/// 
/// The surface of ray-marched objects is typically difficult to represent.
/// Instead, they are represented by a Signed Distance Function.
pub trait Marchable : Bounded {
  /// The Signed Distance from point `p` to the object
  fn sdf( &self, p : &Vec3 ) -> f32;

  /// The color of the object at point `p`.
  /// 
  /// If `p` is not inside the object, it is advised to return the color of the
  /// surface point closest to `p`.
  fn color( &self, p : &Vec3 ) -> Color3;
}
