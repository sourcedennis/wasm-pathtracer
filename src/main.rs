mod graphics;
mod math;
mod scenes;
mod data;
mod tracer;

use graphics::primitives::{Triangle, Plane};
use graphics::ray::{Ray, Tracable};
use graphics::{Material, Color3, Mesh};
use graphics::{BVHNode, BVHNode4};
use data::cap_stack::Stack;
use tracer::{trace_original_color, MatRefract};
use math::Vec3;
use std::rc::Rc;
use std::collections::HashMap;
use scenes::{setup_scene_cloud100k};

// TEMP
use std::time::Instant;
extern crate rand;
use rand::Rng;
use std::mem::size_of;

fn main( ) {
  let num_triangles = 100000;
  let mut triangles = cloud( num_triangles );
  println!( "Triangles made" );
  let now = Instant::now();
  let (numinf, mut bvh) = BVHNode::build( &mut triangles, 32 ); // 128 is the number of bins
  println!( "BVH made. Count={}. Depth={}", BVHNode::node_count( &bvh ), BVHNode::depth( &bvh ) );
  println!( "Time: {}", now.elapsed( ).as_millis( ) );
  println!( "Verified: {:?}", BVHNode::verify( &triangles, numinf, &bvh ) );
  println!( "Memory: {:?}", bvh.len( ) * size_of::<BVHNode>( ) );

  let bvh4 = BVHNode4::collapse( &bvh );
  println!( "Collapsed. Count={}, Depth={}", BVHNode4::node_count( &bvh4 ), BVHNode4::depth( &bvh4 ) );
  println!( "Verified: {:?}", BVHNode4::verify( &triangles, numinf, &bvh4 ) );
  println!( "Memory: {:?}", bvh4.len( ) * size_of::<BVHNode4>( ) );

  // 100 reps takes around 130 seconds
  benchmark( 10 );
}

fn benchmark( num_repetitions : usize ) {
  let mut meshes = HashMap::new( );
  meshes.insert( 4, Mesh::Triangled( cloud( 100000 ) ) );
  let mut scene = setup_scene_cloud100k( &meshes );
  let camera_location = Vec3::new( 0.0, 4.8, 2.6 );
  let rays = setup_rays( 512, 512, camera_location, 0.97, 0.0 );
  let max_ray_depth = 5;
  let mut mat_stack = Stack::new1( ( max_ray_depth + 1 ) as usize, MatRefract::AIR );

  // BVH 2
  scene.rebuild_bvh( 32, false );

  let now = Instant::now();
  let mut total_num_hits = 0;

  println!( "### Benchmarking BVH binary ({} reps - 1 core) ###", num_repetitions );
  for _i in 0..num_repetitions {
    total_num_hits = 0;
    for r in &rays {
      let (num_hits, _c) = trace_original_color( &scene, r, max_ray_depth, &mut mat_stack );
      total_num_hits += num_hits;
    }
  }
  println!( "Time (avg): {}", now.elapsed( ).as_millis( ) as f32 / num_repetitions as f32 );
  println!( "#BVH Hits: {}", total_num_hits );
  
  // BVH 4
  scene.rebuild_bvh( 32, true );

  let now = Instant::now();
  let mut total_num_hits = 0;

  println!( "### Benchmarking BVH 4-way ({} reps - 1 core) ###", num_repetitions );
  for _i in 0..num_repetitions {
    total_num_hits = 0;
    for r in &rays {
      let (num_hits, _c) = trace_original_color( &scene, r, max_ray_depth, &mut mat_stack );
      total_num_hits += num_hits;
    }
  }
  println!( "Time (avg): {}", now.elapsed( ).as_millis( ) as f32 / num_repetitions as f32 );
  println!( "#BVH Hits: {}", total_num_hits );
}

fn cloud( n : usize ) -> Vec< Rc< dyn Tracable > > {
  let mat = Material::diffuse( Color3::RED );
  let mut rng = rand::thread_rng( );
  let mut triangles : Vec< Rc< dyn Tracable > > = Vec::with_capacity( n );
  for _i in 0..n {
    let center = 3.0 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v1 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v2 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let v3 = center + 0.5 * Vec3::new( rng.gen( ), rng.gen( ), rng.gen( ) );
    let t = Triangle::new( v1, v2, v3, mat.clone( ) );
    triangles.push( Rc::new( t ) );
  }
  triangles
}

fn setup_rays( width : usize, height : usize, camera_location : Vec3, camera_rot_x: f32, camera_rot_y : f32 ) -> Vec< Ray > {
  let mut rays = Vec::with_capacity( width * height );

  let fw = width as f32;
  let fh = height as f32;
  let w_inv = 1.0 / fw as f32;
  let h_inv = 1.0 / fh as f32;
  let ar = fw / fh;

  for y in 0..height {
    for x in 0..width {
      let fx = ( ( x as f32 + 0.5_f32 ) * w_inv - 0.5_f32 ) * ar;
      let fy = 0.5_f32 - ( y as f32 + 0.5_f32 ) * h_inv;
  
      let pixel = Vec3::new( fx, fy, 0.8 );
      let dir   = pixel.normalize( ).rot_x( camera_rot_x ).rot_y( camera_rot_y );
  
      rays.push( Ray::new( camera_location, dir ) );
    }
  }

  rays
}
