use crate::graphics::ray::{Ray};

pub struct AABB {
  pub x      : f32,
  pub y      : f32,
  pub z      : f32,
  pub x_size : f32,
  pub y_size : f32,
  pub z_size : f32
}

impl AABB {
  pub fn new( x : f32, y : f32, z : f32, x_size : f32, y_size : f32, z_size : f32 ) -> AABB {
    AABB { x, y, z, x_size, y_size, z_size }
  }

  pub fn hit( &self, ray : &Ray ) -> Option< f32 > {
    let invdx = 1.0 / ray.dir.x;
    let invdy = 1.0 / ray.dir.y;
    let invdz = 1.0 / ray.dir.z;

    let x_min = self.x;
    let y_min = self.y;
    let z_min = self.z;
    let x_max = x_min + self.x_size;
    let y_max = y_min + self.y_size;
    let z_max = z_min + self.z_size;

    // "Clip" the line within the box, along each axis
    let tx1 = ( x_min - ray.origin.x ) * invdx;
    let tx2 = ( x_max - ray.origin.x ) * invdx;
    let ty1 = ( y_min - ray.origin.y ) * invdy;
    let ty2 = ( y_max - ray.origin.y ) * invdy;
    let tz1 = ( z_min - ray.origin.z ) * invdz;
    let tz2 = ( z_max - ray.origin.z ) * invdz;

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
      Some( 0.0 )
    } else { // Box behind camera
      None
    }
  }
}
