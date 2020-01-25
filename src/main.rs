mod rng;
mod math;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::f32::INFINITY;

use crate::rng::Rng;
use crate::math::Vec3;

pub fn main( ) {
  let mut rng = Rng::with_state( SystemTime::now().duration_since(UNIX_EPOCH).expect( "" ).as_millis( ) as u32 );

  let mut sum = Vec3::ZERO;

  let n = Vec3::unit( rng.next( ) * 2.0 - 1.0, rng.next( ) * 2.0 - 1.0, rng.next( ) * 2.0 - 1.0 );
  println!( "{:?}", n );

  let mut min_x = INFINITY;
  let mut max_x = -INFINITY;
  let mut min_y = INFINITY;
  let mut max_y = -INFINITY;
  let mut min_z = INFINITY;
  let mut max_z = -INFINITY;

  for i in 0..1000000 {
    let v = rng.next_hemisphere( &n );
    sum += v;

    assert!( v.dot( n ) > 0.0 );

    min_x = v.x.min( min_x );
    max_x = v.x.max( max_x );
    min_y = v.y.min( min_y );
    max_y = v.y.max( max_y );
    min_z = v.z.min( min_z );
    max_z = v.z.max( max_z );
  }

  println!( "Banana {:?}", sum.normalize( ) );
  //println!( "Banana {} {} {} {} {} {}", min_x, max_x, min_y, max_y, min_z, max_z );
}
