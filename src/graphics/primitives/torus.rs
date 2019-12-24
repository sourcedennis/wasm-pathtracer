// External imports
use roots::{find_roots_quartic, Roots, FloatType};
// Local imports
use crate::math::{Vec2, Vec3};
use crate::graphics::Material;
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::graphics::AABB;

// A torus that lies flat; that is, its gap lies in the x/z-plane
#[derive(Debug, Clone)]
pub struct Torus {
  location : Vec3,
  big_r    : f32,
  small_r  : f32,
  mat      : Material
}

impl Torus {
  pub fn new( location : Vec3, big_r : f32, small_r : f32, mat : Material ) -> Torus {
    Torus { location, big_r, small_r, mat }
  }
}

impl Bounded for Torus {
  fn location( &self ) -> Option< Vec3 > {
    Some( self.location )
  }

  fn aabb( &self ) -> Option< AABB > {
    let r = self.big_r + self.small_r;

    let x_min = self.location.x - r;
    let x_max = self.location.x + r;
    let y_min = self.location.y - self.small_r;
    let y_max = self.location.y + self.small_r;
    let z_min = self.location.z - r;
    let z_max = self.location.z + r;

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

impl Tracable for Torus {
  fn trace( &self, ray: &Ray ) -> Option< Hit > {
    // The torus formula is defined as (where A=big_r and B=small_r):
    // (x^2 + y^2 + z^2 + A^2 - B^2) = 4A^2 * (x^2 + y^2)
    // This is then solved for `t`:
    // x = (( ray.origin.x - self.location.x ) + ray.direction.x * t
    // similar for `y` and `z`
    //
    // A nice solution to this formula is described here:
    // http://cosinekitty.com/raytrace/chapter13_torus.html

    // It is necessary to convert all the f32s to f64s to properly test
    // for this intersection. Otherwise it becomes quite a grainy torus,
    // which is ugly.
    // "Grainy tori are ugly." -Dennis

    let a = self.big_r as f64;
    let b = self.small_r as f64;
    let d = ray.origin - self.location;
    let e = ray.dir;

    let dx = d.x as f64;
    let dy = d.y as f64;
    let dz = d.z as f64;

    let ex = e.x as f64;
    let ey = e.y as f64;
    let ez = e.z as f64;

    let g = 4.0*a*a*(ex*ex + ez*ez);
    let h = 8.0*a*a*(dx*ex + dz*ez);
    let i = 4.0*a*a*(dx*dx + dz*dz);
    let j = ex*ex+ey*ey+ez*ez;
    let k = 2.0 * (dx*ex+dy*ey+dz*ez);
    let l = dx*dx+dy*dy+dz*dz + a*a - b*b;

    // Now solve:
    // j^2 * u^4 + 2jku^3 + (2jl+k^2-g)*u^2 + (2kl-h)*u + (l^2-i) = 0

    let roots = find_roots_quartic( j*j, 2.0*j*k, 2.0*j*l+k*k-g, 2.0*k*l-h, l*l-i );
    let mut dst_roots: [f64; 4] = [0.0, 0.0, 0.0, 0.0 ];
    let mut num_roots = simplify_roots( &mut dst_roots, &roots );
    num_roots = fix_positive( &mut dst_roots[0..num_roots] );
    // Now `dst_roots` contains `num_roots` *positive* roots

    if num_roots == 0 {
      None
    } else {
      let mut closest = dst_roots[ 0 ];
      for i in 1..num_roots {
        closest = closest.min( dst_roots[ i ] );
      }

      let px = d.x as f64 + e.x as f64 * closest;
      let py = d.y as f64 + e.y as f64 * closest;
      let pz = d.z as f64 + e.z as f64 * closest;

      let alpha = 1.0 - a / ( px*px + pz*pz ).sqrt( );
      let n = Vec3::unit( ( alpha * px ) as f32, py as f32, ( alpha * pz ) as f32 );

      if num_roots % 2 == 1 { // Inside the torus
        Some( Hit::new( closest as f32, -n, self.mat.evaluate_at( &Vec2::ZERO ), false ) )
      } else { // Outside the torus
        Some( Hit::new( closest as f32, n, self.mat.evaluate_at( &Vec2::ZERO ), true ) )
      }
    }
  }
}

fn fix_positive( xs : &mut [ f64 ] ) -> usize {
  let mut num_positive = 0;
  for i in 0..xs.len( ) {
    if xs[ i ] >= 0.0001 {
      xs[ num_positive ] = xs[ i ];
      num_positive += 1;
    }
  }
  num_positive
}

fn simplify_roots< F: FloatType >( dst : &mut [F; 4], src : &Roots< F > ) -> usize {
  match src {
    Roots::No( _fs ) => 0,
    Roots::One( fs ) => {
        dst[ 0 ] = fs[ 0 ];
        1
      },
    Roots::Two( fs ) => {
        dst[ 0 ] = fs[ 0 ];
        dst[ 1 ] = fs[ 1 ];
        2
      },
    Roots::Three( fs ) => {
        dst[ 0 ] = fs[ 0 ];
        dst[ 1 ] = fs[ 1 ];
        dst[ 2 ] = fs[ 2 ];
        3
      },
    Roots::Four( fs ) => {
        dst[ 0 ] = fs[ 0 ];
        dst[ 1 ] = fs[ 1 ];
        dst[ 2 ] = fs[ 2 ];
        dst[ 3 ] = fs[ 3 ];
        4
      },
  }
}
