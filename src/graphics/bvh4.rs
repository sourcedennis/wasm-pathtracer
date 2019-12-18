use crate::graphics::{AABB, BVHNode};

#[derive(Copy,Clone,Debug)]
pub struct BVHNode4 {
  pub bounds     : AABB,
  pub left_first : u32,
  // Use the lower 2 bits as child node count
  // 00 = leaf node;
  // 01 = 2 childen;
  // 10 = 3 children;
  // 11 = 4 children;
  // A node with 1 child node does not exists. For obvious reasons =)
  pub count      : u32
}

static PLACEHOLDER : BVHNode4 = BVHNode4 { bounds: AABB::EMPTY, left_first: 0, count: 0 };

// A very cache/memory-inefficient structure used during building only
enum BVHNodeB {
  Node { bounds : AABB, children : Vec< Box< BVHNodeB > > },
  Leaf { bounds : AABB, offset : usize, count : usize }
}

impl BVHNode4 {
  pub fn from_bvh( bvh2 : &Vec< BVHNode > ) -> Vec< BVHNode4 > {
    let mut t = build_tree( bvh2, 0 );
    collapse( &mut t );

    let mut dst = Vec::with_capacity( bvh2.len( ) );
    dst.push( PLACEHOLDER );
    dst[ 0 ] = to_bvh4( &mut dst, t );
    return dst;
  }

  pub fn depth( bvh : &Vec< BVHNode4 > ) -> usize {
    BVHNode4::depth_r( bvh, 0 )
  }

  fn depth_r( bvh : &Vec< BVHNode4 >, i : usize ) -> usize {
    let num_children = ( bvh[ i ].count & 0x3 ) + 1;
    if num_children == 1 { // leaf
      0
    } else { // node
      let mut max_d = 0;
      let left = bvh[ i ].left_first;
      for j in 0..num_children {
        max_d = max_d.max( 1 + BVHNode4::depth_r( bvh, ( left + j ) as usize ) );
      }
      max_d
    }
  }
}

fn build_tree( bvh2 : &Vec< BVHNode >, i : usize ) -> BVHNodeB {
  let n = &bvh2[ i ];
  if n.count == 0 { // Node
    let c1 = build_tree( bvh2, n.left_first as usize );
    let c2 = build_tree( bvh2, n.left_first as usize + 1 );
    BVHNodeB::Node { bounds: n.bounds, children: vec![ Box::new( c1 ), Box::new( c2 ) ] }
  } else {
    BVHNodeB::Leaf { bounds: n.bounds, offset: n.left_first as usize, count: n.count as usize }
  }
}

fn to_bvh4( dst : &mut Vec< BVHNode4 >, tree : BVHNodeB ) -> BVHNode4 {
  match tree {
    BVHNodeB::Node { bounds, children } => {
      let index = dst.len( );
      let num_children = children.len( );
      for _i in 0..children.len( ) {
        dst.push( PLACEHOLDER );
      }
      let mut j = 0;
      for c in children {
        let new_node = to_bvh4( dst, *c );
        dst[ index + j ] = new_node;
        j += 1;
      }
      // assert( children.len( ) >= 2 )
      BVHNode4 { bounds: bounds, left_first: index as u32, count: num_children as u32 - 1 }
    },
    BVHNodeB::Leaf { bounds, offset, count } => {
      BVHNode4 { bounds, left_first: offset as u32, count: ( count << 2 ) as u32 }
    }
  }
}

fn collapse( t : &mut BVHNodeB ) {
  
}
