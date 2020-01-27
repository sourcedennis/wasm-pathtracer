mod rng;
mod math;
mod data;
mod graphics;
mod render_target;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::math::EmpiricalPDF;
use crate::data::PhotonTree;

use std::f32::INFINITY;

use crate::rng::Rng;
use crate::math::Vec3;

pub fn main( ) {
  let mut rng = Rng::with_state( SystemTime::now().duration_since(UNIX_EPOCH).expect( "" ).as_millis( ) as u32 );

  let mut tree = PhotonTree::new( 4 );
  //let mut pdf = EmpiricalPDF::new( 4 );
  let t1 = SystemTime::now( );
  for _i in 0..5000000 {
    // Light source 0 and 1 contribute on the left
    tree.insert( rng.next_in_range( 0, 2 ), Vec3::new( 0.5 * rng.next( ), rng.next( ), rng.next( ) ), rng.next( ) );
    // Light source 2 and 4 contribute on the right
    tree.insert( rng.next_in_range( 2, 4 ), Vec3::new( 0.5 + 0.5 * rng.next( ), rng.next( ), rng.next( ) ), rng.next( ) );
  }
  //println!( "{:?}", tree );
  println!( "Added: {:?}", SystemTime::now( ).duration_since( t1 ) );

  println!( "Sampling left: " );
  // Sample the left
  for i in 0..10 {
    println!( "{:?}", tree.sample( &mut rng, Vec3::new( 0.4, 0.4, 0.4 ) ) );
  }
  println!( "Sampling center: " );
  // Sample the left
  for i in 0..10 {
    println!( "{:?}", tree.sample( &mut rng, Vec3::new( 0.5, 0.4, 0.4 ) ) );
  }
  println!( "Sampling right: " );
  // Sample the right
  for i in 0..10 {
    println!( "{:?}", tree.sample( &mut rng, Vec3::new( 0.6, 0.4, 0.4 ) ) );
  }

  // let t1 = SystemTime::now( );
  // for i in 0..1000000 {
  //   // let res = pdf.sample( &mut rng );
  //   // let res_prob = pdf.bin_prob( res );
  //   // println!( "{:?}", (res,res_prob) );
  //   println!( "{:?}", tree.sample( &mut rng, Vec3::new( 0.4, 0.4, 0.4 ) ) );
  //   tree.sample( &mut rng, Vec3::new( 0.4, 0.4, 0.4 ) );
  // }
  // let t2 = SystemTime::now( ).duration_since( t1 );
  // println!( "{:?}", t2 );
  // for _i in 0..50 {
  //   let v = Vec3::new( rng.next( ), rng.next( ), rng.next( ) );
  //   println!( "{:?}", tree.sample( &mut rng, v ) );
  // }
}

pub fn test_empirical_pdf( ) {
  let mut rng = Rng::with_state( SystemTime::now().duration_since(UNIX_EPOCH).expect( "" ).as_millis( ) as u32 );

  let mut pdf = EmpiricalPDF::new( 5 );
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
