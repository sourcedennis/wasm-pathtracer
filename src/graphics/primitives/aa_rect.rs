use crate::math::{Vec2, Vec3};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::graphics::AABB;

/// An axis-aligned box
#[derive(Debug, Clone)]
pub struct AARect {
  x_min : f32,
  x_max : f32,
  y_min : f32,
  y_max : f32,
  z_min : f32,
  z_max : f32,
  mat   : Material
}

impl AARect {
  pub fn new( x_min : f32, x_max : f32, y_min : f32, y_max : f32, z_min : f32, z_max : f32, mat : Material ) -> AARect {
    AARect { x_min, x_max, y_min, y_max, z_min, z_max, mat }
  }

  /// Constructs a new AA-box with all equal sides
  ///
  /// It is centered at the provided location, with corners at all eight points
  ///   that have a distance of `half_len` in all 3-dimensions from that point
  pub fn cube( center : Vec3, half_len : f32, mat : Material ) -> AARect {
    AARect::new(
      center.x - half_len
    , center.x + half_len
    , center.y - half_len
    , center.y + half_len
    , center.z - half_len
    , center.z + half_len
    , mat
    )
  }
}

impl Bounded for AARect {
  /// See `Bounded::location()`
  fn location( &self ) -> Option< Vec3 > {
    Some( Vec3::new(
      0.5 * ( self.x_min + self.x_max )
    , 0.5 * ( self.y_min + self.y_max )
    , 0.5 * ( self.z_min + self.z_max )
    ) )
  }

  /// See `Bounded::aabb()`
  fn aabb( &self ) -> Option< AABB > {
    Some( AABB::new1(
        self.x_min
      , self.y_min
      , self.z_min
      , self.x_max
      , self.y_max
      , self.z_max
      )
    )
  }
}

impl Tracable for AARect {
  /// See `Tracable::is_emissive()`
  fn is_emissive( &self ) -> bool {
    self.mat.is_emissive( )
  }

  /// See `Tracable::trace()`
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    let invdx = 1.0 / ray.dir.x;
    let invdy = 1.0 / ray.dir.y;
    let invdz = 1.0 / ray.dir.z;

    // "Clip" the line within the box, along each axis
    let tx1 = ( self.x_min - ray.origin.x ) * invdx;
    let tx2 = ( self.x_max - ray.origin.x ) * invdx;
    let ty1 = ( self.y_min - ray.origin.y ) * invdy;
    let ty2 = ( self.y_max - ray.origin.y ) * invdy;
    let tz1 = ( self.z_min - ray.origin.z ) * invdz;
    let tz2 = ( self.z_max - ray.origin.z ) * invdz;

    let txmin = tx1.min(tx2);
    let tymin = ty1.min(ty2);
    let tzmin = tz1.min(tz2);
    let txmax = tx1.max(tx2);
    let tymax = ty1.max(ty2);
    let tzmax = tz1.max(tz2);

    let tmin = txmin.max(tymin).max(tzmin);
    let tmax = txmax.min(tymax).min(tzmax);

    let mat =
      if let Some( v ) = self.mat.evaluate_simple( ) {
        v
      } else {
        // TODO: UV mapping
        self.mat.evaluate_at( &Vec2::ZERO )
      };

    if tmin >= tmax { // Does not intersect
      None
    } else if tmin > 0.0 { // Outside the box
      let normal =
        if tmin == tx1 {
          Vec3::new( -1.0,  0.0,  0.0 )
        } else if tmin == tx2 {
          Vec3::new(  1.0,  0.0,  0.0 )
        } else if tmin == ty1 {
          Vec3::new(  0.0, -1.0,  0.0 )
        } else if tmin == ty2 {
          Vec3::new(  0.0,  1.0,  0.0 )
        } else if tmin == tz1 {
          Vec3::new(  0.0,  0.0, -1.0 )
        } else {
          Vec3::new(  0.0,  0.0,  1.0 )
        };
      Some( Hit::new( tmin, normal, mat, true ) )
  } else if tmax > 0.0 { // Inside the box
      let normal =
        if tmax == tx1 {
          Vec3::new(  1.0,  0.0,  0.0 )
        } else if tmax == tx2 {
          Vec3::new( -1.0,  0.0,  0.0 )
        } else if tmax == ty1 {
          Vec3::new(  0.0,  1.0,  0.0 )
        } else if tmax == ty2 {
          Vec3::new(  0.0, -1.0,  0.0 )
        } else if tmax == tz1 {
          Vec3::new(  0.0,  0.0,  1.0 )
        } else {
          Vec3::new(  0.0,  0.0, -1.0 )
        };
      Some( Hit::new( tmax, normal, mat, false ) )
    } else {
      None
    }
  }

  /// See `Tracable::trace_simple()`
  fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
    let invdx = 1.0 / ray.dir.x;
    let invdy = 1.0 / ray.dir.y;
    let invdz = 1.0 / ray.dir.z;

    // "Clip" the line within the box, along each axis
    let tx1 = ( self.x_min - ray.origin.x ) * invdx;
    let tx2 = ( self.x_max - ray.origin.x ) * invdx;
    let ty1 = ( self.y_min - ray.origin.y ) * invdy;
    let ty2 = ( self.y_max - ray.origin.y ) * invdy;
    let tz1 = ( self.z_min - ray.origin.z ) * invdz;
    let tz2 = ( self.z_max - ray.origin.z ) * invdz;

    let txmin = tx1.min(tx2);
    let tymin = ty1.min(ty2);
    let tzmin = tz1.min(tz2);
    let txmax = tx1.max(tx2);
    let tymax = ty1.max(ty2);
    let tzmax = tz1.max(tz2);

    let tmin = txmin.max(tymin).max(tzmin);
    let tmax = txmax.min(tymax).min(tzmax);

    if tmin >= tmax { // Does not intersect
      None
    } else if tmin > 0.0 { // Outside the box
      Some( tmin )
    } else if tmax > 0.0 { // Inside the box
      Some( tmax )
    } else { // Box behind camera
      None
    }
  }
}
