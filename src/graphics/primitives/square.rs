use crate::math::{Vec2, Vec3};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Hit};
use crate::graphics::AABB;

/// A finite square plane in 3d
/// For now, its normal always points upward
///
/// It is characterised by a location and size
pub struct Square {
  location : Vec3,
  // The size along the x- and z-axis
  size     : f32,
  mat      : Material
}

impl Square {
  pub fn new( location : Vec3, size : f32, mat : Material ) -> Square {
    Square { location, size, mat }
  }
}

impl Tracable for Square {
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    let n_dot_dir = ray.dir.y;

    if n_dot_dir == 0.0 {
      // The normal is orthogonal to the ray, so no hit
      return None;
    }

    let o_distance = self.location.y;

    let t = ( o_distance - ray.origin.y ) / n_dot_dir;

    if t <= 0.0 {
      // The triangle is behind the ray's origin (or equal to)
      return None;
    }
    
    let hit = ray.at( t );
    let dx = ( hit.x - self.location.x ).abs( );
    let dz = ( hit.z - self.location.z ).abs( );

    if 2.0 * dx >= self.size || 2.0 * dz >= self.size {
      return None;
    }

    // Pick the normal that points towards the ray origin, so that it is visible from both sides
    let normal =
      if n_dot_dir > 0.0 {
        Vec3::new( 0.0, -1.0, 0.0 )
      } else {
        Vec3::new( 0.0, 1.0, 0.0 )
      };
    
    let mat =
      if let Some( v ) = self.mat.evaluate_simple( ) {
        v
      } else {
        let u = ( hit.x - self.location.x ) / self.size + 0.5;
        let v = ( hit.z - self.location.z ) / self.size + 0.5;
        self.mat.evaluate_at( &Vec2::new( u, v ) )
      };
    
    Some( Hit::new( t, normal, mat, true ) )
  }

  fn location( &self ) -> Option< Vec3 > {
    Some( self.location )
  }

  fn aabb( &self ) -> Option< AABB > {
    let l = self.location;
    let hsize = self.size * 0.5;

    Some( AABB::new( l.x - hsize, l.y - hsize, l.z - hsize, self.size, self.size, self.size ) )
  }
}
