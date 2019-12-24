use crate::graphics::ray::{Ray};
use crate::math::Vec3;
use std::f32::INFINITY;
use packed_simd::*;

/// An Axis-Aligned bounding box
/// Fast intersection with their distance is available
#[derive(Copy,Clone,Debug)]
pub struct AABB {
  pub x_min : f32,
  pub y_min : f32,
  pub z_min : f32,
  pub x_max : f32,
  pub y_max : f32,
  pub z_max : f32
}

#[derive(Copy,Clone,Debug)]
pub struct AABBx4 {
  pub x_min : f32x4,
  pub y_min : f32x4,
  pub z_min : f32x4,
  pub x_max : f32x4,
  pub y_max : f32x4,
  pub z_max : f32x4
}

impl AABB {
  pub const EMPTY: AABB =
    AABB {
      x_min: 0.0
    , y_min: 0.0
    , z_min: 0.0
    , x_max: 0.0
    , y_max: 0.0
    , z_max: 0.0
    };

  // pub fn new( x : f32, y : f32, z : f32, x_size : f32, y_size : f32, z_size : f32 ) -> AABB {
  //   AABB { x, y, z, x_size, y_size, z_size }
  // }

  pub fn new1( x_min : f32, y_min : f32, z_min : f32
             , x_max : f32, y_max : f32, z_max : f32
             ) -> AABB {
    AABB { x_min, y_min, z_min, x_max, y_max, z_max }
  }

  /*pub fn area( &self ) -> f32 {
    let x_size : f32 = self.x_max - self.x_min;
    let y_size : f32 = self.y_max - self.y_min;
    let z_size : f32 = self.z_max - self.z_min;

    x_size * y_size * z_size
  }*/

  pub fn surface( &self ) -> f32 {
    let x_size = self.x_max - self.x_min;
    let y_size = self.y_max - self.y_min;
    let z_size = self.z_max - self.z_min;
    
    2.0 * ( x_size * y_size + x_size * z_size + y_size * z_size )
  }

  pub fn center( &self ) -> Vec3 {
    Vec3::new(
      0.5 * ( self.x_min + self.x_max )
    , 0.5 * ( self.y_min + self.y_max )
    , 0.5 * ( self.z_min + self.z_max )
    )
  }

  pub fn join( &self, o : &AABB ) -> AABB {
    let x_min = self.x_min.min( o.x_min );
    let y_min = self.y_min.min( o.y_min );
    let z_min = self.z_min.min( o.z_min );
    
    let x_max = self.x_max.max( o.x_max );
    let y_max = self.y_max.max( o.y_max );
    let z_max = self.z_max.max( o.z_max );

    AABB::new1( x_min, y_min, z_min, x_max, y_max, z_max )
  }

  pub fn join_maybe( &self, o : &Option< AABB > ) -> AABB {
    if let Some( a ) = o {
      self.join( a )
    } else {
      *self
    }
  }

  pub fn contains( &self, o : &AABB ) -> bool {
    o.x_min >= self.x_min
      && o.y_min >= self.y_min
      && o.z_min >= self.z_min
      && o.x_max <= self.x_max
      && o.y_max <= self.y_max
      && o.z_max <= self.z_max
  }

  pub fn hit( &self, ray : &Ray ) -> Option< f32 > {
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

    if tmin > tmax { // Does not intersect
      None
    } else if tmin >= 0.0 { // Outside the box
      Some( tmin )
    } else if tmax >= 0.0 { // Inside the box
      Some( 0.0 )
    } else { // Box behind camera
      None
    }
  }

  pub fn hit_furthest( &self, ray : &Ray ) -> Option< f32 > {
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

    if tmin > tmax { // Does not intersect
      None
    } else if tmax >= 0.0 { // Inside the box
      Some( tmax )
    } else { // Box behind camera
      None
    }
  }
}

impl AABBx4 {
  pub fn empty( ) -> AABBx4 {
    AABBx4::new( AABB::EMPTY, AABB::EMPTY, AABB::EMPTY, AABB::EMPTY )
  }

  pub fn extract( &self, i : usize ) -> AABB {
    AABB::new1( self.x_min.extract( i ), self.y_min.extract( i ), self.z_min.extract( i )
              , self.x_max.extract( i ), self.y_max.extract( i ), self.z_max.extract( i )
              )
  }

  pub fn extract_hull( &self, n : usize ) -> AABB {
    // assert( n > 0 )
    let mut hull = self.extract( 0 );
    for i in 1..n {
      hull = hull.join( &self.extract( i ) );
    }
    hull
  }

  pub fn new( a : AABB, b : AABB, c : AABB, d : AABB ) -> AABBx4 {
    let x_min = f32x4::new( a.x_min, b.x_min, c.x_min, d.x_min );
    let y_min = f32x4::new( a.y_min, b.y_min, c.y_min, d.y_min );
    let z_min = f32x4::new( a.z_min, b.z_min, c.z_min, d.z_min );
    let x_max = f32x4::new( a.x_max, b.x_max, c.x_max, d.x_max );
    let y_max = f32x4::new( a.y_max, b.y_max, c.y_max, d.y_max );
    let z_max = f32x4::new( a.z_max, b.z_max, c.z_max, d.z_max );

    AABBx4 { x_min, y_min, z_min, x_max, y_max, z_max }
  }

  pub fn hit( &self, ray : &Ray ) -> f32x4 {
    let mut z_x4 = f32x4::splat( 0.0 );
    let mut ninf_x4 = f32x4::splat( -INFINITY );

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

    let no_intersect = tmin.gt( tmax );
    let outside = tmin.ge( z_x4 );
    let inside = tmax.ge( z_x4 );

    no_intersect.select(
      ninf_x4,
      outside.select(
        tmin,
        inside.select(
          z_x4,
          ninf_x4
        )
      )
    )
  }
}
