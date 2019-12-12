use crate::math::{Vec2, Vec3, EPSILON};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Hit};
use crate::graphics::AABB;

/// A triangle in 3-dimensional space
/// It's normal is inferred from the plane between the vertices
#[derive(Debug)]
pub struct Triangle {
  v0  : Vec3,
  v1  : Vec3,
  v2  : Vec3,
  mat : Material
}

impl Triangle {
  pub fn new( v0 : Vec3, v1 : Vec3, v2 : Vec3, mat : Material ) -> Triangle {
    Triangle { v0, v1, v2, mat }
  }

  pub fn translate( self, v : Vec3 ) -> Triangle {
    Triangle::new( self.v0 + v, self.v1 + v, self.v2 + v, self.mat )
  }
}

// Returns true if P is on the left of line v1-v0 which has normal N
// This function is necessary to ensure no gaps (T-junctions) occur between adjacent triangles.
fn is_approx_left_of( v0 : Vec3, v1 : Vec3, n : Vec3, p : Vec3 ) -> bool {
  let edge = v1 - v0;
  let v0p = p - v0;
  return n.dot( edge.cross( v0p ) ) + EPSILON >= 0.0;
}

impl Tracable for Triangle {
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    let v0 = self.v0;
    let v1 = self.v1;
    let v2 = self.v2;

    let mut n = ( v1 - v0 ).cross( v2 - v0 );

    let n_dot_d = n.dot( ray.dir );
    if n_dot_d == 0.0 {
      // The normal is orthogonal to the ray, meaning the triangle's plane does not intersect with the ray
      return None;
    }

    let orig_dis = n.dot( v0 );

    let t = ( orig_dis - n.dot( ray.origin ) ) / n_dot_d;

    if t <= 0.0 {
      // The triangle is behind the ray's origin
      return None;
    }

    n = n.normalize( );
    let p = ray.at( t );

    if is_approx_left_of( v0, v1, n, p ) && is_approx_left_of( v1, v2, n, p ) && is_approx_left_of( v2, v0, n, p ) {
      let mat =
        if let Some( v ) = self.mat.evaluate_simple( ) {
          v
        } else {
          // TODO: UV mapping
          self.mat.evaluate_at( &Vec2::ZERO )
        };
      if n_dot_d > 0.0 { // Looking at the back-side
        Some( Hit::new( t, -n, mat, false ) )
      } else { // Front side
        Some( Hit::new( t, n, mat, true ) )
      }
    } else {
      None
    }
  }
  
  fn trace_simple( &self, ray: &Ray ) -> Option< f32 > {
    let v0 = self.v0;
    let v1 = self.v1;
    let v2 = self.v2;

    let mut n = ( v1 - v0 ).cross( v2 - v0 );

    let n_dot_d = n.dot( ray.dir );
    if n_dot_d == 0.0 {
      // The normal is orthogonal to the ray, meaning the triangle's plane does not intersect with the ray
      return None;
    }

    let orig_dis = n.dot( v0 );

    let t = ( orig_dis - n.dot( ray.origin ) ) / n_dot_d;

    if t <= 0.0 {
      // The triangle is behind the ray's origin
      return None;
    }

    // Somewhat necessary to avoid edge-case errors
    //   (as `trace_simple(..).is_none() == trace(..).is_none()` should hold)
    n = n.normalize( );
    let p = ray.at( t );

    if is_approx_left_of( v0, v1, n, p ) && is_approx_left_of( v1, v2, n, p ) && is_approx_left_of( v2, v0, n, p ) {
      Some( t )
    } else {
      None
    }
  }

  fn location( &self ) -> Option< Vec3 > {
    if let Some( b ) = self.aabb( ) {
      Some( b.center( ) )
    } else {
      None
    }
  }

  fn aabb( &self ) -> Option< AABB > {
    let mut x_min = self.v0.x.min( self.v1.x ).min( self.v2.x );
    let mut y_min = self.v0.y.min( self.v1.y ).min( self.v2.y );
    let mut z_min = self.v0.z.min( self.v1.z ).min( self.v2.z );

    let mut x_max = self.v0.x.max( self.v1.x ).max( self.v2.x );
    let mut y_max = self.v0.y.max( self.v1.y ).max( self.v2.y );
    let mut z_max = self.v0.z.max( self.v1.z ).max( self.v2.z );

    Some( AABB::new1(
        x_min
      , y_min
      , z_min
      , x_max
      , y_max
      , z_max
      )
    )
  }
}
