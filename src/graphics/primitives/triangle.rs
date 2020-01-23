// Local imports
use crate::math::{Vec2, Vec3, EPSILON};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::graphics::AABB;
use crate::rng::Rng;

/// A triangle in 3-dimensional space
/// It's normal is inferred from the plane between the vertices
#[derive(Debug, Clone)]
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

  fn normal( &self ) -> Vec3 {
    let v0 = self.v0;
    let v1 = self.v1;
    let v2 = self.v2;

    ( v1 - v0 ).cross( v2 - v0 )
  }
}

// Returns true if P is on the left of line v1-v0 which has normal N
// This function is necessary to ensure no gaps (T-junctions) occur between adjacent triangles.
fn is_approx_left_of( v0 : Vec3, v1 : Vec3, n : Vec3, p : Vec3 ) -> bool {
  let edge = v1 - v0;
  let v0p = p - v0;
  return n.dot( edge.cross( v0p ) ) + 0.1 * EPSILON >= 0.0;
}

impl Bounded for Triangle {
  fn aabb( &self ) -> Option< AABB > {
    let x_min = self.v0.x.min( self.v1.x ).min( self.v2.x );
    let y_min = self.v0.y.min( self.v1.y ).min( self.v2.y );
    let z_min = self.v0.z.min( self.v1.z ).min( self.v2.z );

    let x_max = self.v0.x.max( self.v1.x ).max( self.v2.x );
    let y_max = self.v0.y.max( self.v1.y ).max( self.v2.y );
    let z_max = self.v0.z.max( self.v1.z ).max( self.v2.z );

    Some( AABB::new1(
        x_min - 0.1 * EPSILON
      , y_min - 0.1 * EPSILON
      , z_min - 0.1 * EPSILON
      , x_max + 0.1 * EPSILON
      , y_max + 0.1 * EPSILON
      , z_max + 0.1 * EPSILON
      )
    )
  }
}

impl Tracable for Triangle {
  fn is_emissive( &self ) -> bool {
    self.mat.is_emissive( )
  }
  
  fn project_area_sphere( &self, p : &Vec3 ) -> f32 {
    // Project the triangle on the hemisphere of p
    let p0 = ( *p - self.v0 ).normalize( );
    let p1 = ( *p - self.v1 ).normalize( );
    let p2 = ( *p - self.v2 ).normalize( );

    // Heron's formula
    // This introduces a bias, but apparently that is acceptable
    let a = p0.dis( p1 );
    let b = p1.dis( p2 );
    let c = p2.dis( p0 );

    let s = ( a + b + c ) * 0.5;
    ( s * ( s - a ) * ( s - b ) * ( s - c ) ).sqrt( )
  }

  fn pick_random( &self, rng : &mut Rng, p : &Vec3 ) -> Vec3 {
    let p0 = ( *p - self.v0 ).normalize( );
    let p1 = ( *p - self.v1 ).normalize( );
    let p2 = ( *p - self.v2 ).normalize( );

    let u = rng.next( );
    let v = rng.next( ) * ( 1.0 - u );

    let sphere_hit = ( 1.0 - u - v ) * p0 + u * p1 + v * p2;

    ( sphere_hit - *p ).normalize( )
  }
  
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
}
