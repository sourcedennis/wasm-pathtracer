use crate::graphics::AABB;
use crate::graphics::ray::Tracable;
use crate::math::Vec3;
use crate::math::EPSILON;
use std::rc::Rc;

#[derive(Copy,Clone,Debug)]
pub struct BVHNode {
  pub bounds     : AABB,
  pub left_first : u32,
  pub count      : u32
}

impl BVHNode {
  pub fn leaf( bounds : AABB, offset : u32, count : u32 ) -> BVHNode {
    BVHNode { bounds, left_first: offset, count }
  }

  pub fn node( bounds : AABB, first : u32 ) -> BVHNode {
    BVHNode { bounds, left_first: first, count: 0 }
  }
}

#[derive(Clone)]
struct ShapeRep {
  shape    : Rc< dyn Tracable >,
  location : Vec3,
  bounds   : AABB
}

static BVH_PLACEHOLDER: BVHNode =
  BVHNode {
    bounds:     AABB::EMPTY
  , left_first: 0
  , count:      0
  };

type Utility = f32;

// Uses O(n log n) time
pub fn build_bvh( shapes : &mut Vec< Rc< dyn Tracable > >, num_bins : usize ) -> (usize, Vec< BVHNode >) {
  let (num_infinite, mut reps) = shape_reps( shapes );
  let mut dst  = Vec::with_capacity( shapes.len( ) * 2 - 1 );

  let rep_len = reps.len( );
  dst.push( BVH_PLACEHOLDER );

  if rep_len == 0 {
    // Keep the placeholder
    (num_infinite, dst)
  } else {
    let mut tmp_bins = BinResult::new_many( num_bins, rep_len );
    let reps_aabb = aabb( &reps ).unwrap( );
    dst[0] = subdivide( &mut dst, &mut reps, 0, rep_len, num_bins, &reps_aabb, &mut tmp_bins );
    //dst[0] = BVHNode::Leaf { bounds: aabb( &reps ).unwrap( ), offset: 0, size: reps.len( ) };
  
    for i in 0..reps.len( ) {
      shapes[ i + num_infinite ] = reps[ i ].shape.clone( );
    }
  
    (num_infinite, dst)
  }
}

pub fn verify_bvh( shapes : &Vec< Rc< dyn Tracable > >, num_infinite : usize, bvh : &Vec< BVHNode > ) -> bool {
  let a = verify_bvh_bounds( shapes, num_infinite, bvh, 0 ).is_some( );
  let mut contained = vec![false; shapes.len()-num_infinite];
  verify_bvh_contains( &mut contained, bvh, 0 );

  let mut has_all = true;
  for c in contained {
    has_all = has_all && c;
  }

  a && has_all
}

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

fn verify_bvh_bounds( shapes : &Vec< Rc< dyn Tracable > >, num_infinite : usize, bvh : &Vec< BVHNode >, i : usize ) -> Option< AABB > {
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

pub fn bvh_depth( nodes : &Vec< BVHNode > ) -> u32 {
  depth_rec( nodes, 0 )
}

fn depth_rec( nodes : &Vec< BVHNode >, i : usize ) -> u32 {
  /*match nodes[ i ] {
    BVHNode::Leaf { .. } => 0,
    BVHNode::Node { left_index, .. } =>
    1 + depth_rec( nodes, left_index ).max( depth_rec( nodes, left_index + 1 ) )
  }*/
  let n = &nodes[ i ];
  if n.count == 0 { // node
    let left = n.left_first;
    1 + depth_rec( nodes, left as usize ).max( depth_rec( nodes, left as usize + 1 ) )
  } else { // leaf
    0
  }
}

// `offset` and `length` index into `shapes`. Slices cannot be used, as absolute offsets are stored in the BVH.
fn subdivide( dst            : &mut Vec< BVHNode >
            , shapes         : &mut Vec< ShapeRep >
            , offset         : usize
            , length         : usize
            , num_bins       : usize
            , parent_aabb    : &AABB
            , tmp_bins       : &mut BinResult< ShapeRep >
            ) -> BVHNode {
  match split( &mut shapes[offset..(offset+length)], num_bins, parent_aabb, tmp_bins ) {
    SplitRes::DoSplit( split_index, l_aabb, r_aabb ) => {
      let bvh_left_id = dst.len( );
      dst.push( BVH_PLACEHOLDER );
      dst.push( BVH_PLACEHOLDER );
  
      dst[ bvh_left_id + 0 ] = subdivide( dst, shapes, offset, split_index, num_bins, &l_aabb, tmp_bins );
      dst[ bvh_left_id + 1 ] = subdivide( dst, shapes, offset + split_index, length - split_index, num_bins, &r_aabb, tmp_bins );
  
      //BVHNode::Node { bounds: l_aabb.join( &r_aabb ), left_index: bvh_left_id }
      BVHNode::node( l_aabb.join( &r_aabb ), bvh_left_id as u32 )
    },
    SplitRes::DontSplit( parent_aabb ) => {
      // Otherwise, don't split and make a leaf for the shapes
      //BVHNode::Leaf { bounds: parent_aabb, offset, size: length }
      BVHNode::leaf( parent_aabb, offset as u32, length as u32 )
    }
  }
}

enum SplitRes {
  DoSplit( usize, AABB, AABB ),
  DontSplit( AABB )
}

// Find the best split among all 3 axes, accepts the one with the best utility.
//   But only if that utility is better (i.e. lower) than the parent's
fn split( shapes      : &mut [ShapeRep]
        , num_bins    : usize
        , parent_aabb : &AABB
        , tmp_bins    : &mut BinResult< ShapeRep >
        ) -> SplitRes {
  // (x_bins, xl_aabb, xr_aabb, x_index)
  if shapes.len( ) <= 1 {
    SplitRes::DontSplit( aabb( shapes ).unwrap( ) )
  } else if let Some( ( l_aabb, r_aabb, index ) ) =
      split_longest_axis( shapes, num_bins, parent_aabb, tmp_bins ) {

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

fn split_longest_axis(
      shapes      : &mut [ShapeRep]
    , num_bins    : usize
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

// leq function
fn best_of< 'a, T, FVal: Fn(&T) -> f32 >( vals : &'a [Option< T >], f_val : FVal ) -> Option< &'a T > {
  let mut val : Option< &'a T > = None;
  for v in vals {
    if let Some( ref cv ) = val {
      if let Some( av ) = v {
        if f_val( av ) < f_val( cv ) {
          val = Some( av );
        }
      }
    } else if let Some( av ) = v {
      val = Some( av );
    }
  }
  val
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

  if !bin( shapes, f_axis, dst_bins ) {
    return None;
  };

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

/// Returns the number of infinitely-sides shapes, and puts them *left*
/// For the non-infinite shapes, returns a vector of `ShapeRep`s
///
/// WARNING: The order of `shapes` and `dst` is *not* the same
fn shape_reps( mut shapes : &mut Vec< Rc< dyn Tracable > > ) -> ( usize, Vec< ShapeRep > ) {
  let mut num_infinite = 0;
  let mut dst = Vec::with_capacity( shapes.len( ) );
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

struct BinResult< T > {
  bins : Vec< Vec< T > >
}

impl< T: Clone > BinResult< T > {
  fn new_many( num_bins : usize, bin_size : usize ) -> BinResult< T > {
    let mut bins = Vec::with_capacity( num_bins );
    for _i in 0..num_bins {
      bins.push( Vec::with_capacity( bin_size ) );
    }
    BinResult { bins }
  }

  fn clear( &mut self ) {
    for i in 0..self.bins.len( ) {
      self.bins[ i ].clear( );
    }
  }

  fn write_to( &self, dst : &mut [T] ) {
    let mut i = 0;
    for b in &self.bins {
      for v in b {
        dst[ i ] = v.clone( );
        i += 1;
      }
    }
  }

  fn num_bins( &self ) -> usize {
    self.bins.len( )
  }
}


/*fn optimal_bvh( dst : &mut Vec< BVHNode >, shapes : &mut [ShapeRep], offset : usize, size : usize ) {
  // Find optimal x

  // Find optimal y

  // Find optimal z
}*/
