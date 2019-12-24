// External imports
use std::rc::Rc;
// Local imports
use crate::graphics::AABB;
use crate::graphics::ray::Tracable;
use crate::math::Vec3;

/// A node in a 2-way BVH
/// 
/// It represents both internal nodes and leaves. It is an internal node when
/// `count` is 0; it is a leaf otherwise (where it represents the number of shapes).
/// `left_first` represents the id of the left BVH child (if it is a internal node)
///   and the offset in the array of primitives (if it is a leaf).
#[derive(Copy,Clone,Debug)]
#[repr(align(32))]
pub struct BVHNode {
  pub bounds     : AABB,
  pub left_first : u32,
  pub count      : u32
}

impl BVHNode {
  /// Constructs a new leaf node. A leaf contains `count` shapes in the shapes
  /// array, starting at offset `offset`.
  pub fn leaf( bounds : AABB, offset : u32, count : u32 ) -> BVHNode {
    BVHNode { bounds, left_first: offset, count }
  }

  /// Constructs a new internal node. An internal node has two children, where
  /// `first` is the index of the left child. The right child *must* be located
  /// at index `first+1`.
  pub fn node( bounds : AABB, first : u32 ) -> BVHNode {
    BVHNode { bounds, left_first: first, count: 0 }
  }

  /// Returns true if the node is a leaf
  pub fn is_leaf( &self ) -> bool {
    self.count > 0
  }

  /// Constructs a 2-way BVH for the shapes in `shapes`. The order of these
  /// shapes will be modified.
  /// Shapes with an infinite size (e.g. planes) are *not* added to the BVH;
  ///   instead, these are moved to the start of the array.
  /// The first element in the returned tuple is the number of such elements.
  ///   The second is the BVH.
  /// The root node is located at index 0 in the array.
  pub fn build( shapes : &mut [Rc< dyn Tracable >], num_bins : usize ) -> (usize, Vec< BVHNode >) {
    build_bvh( shapes, num_bins )
  }

  /// Verifies whether the BVH is valid for the shapes
  /// It checks:
  /// - If the shapes in a leaf are fully contained in the bounds of the leaf
  /// - If the bounds of a node's children are contained within its own bounds
  /// Only if both conditions hold for all shapes and leaves, is the BVH valid
  pub fn verify( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode > ) -> bool {
    verify_bvh( shapes, num_infinite, bvh )
  }

  /// Returns the depth of the tree
  /// The depth is the maximum number of edges from the root to any leaf
  pub fn depth( nodes : &Vec< BVHNode > ) -> u32 {
    bvh_depth( nodes )
  }

  /// Returns the number of nodes in the tree (this includes leaves)
  pub fn node_count( nodes : &Vec< BVHNode > ) -> usize {
    BVHNode::count_node_rec( nodes, 0 )
  }

  // Recursively counts the number of nodes in the tree, starting at index `i`.
  fn count_node_rec( nodes : &Vec< BVHNode >, i : usize ) -> usize {
    if nodes[ i ].is_leaf( ) { // leaf node
      1
    } else {
      1 + BVHNode::count_node_rec( nodes, nodes[ i ].left_first as usize ) + BVHNode::count_node_rec( nodes, nodes[ i ].left_first as usize + 1 )
    }
  }
}

/// A Shape representation that is used during the construction
/// This avoids having the re-compute the location and AABB many times.
#[derive(Clone)]
struct ShapeRep {
  shape    : Rc< dyn Tracable >,
  location : Vec3,
  bounds   : AABB
}

/// Used to initialise "empty" array elements
static BVH_PLACEHOLDER: BVHNode =
  BVHNode {
    bounds:     AABB::EMPTY
  , left_first: 0
  , count:      0
  };

// Builds a 2-way BVH with the given number of bins
// Uses O(k * n log n) time, where `k` is the number of bins
// Returns the number of "infinite" nodes that did not fit in the tree,
//   together with the BVH tree.
fn build_bvh( shapes : &mut [Rc< dyn Tracable >], num_bins : usize ) -> (usize, Vec< BVHNode >) {
  let (num_infinite, mut reps) = shape_reps( shapes );

  let rep_len = reps.len( );
  let mut dst  = Vec::with_capacity( rep_len * 2 );
  dst.push( BVH_PLACEHOLDER );
  dst.push( BVH_PLACEHOLDER ); // Ignore. This makes sure 2 children fit in a cache-line

  if rep_len == 0 {
    // Keep the placeholder
    (num_infinite, dst)
  } else {
    let mut tmp_bins = BinResult::new_many( num_bins, rep_len );
    let reps_aabb = aabb( &reps ).unwrap( );
    dst[0] = subdivide( &mut dst, &mut reps, 0, rep_len, &reps_aabb, &mut tmp_bins );

    for i in 0..reps.len( ) {
      shapes[ i + num_infinite ] = reps[ i ].shape.clone( );
    }

    (num_infinite, dst)
  }
}

// Returns true if the BVH is valid. (See `BVHNode::verify()`)
fn verify_bvh( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode > ) -> bool {
  let a = verify_bvh_bounds( shapes, num_infinite, bvh, 0 ).is_some( );
  let mut contained = vec![false; shapes.len()-num_infinite];
  verify_bvh_contains( &mut contained, bvh, 0 );

  let mut has_all = true;
  for c in contained {
    has_all = has_all && c;
  }

  a && has_all
}

// Sets `true` into `contained` for each element that is contained in the BVH rooted at node `i`.
fn verify_bvh_contains( contained : &mut [bool], bvh : &Vec< BVHNode >, i : usize ) {
  if bvh[ i ].count == 0 { // node
    verify_bvh_contains( contained, bvh, bvh[ i ].left_first as usize );
    verify_bvh_contains( contained, bvh, bvh[ i ].left_first as usize + 1 );
  } else { // leaf
    let first = bvh[ i ].left_first;
    for i in first..(first + bvh[ i ].count) {
      contained[ i as usize ] = true;
    }
  }
}

// Returns `Some(..)` if the bounds of the BVH rooted at node `i` contains the
//   bounds of its children; and this is recursively true for their children.
fn verify_bvh_bounds( shapes : &[Rc< dyn Tracable >], num_infinite : usize, bvh : &Vec< BVHNode >, i : usize ) -> Option< AABB > {
  let n = &bvh[ i ];
  let bounds = &n.bounds;

  if n.count == 0 { // node
    let left_index = n.left_first as usize;

    if let Some( lb ) = verify_bvh_bounds( shapes, num_infinite, bvh, left_index ) {
      if let Some( rb ) = verify_bvh_bounds( shapes, num_infinite, bvh, left_index + 1 ) {
        let b = lb.join( &rb );
        if bounds.contains( &b ) {
          Some( *bounds )
        } else {
          None
        }
      } else {
        None
      }
    } else {
      None
    }
  } else { // leaf
    let offset = n.left_first as usize;
    let size = n.count as usize;

    let mut cum_bounds = shapes[ num_infinite+offset ].aabb( ).unwrap( );
    for i in (num_infinite+offset)..(num_infinite+offset+size) {
      if let Some( b ) = shapes[ i ].aabb( ) {
        if !bounds.contains( &b ) {
          return None;
        }
        cum_bounds = cum_bounds.join( &b );
      } else {
        return None;
      }
    }
    Some( *bounds )
  }
}

// Returns the depth of the BVH (See `BVHNode::depth(..)`)
fn bvh_depth( nodes : &Vec< BVHNode > ) -> u32 {
  depth_rec( nodes, 0 )
}

// Recursively finds the depth of the BVH rooted in node `i`.
fn depth_rec( nodes : &Vec< BVHNode >, i : usize ) -> u32 {
  let n = &nodes[ i ];
  if n.count != 0 { // leaf
    0
  } else { // Node whose check is kept
    let left = n.left_first;
    1 + depth_rec( nodes, left as usize ).max( depth_rec( nodes, left as usize + 1 ) )
  }
}

// Subdivide the region in `shapes` (marked by `offset` and `length`)
// It splits along the largest axis
// (Slices are not used, as absolute offsets are stored in the BVH)
fn subdivide( dst         : &mut Vec< BVHNode >
            , shapes      : &mut [ShapeRep]
            , offset      : usize
            , length      : usize
            , parent_aabb : &AABB
              // Storage for the bins that is pre-allocated
            , tmp_bins    : &mut BinResult< ShapeRep >
            ) -> BVHNode {
  match split( &mut shapes[offset..(offset+length)], parent_aabb, tmp_bins ) {
    SplitRes::DoSplit( split_index, l_aabb, r_aabb ) => {
      let bvh_left_id = dst.len( );
      dst.push( BVH_PLACEHOLDER );
      dst.push( BVH_PLACEHOLDER );

      dst[ bvh_left_id + 0 ] = subdivide( dst, shapes, offset, split_index, &l_aabb, tmp_bins );
      dst[ bvh_left_id + 1 ] = subdivide( dst, shapes, offset + split_index, length - split_index, &r_aabb, tmp_bins );

      BVHNode::node( l_aabb.join( &r_aabb ), bvh_left_id as u32 )
    },
    SplitRes::DontSplit( parent_aabb ) => {
      // Otherwise, don't split and make a leaf for the shapes
      BVHNode::leaf( parent_aabb, offset as u32, length as u32 )
    }
  }
}

// The result for a split
enum SplitRes {
  // (semi-optimal split index in shapes array, left AABB, right AABB)
  DoSplit( usize, AABB, AABB ),
  // No utility-improving split was found. Returns the AABB of all contained shapes
  DontSplit( AABB )
}

// Find the best split among all 3 axes, accepts the one with the best utility.
//   But only if that utility is better (i.e. lower) than the parent's
// When a split is performed, the shapes in `shapes` are "reordered"
//   That is, all nodes in the left AABB are to the left of the split-index
//   and the nodes in the right AABB are to the right of the split-index
fn split( shapes      : &mut [ShapeRep]
        , parent_aabb : &AABB
          // Storage for the bins that is pre-allocated
        , tmp_bins    : &mut BinResult< ShapeRep >
        ) -> SplitRes {
  if shapes.len( ) <= 1 {
    SplitRes::DontSplit( aabb( shapes ).unwrap( ) )
  } else if let Some( ( l_aabb, r_aabb, index ) ) =
      split_longest_axis( shapes, parent_aabb, tmp_bins ) {

    let utility = l_aabb.surface( ) * (index as f32) + r_aabb.surface( ) * ( shapes.len( ) - index ) as f32;
    let parent_aabb = l_aabb.join( &r_aabb );
    let parent_utility = parent_aabb.surface( ) * shapes.len( ) as f32;

    if utility < parent_utility {
      tmp_bins.write_to( shapes );
      SplitRes::DoSplit( index, l_aabb, r_aabb )
    } else {
      SplitRes::DontSplit( parent_aabb )
    }
  } else {
    SplitRes::DontSplit( aabb( shapes ).unwrap( ) )
  }
}

// Splits along the longest axis
// If this split the shapes in `shapes` are placed on the appropriate side of
//   the split index.
fn split_longest_axis(
      shapes      : &mut [ShapeRep]
    , parent_aabb : &AABB
    , dst_bins    : &mut BinResult< ShapeRep >
    ) -> Option< (AABB, AABB, usize) > {

  let x_size = parent_aabb.x_max - parent_aabb.x_min;
  let y_size = parent_aabb.y_max - parent_aabb.y_min;
  let z_size = parent_aabb.z_max - parent_aabb.z_min;

  if x_size > y_size {
    if x_size > z_size {
      split_axis( shapes, |s| s.location.x, dst_bins )
    } else {
      split_axis( shapes, |s| s.location.z, dst_bins )
    }
  } else if y_size > z_size {
    split_axis( shapes, |s| s.location.y, dst_bins )
  } else {
    split_axis( shapes, |s| s.location.z, dst_bins )
  }
}

/// Find the optimal split between bins in O(n*k) time along the axis represented by F.
/// Here `k` is the number of bins and `n` the number of shapes.
/// As `k` is constant, the time complexity can be considered O(n).
/// ASSERT: `shapes` must not be empty
fn split_axis< FAxis : Fn(&ShapeRep) -> f32 >(
      shapes   : &[ShapeRep]
    , f_axis   : FAxis
    , dst_bins : &mut BinResult< ShapeRep >
    ) -> Option< (AABB, AABB, usize) > {
  let num_bins = dst_bins.num_bins( );
  assert!( num_bins > 1 );

  if shapes.len( ) <= 1 {
    return None;
  }

  // Put the shapes in their appropriate bins
  if !bin( shapes, f_axis, dst_bins ) {
    return None;
  };

  // Now find the optimal split between the bins
  //   (which is semi-optimal between the shapes in the bins)
  let mut l = 0;
  let mut r = num_bins - 1;

  let mut l_aabb = aabb( &dst_bins.bins[ l ] ).unwrap( );
  let mut r_aabb = aabb( &dst_bins.bins[ r ] ).unwrap( );

  let mut l_cnt = dst_bins.bins[ l ].len( );
  let mut r_cnt = dst_bins.bins[ r ].len( );

  let mut ln_aabb = l_aabb.join_maybe( &aabb( &dst_bins.bins[ l + 1 ] ) );
  let mut rn_aabb = r_aabb.join_maybe( &aabb( &dst_bins.bins[ r - 1 ] ) );

  let mut ln_cnt = l_cnt + dst_bins.bins[ l + 1 ].len( );
  let mut rn_cnt = r_cnt + dst_bins.bins[ r - 1 ].len( );

  while l + 1 < r {
    // Smaller utility is better
    if (ln_aabb.surface( ) * ln_cnt as f32 + r_aabb.surface( ) * r_cnt as f32) <
         (l_aabb.surface( ) * l_cnt as f32 + rn_aabb.surface( ) * rn_cnt as f32) {
      // Prefer the new left
      l += 1;
      l_aabb = ln_aabb;
      l_cnt  = ln_cnt;

      if l + 1 < r {
        ln_aabb = l_aabb.join_maybe( &aabb( &dst_bins.bins[ l + 1 ] ) );
        ln_cnt  = l_cnt + dst_bins.bins[ l + 1 ].len( );
      }
    } else {
      // Prefer the new right
      r -= 1;
      r_aabb = rn_aabb;
      r_cnt  = rn_cnt;

      if l + 1 < r {
        rn_aabb = r_aabb.join_maybe( &aabb( &dst_bins.bins[ r - 1 ] ) );
        rn_cnt  = r_cnt + dst_bins.bins[ r - 1 ].len( );
      }
    }
  }

  Some( ( l_aabb, r_aabb, l_cnt ) )
}

/// Returns the number of infinitely-sized shapes, and puts them *left* in `shapes`.
/// For the non-infinite shapes, returns a vector of `ShapeRep`s.
///
/// WARNING: The order of `shapes` and `dst` is *not* the same
fn shape_reps( shapes : &mut [Rc< dyn Tracable >] ) -> ( usize, Vec< ShapeRep > ) {
  let mut num_infinite = 0;
  let mut dst : Vec< ShapeRep > = Vec::with_capacity( shapes.len( ) );
  for i in 0..shapes.len( ) {
    let shape = &shapes[ i ];
    if let Some( bounds ) = shape.aabb( ) {
      if let Some( location ) = shape.location( ) {
        dst.push( ShapeRep { shape: shape.clone( ), location, bounds } )
      } else {
        shapes.swap( num_infinite, i );
        num_infinite += 1;
      }
    } else {
      shapes.swap( num_infinite, i );
      num_infinite += 1;
    }
  }
  ( num_infinite, dst )
}

// Returns the AABB around all shapes in `s`.
fn aabb( s : &[ShapeRep] ) -> Option< AABB > {
  if s.len( ) == 0 {
    None
  } else {
    let mut res = s[ 0 ].bounds;
    for i in 1..s.len( ) {
      res = res.join( &s[ i ].bounds );
    }
    Some( res )
  }
}

// Bin all elements in `xs` into bins in `dst`.
// The function `f` determines the "value" of the nodes
// The nodes are uniformly binned by this value.
fn bin< T: Clone, F: Fn(&T) -> f32 >( xs : &[T], f : F, dst : &mut BinResult< T > ) -> bool {
  let n = xs.len( );
  let mut min_v = f( &xs[ 0 ] );
  let mut max_v = f( &xs[ 0 ] );
  for i in 1..n {
    let v = f( &xs[ i ] );
    min_v = min_v.min( v );
    max_v = max_v.max( v );
  }

  if min_v == max_v {
    // It makes no sense to bin this
    false
  } else {
    let num_bins = dst.num_bins( );
    dst.clear( );

    let segment_width = ( max_v - min_v ) / ( num_bins as f32 );
    for i in 0..n {
      let v = f( &xs[ i ] );
      let segment_id = ( ( ( v - min_v ) as f32 / segment_width ).floor( ) as usize ).min( num_bins - 1 );
      dst.bins[ segment_id ].push( xs[ i ].clone( ) );
    }
    true
  }
}

/// A set of bins with elements of type `T`
struct BinResult< T > {
  bins : Vec< Vec< T > >
}

impl< T: Clone > BinResult< T > {
  /// Returns a new set of bins with the provided capacity and bin-count
  fn new_many( num_bins : usize, bin_size : usize ) -> BinResult< T > {
    let mut bins = Vec::with_capacity( num_bins );
    for _i in 0..num_bins {
      bins.push( Vec::with_capacity( bin_size ) );
    }
    BinResult { bins }
  }

  /// Clears all bins
  fn clear( &mut self ) {
    for i in 0..self.bins.len( ) {
      self.bins[ i ].clear( );
    }
  }

  /// Writes the contents of the bins sequentially to `dst`
  fn write_to( &self, dst : &mut [T] ) {
    let mut i = 0;
    for b in &self.bins {
      for v in b {
        dst[ i ] = v.clone( );
        i += 1;
      }
    }
  }

  /// Returns the number of bins
  fn num_bins( &self ) -> usize {
    self.bins.len( )
  }
}
