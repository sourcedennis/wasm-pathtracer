mod graphics;
mod math;

use graphics::primitives::{Triangle, Plane};
use graphics::Material;
use graphics::Color3;
use graphics::ray::Tracable;
use graphics::{BVHNode};
use math::Vec3;
use std::rc::Rc;

// TEMP
use std::time::Instant;
extern crate rand;
use rand::Rng;
use std::mem::size_of;

fn main( ) {
  let mat = Material::diffuse( Color3::RED );
  let t1 = Triangle::new( Vec3::new( 0.0, 0.0, 0.0 ), Vec3::new( 1.0, 0.0, 0.0 ), Vec3::new( 1.0, 1.0, 0.0 ), mat.clone( ) );
  let t2 = Triangle::new( Vec3::new( 3.0, 0.0, 0.0 ), Vec3::new( 4.0, 0.0, 0.0 ), Vec3::new( 4.0, 1.0, 0.0 ), mat.clone( ) );
  let t3 = Triangle::new( Vec3::new( 0.0, 4.0, 0.0 ), Vec3::new( 1.0, 4.0, 0.0 ), Vec3::new( 1.0, 5.0, 0.0 ), mat.clone( ) );

  let mut rng = rand::thread_rng( );

  //let mut triangles : Vec< Rc< dyn Tracable > > = vec![ Rc::new( t1 ), Rc::new( t2 ), Rc::new( t3 ) ];
  //let bvh = build_bvh( &mut triangles, 4 );

  //println!( "X {:?}", bvh );
  //println!( "Triangles {:?}", triangles );
  let num_triangles = 100000;
  let mut triangles : Vec< Rc< dyn Tracable > > = Vec::with_capacity( num_triangles + 1 );
  for _i in 0..num_triangles {
    let center = 3.0 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v1 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v2 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v3 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let t = Triangle::new( v1, v2, v3, mat.clone( ) );
    triangles.push( Rc::new( t ) );
  }
  triangles.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  println!( "Triangles made" );
  let now = Instant::now();
  let (numinf, bvh) = BVHNode::build( &mut triangles, 16 ); // 128 is the number of bins
  println!( "BVH made {} {}", bvh.len( ), numinf );
  println!( "Time: {}", now.elapsed( ).as_millis( ) );
  println!( "Verified: {:?}", BVHNode::verify( &triangles, numinf, &bvh ) );
  println!( "BVH depth: {}", BVHNode::depth( &bvh ) );
}
