use crate::material::{Material, Color3};
use crate::ray::{Ray, Hit};
use crate::vec3::{Vec3};
use crate::math::EPSILON;

pub struct Scene {
  lights : Vec< Light >,
  shapes : Vec< Box< Tracable > >
}

pub struct LightHit {
  pub dir      : Vec3,
  pub distance : f32,
  pub color    : Color3,
}

impl Scene {
  pub fn new( lights : Vec< Light >, shapes : Vec< Box< Tracable > > ) -> Scene {
    Scene { lights, shapes }
  }

  // The vector of lights that can reach the location
  pub fn lights_at( &self, hit_loc: &Vec3 ) -> Vec< LightHit > {
    let mut lights = Vec::new( );

    for l in &self.lights {
      let mut to_light = l.location - *hit_loc;
      let distance = to_light.len( );
      to_light = to_light / distance;

      let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
      if !is_hit_within_sq( self.trace( &shadow_ray ), ( l.location - *hit_loc ).len_sq( ) ) {
        lights.push( LightHit { dir: to_light, distance, color: l.color } );
      }
    }

    lights
  }
}

fn is_hit_within_sq( m_hit : Option< Hit >, d_sq : f32 ) -> bool {
  if let Some( h ) = m_hit {
    h.distance * h.distance < d_sq
  } else {
    false
  }
}

pub struct Light {
  location : Vec3,
  color    : Color3
}

impl Light {
  pub fn new( location : Vec3, color : Color3 ) -> Light {
    Light { location, color }
  }
}

// Trace a single ray into the object
pub trait Tracable {
  fn trace( &self, ray : &Ray ) -> Option< Hit >;
}

pub struct Sphere {
  location : Vec3,
  radius   : f32,
  mat      : Material
}

pub struct Plane {
  location : Vec3,
  normal   : Vec3,
  mat      : Material
}

pub struct AABB {
  x_min : f32,
  x_max : f32,
  y_min : f32,
  y_max : f32,
  z_min : f32,
  z_max : f32,
  mat   : Material
}

impl Sphere {
  pub fn new( location : Vec3, radius : f32, mat : Material ) -> Sphere {
    Sphere { location, radius, mat }
  }
}

impl Plane {
  pub fn new( location : Vec3, normal : Vec3, mat : Material ) -> Plane {
    Plane { location, normal, mat }
  }
}

impl AABB {
  pub fn new( x_min : f32, x_max : f32, y_min : f32, y_max : f32, z_min : f32, z_max : f32, mat : Material ) -> AABB {
    AABB { x_min, x_max, y_min, y_max, z_min, z_max, mat }
  }

  pub fn cube( center : Vec3, radius : f32, mat : Material ) -> AABB {
    AABB::new(
      center.x - radius
    , center.x + radius
    , center.y - radius
    , center.y + radius
    , center.z - radius
    , center.z + radius
    , mat
    )
  }
}

impl Tracable for Scene {
  fn trace( &self, ray : &Ray ) -> Option< Hit > {
    let mut best_hit: Option< Hit > = None;

    for s in &self.shapes {
      let new_hit: Option< Hit > = s.trace( ray );

      if let Some( nh ) = new_hit {
        if let Some( bh ) = best_hit {
          if nh.distance < bh.distance {
            best_hit = new_hit;
          }
        } else {
          best_hit = new_hit;
        }
      }
    }

    best_hit
  }
}

impl Tracable for Sphere {
  fn trace( &self, ray : &Ray ) -> Option< Hit > {
    // Using algebraic solution. (Non-geometric)
    // Solve: ((O-P)+D*t)^2 - R^2
    let a = 1_f32; // D^2
    let b = 2_f32 * ray.dir.dot( ray.origin - self.location );
    let c = ( ray.origin - self.location ).dot( ray.origin - self.location ) - self.radius*self.radius;
    let D = b * b - 4_f32 * a * c;
    let mut is_entering = true;
  
    if D < 0_f32 {
      return None;
    }
  
    let t0 = ( -b + D.sqrt( ) ) / ( 2_f32 * a );
    let t1 = ( -b - D.sqrt( ) ) / ( 2_f32 * a );
  
    let mut t = t0.min( t1 );
    if t <= 0_f32 {
      t = t0.max( t1 );
  
      if t <= 0_f32 { // The sphere is fully behind the "camera"
        return None
      } else { // The camera is inside the sphere
        is_entering = false;
      }
    }
  
    let normal = ( ray.at( t ) - self.location ) / self.radius;
  
    return Some( Hit::new( t, normal, self.mat, is_entering ) );
  }
}

impl Tracable for Plane {
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

    return Some( Hit::new( t, normal, self.mat, true ) );
  }
}

impl Tracable for AABB {
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

    if tmin >= tmax { // Does not intersect
      None
    } else if tmin > 0.0 {
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
      Some( Hit::new( tmin, normal, self.mat, true ) )
    } else if tmax > 0.0 {
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
      Some( Hit::new( tmax, normal, self.mat, false ) )
    } else {
      None
    }
  }
}
