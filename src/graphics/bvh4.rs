use crate::graphics::bvh::BVHNode;
use crate::graphics::AABB;
use std::f32::INFINITY;

pub fn collapse_bvh4( bvh : &mut Vec< BVHNode > ) {
  let mut memo : Vec< Option< Vec< f32 > > > = vec![ None; bvh.len( ) ];
  let min_cost = r_cost( &mut memo, bvh, 0, 4 );

  let pre_cost = current_cost( bvh, 0 );
  //println!( "Cost: {} / {}", c, current_cost( bvh, 0 ) );
  let found_cost = collapse_with( bvh, &mut memo, 0, 4 );

  assert!( min_cost == found_cost as f32 );
  //println!( "Cost: {} {}", cf, current_cost( bvh, 0 ) );
}

fn collapse_with( bvh : &mut Vec< BVHNode >, memo : &mut Vec< Option< Vec< f32 > > >, node_i : usize, cutsize : usize ) -> usize {
  let T_COST = 1; // Cost to perform an AABB intersection
  let MAX_CHILDS = 4;
  
  if bvh[ node_i ].count != 0 { // leaf
    // A leaf still has an AABB
    T_COST
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;
    
    if let Some( m ) = &memo[ node_i ] {
      // First find the value of `t`
      let mut t_min     = 1;
      let mut t_min_val = m[ 1 - 1 ];
      for t in 2..(cutsize+1) {
        if m[ t - 1 ] < t_min_val {
          t_min = t;
          t_min_val = m[ t - 1 ];
        }
      }

      // Then find `i`
      let mut c0 = 0;
      if t_min == 1 { // Keep the node
        t_min = 4;
        c0 = 1;
      } else { // Discard the node. Mark it as discarded
        bvh[ node_i ].bounds.x_min = INFINITY;
      }

      let mut i_min = 1;
      let mut i_min_val = node_flat_cost( memo, bvh, node_left_i, 1 ) + node_flat_cost( memo, bvh, node_right_i, t_min - 1 );
      
      for i in 2..t_min {
        let i_val = node_flat_cost( memo, bvh, node_left_i, i ) + node_flat_cost( memo, bvh, node_right_i, t_min - i );

        if i_val < i_min_val {
          i_min = i;
          i_min_val = i_val;
        }
      }

      let c1 = collapse_with( bvh, memo, node_left_i, i_min );
      let c2 = collapse_with( bvh, memo, node_right_i, t_min - i_min );

      c0 + c1 + c2
    } else {
      panic!( "WUT" );
    }

  }
}

fn node_flat_cost( memo : &mut Vec< Option< Vec< f32 > > >, bvh : &Vec< BVHNode >, node_i : usize, cutsize : usize ) -> f32 {
  if bvh[ node_i ].count != 0 { // leaf
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

  if bvh[ node_i ].count != 0 { // leaf
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
  if bvh[ node_i ].count != 0 { // leaf
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
