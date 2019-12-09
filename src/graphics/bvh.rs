
pub struct BVHNode {
  Leaf { bounds :: AABB, offset : usize, size : usize }
  Node { bounds :: AABB, leftIndex : usize }
}

struct Config {
  dst     :: Vec< BVHNode >,
  shapes1 :: Vec< ShapeRep >,
  shapes2 :: Vec< ShapeRep >
}

struct ShapeRep {
  index    :: usize,
  location :: Vec3,
  bounds   :: AABB
}

pub fn build_bvh( shapes : &mut Vec< Box< dyn Tracable > > ) {
  let n = shapes.size( );

  let mut shapes_rep = Vec::with_capacity( n );

  for i in 0..n {
    shapes_rep.push(
      ShapeRep {
        index    = i
      , location = shapes[ i ].location( )
      , bounds   = shapes[ i ].bounds( )
      }
    )
  }

  let mut config =
    Config {
      dst     = vec![ BVHNode { }; 2 * n - 1 ]
    , shapes1 = shapes_rep
    , shapes2 = shapes_rep
    }
  bvh( &mut config, 0, n )
}

fn bvh( config : &mut Config, offset : usize, length : usize ) {
  let n = config.shapes.size( );

  // Sort by x
  config.shapes1.sort_by_key( |a| s.location.x );
  let (x_split, x_area) = split_plane( config.shapes1 );

  // Sort by y
  let mut y_sort = range( offset, length );
  y_sort.sort_by_key( |a| config.shapes[a].location.y );
  let (y_split, y_area, _) = split_plane( config.shapes, &y_sort );

  // Sort by z
  let mut z_sort = range( offset, length );
  z_sort.sort_by_key( |a| config.shapes[a].location.z );
  let (z_split, z_area, _) = split_plane( config.shapes, &z_sort );

  let split =
    if x_area < y_area {
      if z_area < x_area {
        2 // z
      } else {
        0 // x
      }
    } else if z_area < y_area {
      2 // y
    } else {
      1
    };

  if split == 0 {
    if x_area < p_area {
      apply_index( conf.shapes, x_sort );
      bvh( config, offset, x_split );
      bvh( config, offset + x_split, length - x_split );
    } // otherwise split not worth it
  } else if split == 1 {
    if y_area < p_area {
      apply_index( conf.shapes, y_sort );
      bvh( config, offset, y_split );
      bvh( config, offset + y_split, length - y_split );
    } // otherwise split not worth it
  } else {
    if z_area < p_area {
      apply_index( conf.shapes, z_sort );
      bvh( config, offset, z_split );
      bvh( config, offset + z_split, length - z_split );
    } // otherwise split not worth it
  }
}

// Split *before* the returned index
fn split_plane( shapes : &[ShapeRep] ) -> ( usize, f32 ) {
  let mut l = 0;
  let mut r = range.length - 1;

  let l_area = shapes[ l ].aabb( );
  let r_area = shapes[ r ].aabb( );

  let new_l_area = l_area.join( shapes[ l + 1 ].aabb( ) );
  let new_r_area = r_area.join( shapes[ r - 1 ].aabb( ) );

  // Grow the smallest area. This ensures their sum is the smallest
  while l + 1 < r {
    while l + 1 < r && new_l_area.size( ) < new_r_area.size( ) {
      l_area = new_l_area;
      l += 1;
      new_l_area = l_area.join( shapes[ l ] );
    }
    while l + 1 < r && new_r_area.size( ) < new_l_area.size( ) {
      r_area = new_r_area;
      r -= 1;
      new_r_area = r_area.join( shapes[ r ] );
    }
  }

  let num_left  = l + 1;
  let num_right = shapes.length - r;

  ( r, l_area.size( ) * num_left as f32 + r_area.size( ) * num_right as f32 )
}
