// External imports
use std::f32::INFINITY;
use std::i32;
use std::rc::Rc;
use std::fmt;
// Local imports
use crate::graphics::bvh::BVHNode;
use crate::graphics::{AABB, AABBx4};
use crate::graphics::ray::Tracable;

/// A node in a 4-way BVH
/// 
/// It only represent internal nodes (so no leaves). Leaves are represented by
/// a single integer, which references a range of shapes in the leaf.
/// Each internal node has at most 4 children. The node is intended to be used
/// with SIMD instructions, to perform fast traversal.
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
  /// Constructs a new internal BVH node
  pub fn node( child_bounds : AABBx4, children : [i32; 4], num_children : u32 ) -> BVHNode4 {
    BVHNode4 { child_bounds, children, num_children }
  }

  /// Collapses a 2-way BVH into a 4-way BVH.
  /// Each internal node in a 4-way BVH has at most 4 child nodes.
  /// The first element of the produced Vec is the root node in the tree
  pub fn collapse( bvh2 : &Vec< BVHNode > ) -> Vec< BVHNode4 > {
    let bvh_placeholder = BVHNode4 { child_bounds: AABBx4::empty( ), children: [i32::MIN, i32::MIN, i32::MIN, i32::MIN], num_children: 0 };

    // Find the lowest tree cost
    let mut memo : Vec< Option< Vec< f32 > > > = vec![ None; bvh2.len( ) ];
    r_cost( &mut memo, bvh2, 0, 4 );
  
    // Backtrack to build the tree with that cost
    let mut dst = Vec::with_capacity( bvh2.capacity( ) );
    let res = collapse_with( &mut dst, bvh2, &memo, 0, 4 );

    if res.len( ) > 1 {
      // Rebuild the tree if it doesn't comform to expectation
      // TODO: Optimize this away. Though, this is not the bottleneck

      dst.clear( );
      dst.push( bvh_placeholder );
      let res2 = collapse_with( &mut dst, bvh2, &memo, 0, 4 );
      
      let mut children: [i32;4] = [0,0,0,0];
      let num_children = res2.len( );
      let mut bounds_box = [ AABB::EMPTY, AABB::EMPTY, AABB::EMPTY, AABB::EMPTY ];
      for i in 0..res2.len( ) {
        bounds_box[ i ] = res2[ i ].0;
        children[ i ]   = res2[ i ].1;
      }
      let simd_bounds = AABBx4::new( bounds_box[ 0 ], bounds_box[ 1 ], bounds_box[ 2 ], bounds_box[ 3 ] );

      dst[ 0 ] = BVHNode4::node( simd_bounds, children, num_children as u32 );
    } else {
      assert!( res[ 0 ].1 == 0 );
    }
    dst
  }

  /// Returns the number of nodes that are in the tree
  /// This includes (concisely-represented) leaf nodes
  pub fn node_count( bvh : &Vec< BVHNode4 > ) -> usize {
    BVHNode4::node_count_rec( bvh, 0 )
  }

  /// Recursively finds the number of nodes in the tree
  /// See `BVH4Node::node_count(..)`
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

  /// Returns the depth of the tree
  /// The depth is the maximum number of edges from the root to any leaf
  pub fn depth( bvh : &Vec< BVHNode4 > ) -> usize {
    BVHNode4::depth_rec( bvh, 0 )
  }

  /// Recursively finds the tree depth
  /// See `BVH4Node::depth(..)`
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

  /// Verifies the correctness of the tree
  /// This is done by checking the following properties:
  /// * Does the tree contain all shapes in `shapes`?
  /// * Do the bounds of each node properly contain the bounds of its children?
  pub fn verify( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode4 > ) -> bool {
    verify_bvh( shapes, num_infinite, bvh )
  }
}

/// Collapse the tree by backtracking on the minimal cost in `memo` (which is obtained from `r_cost(..)`)
/// It returns a collection of nodes that replace the `node_i` (from the 2-way BVH) in the 4-way BVH;
/// for each node the AABB is also returned, such that it can be included in the parent node.
/// When a node is discarded (because it is replaced by its children), the multiple replacing children
/// are returned.
/// Note that `cutsize` is the maximum number of nodes that node `node_i` can be replaced with.
fn collapse_with( dst : &mut Vec< BVHNode4 >, bvh : &Vec< BVHNode >, memo : &Vec< Option< Vec< f32 > > >, node_i : usize, cutsize : usize ) -> Vec< (AABB, i32) > {
  let bvh_placeholder = BVHNode4 { child_bounds: AABBx4::empty( ), children: [i32::MIN, i32::MIN, i32::MIN, i32::MIN], num_children: 0 };

  // At this point `memo` is already filled with costs for each tree-cut
  // Backtrack on this to build the 4-way BVH with minimal cost
  
  if bvh[ node_i ].is_leaf( ) { // leaf
    // A leaf still has an AABB, but no node in `dst`
    let shape_range = 0x80000000 | ( bvh[ node_i ].count << 27 ) | ( bvh[ node_i ].left_first );
    vec![ ( bvh[ node_i ].bounds, unsafe { std::mem::transmute::< u32, i32 >( shape_range ) } ) ]
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;
    
    // Finds the optimal `t`, being the number of children this node should have
    let t = find_t( bvh, memo, node_i, cutsize );

    if t == 1 { // Keep the node (So it can have 4 children)
      let index = dst.len( );
      dst.push( bvh_placeholder );

      // Find optimal `i`. Being the number of children the left-child has. The right child has `4-i` children.
      let i_min = find_i( bvh, memo, node_left_i, node_right_i, 4 );

      let lcs = collapse_with( dst, bvh, memo, node_left_i, i_min );
      let rcs = collapse_with( dst, bvh, memo, node_right_i, 4 - i_min );

      // Build the components of the `BVHNode4`, and add the node to the BVH
      let mut children = [i32::MIN, i32::MIN, i32::MIN, i32::MIN];
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

      let num_children = lcs.len( ) + rcs.len( );
      let simd_bounds = AABBx4::new( bounds_box[ 0 ], bounds_box[ 1 ], bounds_box[ 2 ], bounds_box[ 3 ] );
      dst[ index ] = BVHNode4::node( simd_bounds, children, num_children as u32 );

      vec![ ( simd_bounds.extract_hull( num_children ), index as i32 ) ]
    } else { // Discard the node (So it has `t` children, where `t < cutsize`)
      // Find optimal `i`. Being the number of children the left-child has. The right child has `t-i` children.
      let i_min = find_i( bvh, memo, node_left_i, node_right_i, t );

      let c1 = collapse_with( dst, bvh, memo, node_left_i, i_min );
      let c2 = collapse_with( dst, bvh, memo, node_right_i, t - i_min );

      // That means the node is replaced by the 4-way BVH equivalent of its children
      [&c1[..], &c2[..]].concat()
    }
  }
}

/// Finds the optimal number (that is no more than `cutsize`) of children `node_i` should have.
/// WARNING: Should only be called once `memo` is fully constructed
fn find_t( bvh : &Vec< BVHNode >, memo : &Vec< Option< Vec< f32 > > >, node_i : usize, cutsize : usize ) -> usize {
  if bvh[ node_i ].is_leaf( ) {
    1
  } else if let Some( m ) = &memo[ node_i ] {
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

/// Finds the optimal number of nodes `i` that should be obtained by collapsing node `node_left_i`.
///   When collapsing the right node (`node_right_i`), it should have `t - i` nodes.
///   So the optimal `i` is lower than `t`.
fn find_i( bvh : &Vec< BVHNode >, memo : &Vec< Option< Vec< f32 > > >, node_left_i : usize, node_right_i : usize, t : usize ) -> usize {
  let mut i_min = 1;
  let mut i_min_val = node_flat_cost( memo, bvh, node_left_i, 1 ) + node_flat_cost( memo, bvh, node_right_i, t - 1 );
  
  for i in 2..t {
    let i_val = node_flat_cost( memo, bvh, node_left_i, i ) + node_flat_cost( memo, bvh, node_right_i, t - i );

    if i_val < i_min_val {
      i_min = i;
      i_min_val = i_val;
    }
  }

  i_min
}

/// Returns the minimal cost of `node_i`, where the maximum number of children is at most `cutsize`.
/// WARNING: Should only be called once `memo` is fully constructed
fn node_flat_cost( memo : &Vec< Option< Vec< f32 > > >, bvh : &Vec< BVHNode >, node_i : usize, cutsize : usize ) -> f32 {
  if bvh[ node_i ].is_leaf( ) {
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

/// Applies memoisation to find the optimal tree-cut for `node_i`. This minimises the traversal cost in the tree.
/// That is, the tree is made as shallow as possible.
fn r_cost( memo : &mut Vec< Option< Vec< f32 > > >, bvh : &Vec< BVHNode >, node_i : usize, cutsize : usize ) -> f32 {
  let t_cost = 1.0; // Cost to perform an AABB intersection
  let max_childs = 4;

  if bvh[ node_i ].is_leaf( ) {
    // A leaf still has an AABB
    t_cost
  } else {
    let node_left_i  = bvh[ node_i ].left_first as usize;
    let node_right_i = ( node_left_i + 1 ) as usize;
  
    if memo[ node_i ] == None {
      let mut cost = vec![ INFINITY; max_childs ];
      for t in 2..(max_childs+1) {
        for i in 1..t {
          let r = r_cost( memo, bvh, node_left_i, i ) + r_cost( memo, bvh, node_right_i, t - i );
          cost[ t - 1 ] = cost[ t - 1 ].min( r );
        }
        cost[ 1 - 1 ] = cost[ 1 - 1 ].min( t_cost + cost[ t - 1 ] );
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

/// Returns the current traversal cost of the full BVH-2 rooted in `node_i`
fn current_cost( bvh : &Vec< BVHNode >, node_i : usize ) -> f32 {
  if bvh[ node_i ].is_leaf( ) {
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

/// Verifies correctness of the obtained 4-way BVH (See `BVHNode::verify(..)`)
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

/// Sets `true` in `contained` for each shape that is in the BVH rooted in `i`.
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

/// Returns `Some(..)` if the bounds for `node_i` contain the bounds of its children;
///   and this is recursively true for their children.
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

// Nicely prints a BVHNode4 for much-needed debugging
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
