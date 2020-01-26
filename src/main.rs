mod rng;
mod math;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::math::EmpiralPDF;

use std::f32::INFINITY;

use crate::rng::Rng;
use crate::math::Vec3;

pub fn main( ) {
  let mut rng = Rng::with_state( SystemTime::now().duration_since(UNIX_EPOCH).expect( "" ).as_millis( ) as u32 );

  let mut pdf = EmpiralPDF::new( 5 );
  pdf.set( 0, 10.0 );
  pdf.set( 1,  1.0 );
  pdf.set( 2,  5.0 );
  pdf.set( 3, 60.0 );
  pdf.set( 4,  2.0 );
  
  let mut probs = vec![ 0.0; 5 ];

  for _i in 0..100000000 {
    probs[ pdf.sample( &mut rng ) ] += ( 1.0 / 100000000.0 ) * 78.0;
  }

  println!( "{:?}", probs );
  
  probs = vec![ 0.0; 5 ];

  pdf.set( 2, 50.0 );

  for _i in 0..100000000 {
    probs[ pdf.sample( &mut rng ) ] += ( 1.0 / 100000000.0 ) * 123.0;
  }
  
  println!( "{:?}", probs );
}

// Test case. Shows that the hemisphere is uniformly sampled
fn test_hemisphere( ) {
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

  println!( "{:?}", sum.normalize( ) );
}
