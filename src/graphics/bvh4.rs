use crate::graphics::bvh::BVHNode;
use crate::graphics::{AABB, AABBx4};
use crate::graphics::ray::Tracable;
use std::f32::INFINITY;
use std::rc::Rc;
use packed_simd::*;
use std::i32;
use std::fmt;

#[derive(Copy,Clone)]
#[repr(align(128))]
pub struct BVHNode4 {
  // The bounds of the children
  pub child_bounds : AABBx4,
  // 1 top bit set if a leaf. 4 bits for shape count. 27 bits for shape index
  pub children     : [i32; 4],
  pub num_children : u32
  // 3x 32-bit free
}

impl BVHNode4 {
  pub fn node( child_bounds : AABBx4, children : [i32; 4], num_children : u32 ) -> BVHNode4 {
    BVHNode4 { child_bounds, children, num_children }
  }

  pub fn collapse( bvh2 : &Vec< BVHNode > ) -> Vec< BVHNode4 > {
    let mut memo : Vec< Option< Vec< f32 > > > = vec![ None; bvh2.len( ) ];
    let min_cost = r_cost( &mut memo, bvh2, 0, 4 );
    let BVH_PLACEHOLDER = BVHNode4 { child_bounds: AABBx4::empty( ), children: [i32::MIN, i32::MIN, i32::MIN, i32::MIN], num_children: 0 };
  
    let pre_cost = current_cost( bvh2, 0 );
    //println!( "Cost: {} / {}", c, current_cost( bvh, 0 ) );
    let mut dst = Vec::with_capacity( bvh2.capacity( ) );
    let res = collapse_with( &mut dst, bvh2, &memo, 0, 4 );

    if res.len( ) > 1 {
      //println!( "RESTART {:?}", res );
      dst.clear( );
      dst.push( BVH_PLACEHOLDER );
      let res2 = collapse_with( &mut dst, bvh2, &memo, 0, 4 );
      //println!( "RESTART DONE" );
      
      let mut children: [i32;4] = [0,0,0,0];
      let num_children = res2.len( );
      let mut bounds_box = [ AABB::EMPTY, AABB::EMPTY, AABB::EMPTY, AABB::EMPTY ];
      for i in 0..res2.len( ) {
        bounds_box[ i ] = res2[ i ].0;
        children[ i ]   = res2[ i ].1;
      }
      //println!( "RESTART DONE2" );
      let simd_bounds = AABBx4::new( bounds_box[ 0 ], bounds_box[ 1 ], bounds_box[ 2 ], bounds_box[ 3 ] );

      dst[ 0 ] = BVHNode4::node( simd_bounds, children, num_children as u32 );
    } else {
      assert!( res[ 0 ].1 == 0 );
    }
    dst
  }

  pub fn node_count( bvh : &Vec< BVHNode4 > ) -> usize {
    BVHNode4::node_count_rec( bvh, 0 )
  }

  fn node_count_rec( bvh : &Vec< BVHNode4 >, i : i32 ) -> usize {
    if i < 0 { // leaf
      1
    } else {
      let mut count_sum = 1;
      for j in 0..bvh[ i as usize ].num_children {
        count_sum += BVHNode4::node_count_rec( bvh, bvh[ i as usize ].children[ j as usize ] );
      }
      count_sum
    }
  }

  pub fn depth( bvh : &Vec< BVHNode4 > ) -> usize {
    BVHNode4::depth_rec( bvh, 0 )
  }

  fn depth_rec( bvh : &Vec< BVHNode4 >, i : i32 ) -> usize {
    if i < 0 { // leaf
      0
    } else {
      let mut depth = BVHNode4::depth_rec( bvh, bvh[ i as usize ].children[ 0 ] );
      for j in 1..bvh[ i as usize ].num_children {
        depth = depth.max( BVHNode4::depth_rec( bvh, bvh[ i as usize ].children[ j as usize ] ) );
      }
      depth + 1
    }
  }

  pub fn verify( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode4 > ) -> bool {
    verify_bvh( shapes, num_infinite, bvh )
  }
}

fn collapse_with( dst : &mut Vec< BVHNode4 >, bvh : &Vec< BVHNode >, memo : &Vec< Option< Vec< f32 > > >, node_i : usize, cutsize : usize ) -> Vec< (AABB, i32) > {
  let T_COST = 1; // Cost to perform an AABB intersection
  let MAX_CHILDS = 4;
  let BVH_PLACEHOLDER = BVHNode4 { child_bounds: AABBx4::empty( ), children: [i32::MIN, i32::MIN, i32::MIN, i32::MIN], num_children: 0 };
  
  if bvh[ node_i ].is_leaf( ) { // leaf
    // A leaf still has an AABB
    let count = 0x80000000 | ( bvh[ node_i ].count << 27 ) | ( bvh[ node_i ].left_first );
    vec![ ( bvh[ node_i ].bounds, unsafe { std::mem::transmute::< u32, i32 >( count ) } ) ]
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;
    
    let t = find_t( bvh, memo, node_i, cutsize );

    if t == 1 { // Keep the node
      let index = dst.len( );
      dst.push( BVH_PLACEHOLDER );
      //println!( "KEEP {}", index );

      // Find `i`
      let mut i_min = 1;
      let mut i_min_val = node_flat_cost( memo, bvh, node_left_i, 1 ) + node_flat_cost( memo, bvh, node_right_i, 4 - 1 );
      
      for i in 2..4 {
        let i_val = node_flat_cost( memo, bvh, node_left_i, i ) + node_flat_cost( memo, bvh, node_right_i, 4 - i );

        if i_val < i_min_val {
          i_min = i;
          i_min_val = i_val;
        }
      }

      let num_children = find_t( bvh, memo, node_left_i, i_min ) + find_t( bvh, memo, node_right_i, 4 - i_min );
      let mut children = [i32::MIN, i32::MIN, i32::MIN, i32::MIN];

      let lcs = collapse_with( dst, bvh, memo, node_left_i, i_min );
      let rcs = collapse_with( dst, bvh, memo, node_right_i, 4 - i_min );

      let mut bounds_box = [ AABB::EMPTY, AABB::EMPTY, AABB::EMPTY, AABB::EMPTY ];
      let mut j = 0;
      for e in &lcs {
        children[ j ] = e.1;
        bounds_box[ j ] = e.0;
        j += 1;
      }
      for e in &rcs {
        children[ j ] = e.1;
        bounds_box[ j ] = e.0;
        j += 1;
      }
      //println!( "{} + {} = {}", lcs.len( ), rcs.len( ), num_children );

      let simd_bounds = AABBx4::new( bounds_box[ 0 ], bounds_box[ 1 ], bounds_box[ 2 ], bounds_box[ 3 ] );
      dst[ index ] = BVHNode4::node( simd_bounds, children, num_children as u32 );

      vec![ ( simd_bounds.extract_hull( num_children ), index as i32 ) ]
    } else { // Discard the node
      let mut i_min = 1;
      let mut i_min_val = node_flat_cost( memo, bvh, node_left_i, 1 ) + node_flat_cost( memo, bvh, node_right_i, t - 1 );
      
      for i in 2..t {
        let i_val = node_flat_cost( memo, bvh, node_left_i, i ) + node_flat_cost( memo, bvh, node_right_i, t - i );

        if i_val < i_min_val {
          i_min = i;
          i_min_val = i_val;
        }
      }

      let c1 = collapse_with( dst, bvh, memo, node_left_i, i_min );
      let c2 = collapse_with( dst, bvh, memo, node_right_i, t - i_min );

      [&c1[..], &c2[..]].concat()
    }
  }
}

fn find_t( bvh : &Vec< BVHNode >, memo : &Vec< Option< Vec< f32 > > >, node_i : usize, cutsize : usize ) -> usize {
  if bvh[ node_i ].is_leaf( ) {
    1
  } else if let Some( m ) = &memo[ node_i ] {
    // First find the value of `t`
    let mut t_min     = 1;
    let mut t_min_val = m[ 1 - 1 ];
    for t in 2..(cutsize+1) {
      if m[ t - 1 ] < t_min_val {
        t_min = t;
        t_min_val = m[ t - 1 ];
      }
    }
    t_min
  } else {
    panic!( "INVALID T" );
  }
}

fn node_flat_cost( memo : &Vec< Option< Vec< f32 > > >, bvh : &Vec< BVHNode >, node_i : usize, cutsize : usize ) -> f32 {
  if bvh[ node_i ].is_leaf( ) { // leaf
    1.0
  } else if let Some( m ) = &memo[ node_i ] {
    let mut cut_min = m[ 0 ];
    for i in 1..cutsize {
      cut_min = cut_min.min( m[ i ] );
    }
    cut_min
  } else {
    INFINITY
  }
}

fn r_cost( memo : &mut Vec< Option< Vec< f32 > > >, bvh : &Vec< BVHNode >, node_i : usize, cutsize : usize ) -> f32 {
  let T_COST = 1.0; // Cost to perform an AABB intersection
  let MAX_CHILDS = 4;

  if bvh[ node_i ].is_leaf( ) { // leaf
    // A leaf still has an AABB
    T_COST
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;
  
    if memo[ node_i ] == None {
      let mut cost = vec![ INFINITY; MAX_CHILDS ];
      for t in 2..(MAX_CHILDS+1) {
        for i in 1..t {
          let r = r_cost( memo, bvh, node_left_i, i ) + r_cost( memo, bvh, node_right_i, t - i );
          cost[ t - 1 ] = cost[ t - 1 ].min( r );
        }
        cost[ 1 - 1 ] = cost[ 1 - 1 ].min( T_COST + cost[ t - 1 ] );
      }
      memo[ node_i ] = Some( cost );
    }
    
    if let Some( m ) = &memo[ node_i ] {
      if cutsize == 0 {
        0.0
      } else {
        let mut cut_min = m[ 0 ];
        for i in 1..cutsize {
          cut_min = cut_min.min( m[ i ] );
        }
        cut_min
      }
    } else {
      panic!( "r_cost None while it was set to Some()" )
    }
  }
}

fn current_cost( bvh : &Vec< BVHNode >, node_i : usize ) -> f32 {
  if bvh[ node_i ].is_leaf( ) { // leaf
    1.0
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;

    if bvh[ node_i ].bounds.x_min == INFINITY {
      current_cost( bvh, node_left_i ) + current_cost( bvh, node_right_i )
    } else {
      1.0 + current_cost( bvh, node_left_i ) + current_cost( bvh, node_right_i )
    }
  }
}

fn verify_bvh( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode4 > ) -> bool {
  let self_bounds = bvh[ 0 ].child_bounds.extract_hull( bvh[ 0 ].num_children as usize );

  let a = verify_bvh_bounds( shapes, num_infinite, bvh, self_bounds, 0 ).is_some( );
  let mut contained = vec![false; shapes.len()-num_infinite];
  verify_bvh_contains( &mut contained, bvh, 0 );

  let mut has_all = true;
  for c in &contained {
    has_all = has_all && *c;
  }

  a && has_all
}

fn verify_bvh_contains( contained : &mut [bool], bvh : &Vec< BVHNode4 >, i : i32 ) {
  if i >= 0 { // node
    for j in 0..bvh[ i as usize ].num_children {
      verify_bvh_contains( contained, bvh, bvh[ i as usize ].children[ j as usize ] );
    }
  } else { // leaf
    let num_shapes = ( ( unsafe { std::mem::transmute::< i32, u32 >( i ) } >> 27 ) & 0x3 ) as usize;
    let shape_index = ( unsafe { std::mem::transmute::< i32, u32 >( i ) } & 0x7FFFFFF ) as usize;

    for i in 0..num_shapes {
      contained[ shape_index + i ] = true;
    }
  }
}

fn verify_bvh_bounds( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode4 >, bounds : AABB, i : i32 ) -> Option< AABB > {
  if i >= 0 {
    // WARNING: Only works with non-empty inner nodes

    let n = &bvh[ i as usize ];

    if n.num_children > 4 {
      return None;
    }

    let mut new_bounds =
      if let Some( b ) = verify_bvh_bounds( shapes, num_infinite, bvh, n.child_bounds.extract( 0 ), n.children[ 0 ] ) {
        b
      } else {
        return None;
      };

    for i in 1..n.num_children {
      if let Some( b ) = verify_bvh_bounds( shapes, num_infinite, bvh, n.child_bounds.extract( i as usize ), n.children[ i as usize ] ) {
        new_bounds = new_bounds.join( &b );
      } else {
        return None;
      }
    }

    Some( bounds )
  } else { // leaf
    let num_shapes = ( ( unsafe { std::mem::transmute::< i32, u32 >( i ) } >> 27 ) & 0x3 ) as usize;
    let shape_index = ( unsafe { std::mem::transmute::< i32, u32 >( i ) } & 0x7FFFFFF ) as usize;

    let mut cum_bounds = shapes[ num_infinite+shape_index ].aabb( ).unwrap( );
    for i in (num_infinite+shape_index)..(num_infinite+shape_index+num_shapes) {
      if let Some( b ) = shapes[ i ].aabb( ) {
        if !bounds.contains( &b ) {
          return None;
        }
        cum_bounds = cum_bounds.join( &b );
      } else {
        return None;
      }
    }
    Some( bounds )
  }
}

impl fmt::Debug for BVHNode4 {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.num_children == 0 {
      write!(f, "BVHNode4 {{ children: [] }}" )
    } else if self.num_children == 1 {
      write!(f, "BVHNode4 {{ children: [{}] }}", self.children[ 0 ] )
    } else if self.num_children == 2 {
      write!(f, "BVHNode4 {{ children: [{}, {}] }}", self.children[ 0 ], self.children[ 1 ] )
    } else if self.num_children == 3 {
      write!(f, "BVHNode4 {{ children: [{}, {}, {}] }}", self.children[ 0 ], self.children[ 1 ], self.children[ 2 ] )
    } else if self.num_children == 4 {
      write!(f, "BVHNode4 {{ children: [{}, {}, {}, {}] }}", self.children[ 0 ], self.children[ 1 ], self.children[ 2 ], self.children[ 3 ] )
    } else {
      write!(f, "BVHNode4 {{ ? }}" )
    }
  }
}