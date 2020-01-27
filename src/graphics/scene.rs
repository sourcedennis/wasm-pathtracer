// External imports
use std::f32::{INFINITY};
use std::rc::Rc;
// Local imports
use crate::graphics::{Color3, AABB};
use crate::graphics::ray::{Ray, Hit, Tracable};
use crate::graphics::lights::Light;
use crate::math::{Vec3, EPSILON};
use crate::graphics::{BVHNode, BVHNode4};

// A scene description for a path tracer

/// The possible BVH representations
enum BVHEnum {
  BVH2( usize, Vec< BVHNode > ),
  BVH4( usize, Vec< BVHNode4 > ),
  BVHNone
}

pub enum LightEnum {
  /// Point light
  Point( Light ),
  /// Area light. Index in the `shapes` array (of `Scene`)
  Area( usize ) 
}

/// A Scene consists of shapes and lights
/// The camera is *not* part of the scene
///
/// (For specific scenes, look at the `/scenes.rs` file)
pub struct Scene {
  pub background : Color3,
  pub lights     : Vec< LightEnum >,
  pub shapes     : Vec< Rc< dyn Tracable > >,
      bvh        : BVHEnum
}

type ShapeId = usize;

impl Scene {
  /// Constructs a new scene with the specified lights and shapes
  #[allow(unused)]
  pub fn new( background : Color3
            , lights     : Vec< Light >
            , shapes     : Vec< Rc< dyn Tracable > >
            ) -> Scene {
    let mut num_area_lights = 0;

    for s in &shapes {
      num_area_lights += if s.is_emissive( ) { 1 } else { 0 }
    }

    let mut light_enums = Vec::with_capacity( lights.len( ) + num_area_lights );

    for l in lights {
      light_enums.push( LightEnum::Point( l ) );
    }

    let mut scene = Scene { background, lights: vec![], bvh: BVHEnum::BVHNone, shapes };
    scene.rebuild_bvh( 16, false );

    for i in 0..scene.shapes.len( ) {
      if scene.shapes[ i ].is_emissive( ) {
        light_enums.push( LightEnum::Area( i ) );
      }
    }

    Scene { background, lights: light_enums, bvh: scene.bvh, shapes: scene.shapes }
  }

  /// Rebuilds the BVH, and returns the number of nodes
  /// The BVH is build with the provided number of bins in `num_bins`.
  /// If `is_bvh4` is true, a 4-way BVH is built. Otherwise a 2-way BVH is built.
  /// 
  /// To disable the BVH see `Scene::disable_bvh(..)`
  pub fn rebuild_bvh( &mut self, num_bins : usize, is_bvh4 : bool ) -> u32 {
    let (num_inf, bvh) = BVHNode::build( &mut self.shapes, num_bins );
    let num_nodes;

    if is_bvh4 {
      let bvh4 = BVHNode4::collapse( &bvh );
      num_nodes = BVHNode4::node_count( &bvh4 );
      
      if !BVHNode4::verify( &self.shapes, num_inf, &bvh4) {
        // This should not happen, but panicing here is better than later
        panic!( "WHAT" );
      }
      
      self.bvh = BVHEnum::BVH4( num_inf, bvh4 );
    } else {
      num_nodes = BVHNode::node_count( &bvh );
      self.bvh = BVHEnum::BVH2( num_inf, bvh );
    }

    num_nodes as u32
  }

  /// Disables the BVH. On the next render, no BVH is used.
  pub fn disable_bvh( &mut self ) {
    self.bvh = BVHEnum::BVHNone;
  }

  /// Is the ray from point `p` to `point_on_shape` occluded by anything (other than `shape` itself)?
  pub fn shadow_ray( &self, p : &Vec3, point_on_shape : &Vec3, shape : Option< ShapeId > ) -> (usize, bool) {
    let mut dir = *point_on_shape - *p;
    let dir_len = dir.len( );
    dir         = dir / dir_len;
    let ray     = Ray::new( *p + dir * EPSILON, dir );

    let (num_bvh_hits, res) = self.trace_g( &ray );

    if let Some( ( dis, shape_id ) ) = res {
      if dis < dir_len {
        if let Some( light_shape_id ) = shape {
          if shape_id == light_shape_id {
            // It's only "occluded" by the shape to which the shadow ray was cast
            ( num_bvh_hits, false )
          } else {
            // It is occluded by some other shape
            ( num_bvh_hits, true )
          }
        } else {
          // It is occluded by some other shape
          ( num_bvh_hits, true )
        }
      } else {
        // The hit is beyond `point_on_shape`
        ( num_bvh_hits, false )
      }
    } else {
      ( num_bvh_hits, false ) // Not occluded
    }
  }

  /// Traces a  ray into the scene and returns the first element hit
  /// The first tuple-element is the number of BVH node traversals
  pub fn trace( &self, ray : &Ray ) -> (usize, Option< Hit >) {
    let (d, t) = self.trace_g( ray );
    if let Some( (_, shape_id) ) = t {
      (d, self.shapes[ shape_id ].trace( ray ))
    } else {
      (d, None)
    }
  }

  /// Traces a ray into the scene and returns the distance to the first element
  /// hit. Typically this is faster than calling `Scene::trace(..)` as
  /// computation of properties (such as normals) is avoided.
  /// 
  /// The first tuple-element is the number of BVH node traversals
  pub fn trace_simple( &self, ray : &Ray ) -> (usize, Option< f32 >) {
    let (d, res) = self.trace_g( ray );
    if let Some( (dis, _) ) = res {
      (d, Some( dis ))
    } else {
      (d, None)
    }
  }

  /// General trace function. It returns the distance and reference to the first object hit.
  /// The first tuple-element is the number of BVH node traversals
  fn trace_g< 'a >( &'a self, ray : &Ray ) -> (usize, Option< (f32, ShapeId) >) {
    match &self.bvh {
      BVHEnum::BVH2( numinf, bvh ) => {
        if let Some( h1 ) = trace_shapes( ray, &self.shapes[..*numinf] ) {
          let (d2, h2) = traverse_bvh_guarded( ray, *numinf, &bvh, &self.shapes, 0, h1.0 );
          (d2, closest( Some( h1 ), h2 ))
        } else {
          traverse_bvh_guarded( ray, *numinf, &bvh, &self.shapes, 0, INFINITY )
        }
      },
      BVHEnum::BVH4( numinf, bvh ) => {
        if let Some( h1 ) = trace_shapes( ray, &self.shapes[..*numinf] ) {
          let (d2, h2) = traverse_bvh4( ray, *numinf, &bvh, &self.shapes, 0, h1.0 );
          (d2, closest( Some( h1 ), h2 ))
        } else {
          traverse_bvh4( ray, *numinf, &bvh, &self.shapes, 0, INFINITY )
        }
      },
      _ => {
        (0, trace_shapes( ray, &self.shapes ))
      }
    }
  }
}

/// Traverses the 2-way BVH starting at node `node_i`.
/// `node_i` is only entered if its AABB hits the ray, which is checked.
///   (That check being the "guard")
#[allow(dead_code)]
fn traverse_bvh_guarded< 'a >(
      ray     : &Ray
    , num_inf : usize
    , bvh     : &[BVHNode]
    , shapes  : &'a [Rc< dyn Tracable >]
    , node_i  : usize
    , max_dis : f32 ) -> (usize, Option< (f32, ShapeId) >) {

  let node   = &bvh[ node_i ];
  let bounds = &node.bounds;

  if let Some( h ) = bounds.hit( ray ) {
    if h < max_dis {
      let (t2, res) = traverse_bvh( ray, num_inf, bvh, shapes, node_i, max_dis );
      (t2 + 1, res)
    } else {
      (1, None)
    }
  } else {
    (1, None)
  }
}

/// Traverses a 2-way BVH starting at node `node_i`
/// The AABB of `node_i` are *not* intersected with the ray, thus only call
///   this if `node_i`'s AABB is known to intersect. (Or if checking this is
///   considered to be not worth it)
fn traverse_bvh< 'a >(
      ray     : &Ray
    , num_inf : usize
    , bvh     : &[BVHNode]
    , shapes  : &'a [Rc< dyn Tracable >]
    , node_i  : usize
    , max_dis : f32 ) -> (usize, Option< (f32, ShapeId) >) {

  let node = &bvh[ node_i ];

  // A big nested if that performs ordered traversal of the BVH very fast
  // Basically, any loop for this node is manually unwound.
  if node.count != 0 { // leaf
    let offset = node.left_first as usize;
    let size = node.count as usize;

    if let Some( ( dis, res ) ) = trace_shapes_md( ray, &shapes[(num_inf+offset)..(num_inf+offset+size)], max_dis ) {
      (1, Some((dis, num_inf+offset+res)))
    } else {
      ( 1, None )
    }
  } else { // node
    let left_index = node.left_first as usize;

    if let Some( left_dis ) = aabb_distance( ray, &bvh[ left_index ].bounds, max_dis ) {
      if let Some( right_dis ) = aabb_distance( ray, &bvh[ left_index + 1 ].bounds, max_dis ) {
        if left_dis < right_dis { // traverse left first
          let (ld, tl) = traverse_bvh( ray, num_inf, bvh, shapes, left_index, max_dis );
          if let Some( ( lshape_dis, lshape ) ) = tl {
            if lshape_dis < right_dis {
              (1 + ld, Some( ( lshape_dis, lshape ) ) )
            } else {
              let (rd, tr) = traverse_bvh( ray, num_inf, bvh, shapes, left_index + 1, lshape_dis );
              if let Some( rhit ) = tr {
                (1 + ld + rd, Some( rhit ))
              } else {
                (1 + ld + rd, Some( ( lshape_dis, lshape ) ))
              }
            }
          } else { // left doesn't hit
            let (rd, tr) = traverse_bvh( ray, num_inf, bvh, shapes, left_index + 1, max_dis );
            (1 + ld + rd, tr)
          }
        } else { // traverse right first
          let (rd, tr) = traverse_bvh( ray, num_inf, bvh, shapes, left_index + 1, max_dis );
          if let Some( ( rshape_dis, rshape ) ) = tr {
            if rshape_dis < left_dis {
              (1 + rd, Some( ( rshape_dis, rshape ) ) )
            } else {
              let (ld, tl) = traverse_bvh( ray, num_inf, bvh, shapes, left_index, rshape_dis );
              if let Some( lhit ) = tl {
                (1 + ld + rd, Some( lhit ))
              } else {
                (1 + ld + rd, Some( ( rshape_dis, rshape ) ))
              }
            }
          } else { // right doesn't hit
            let (ld, tl) = traverse_bvh( ray, num_inf, bvh, shapes, left_index, max_dis );
            (1 + ld + rd, tl)
          }
        }
      } else { // right doesn't hit
        let (ld, tl) = traverse_bvh( ray, num_inf, bvh, shapes, left_index, max_dis );
        (ld+1, tl)
      }
    } else { // left doesn't hit
      let (rd, tr) = traverse_bvh_guarded( ray, num_inf, bvh, shapes, left_index + 1, max_dis );
      (rd+1, tr)
    }
  }
}

/// Traverses a BVH starting at node `node_i`.
#[allow(dead_code)]
fn traverse_bvh4< 'a >(
      ray         : &Ray
    , num_inf     : usize
    , bvh         : &[BVHNode4]
    , shapes      : &'a [Rc< dyn Tracable >]
    , node_i      : i32
    , mut max_dis : f32 ) -> (usize, Option< (f32, ShapeId) >) {
  

  if node_i < 0 { // leaf
    let ni = unsafe { std::mem::transmute::< i32, u32 >( node_i ) };
    let num_shapes = ( ( ni >> 27 ) & 0x3 ) as usize;
    let shape_index = ( ni & 0x7FFFFFF ) as usize;

    if let Some( ( dis, res ) ) = trace_shapes_md( ray, &shapes[(num_inf+shape_index)..(num_inf+shape_index+num_shapes)], max_dis ) {
      (1, Some((dis, num_inf+shape_index+res)))
    } else {
      ( 1, None )
    }
  } else { // node
    let node = &bvh[ node_i as usize ];
    let num_children  = node.num_children as usize;

    let hits = node.child_bounds.hit( ray ); // The SIMD intersection
    
    // Store and order the children
    let mut children = [ (0, INFINITY), (0, INFINITY), (0, INFINITY), (0, INFINITY) ];
    for i in 0..num_children {
      children[ i ] = ( node.children[ i ], hits.extract( i ) );
    }
    sort_small( &mut children, num_children );

    let (mut num_traversed, mut res) = ( 1, None );

    for i in 0..num_children {
      if children[ i ].1 > max_dis {
        return ( num_traversed, res );
      } else if children[ i ].1 >= 0.0 {
        let ( nt2, res2 ) = traverse_bvh4( ray, num_inf, bvh, shapes, children[ i ].0, max_dis );
  
        if let Some( ( d, _ ) ) = res2 {
          max_dis = d;
          res = res2;
        }
        num_traversed += nt2;
      }
    }

    ( num_traversed, res )
  }
}

/// A fast sorting function for arrays with *at most 4 elements*.
/// The elements are sorted by their second tuple-element
fn sort_small( a : &mut [(i32, f32)], n : usize ) {
  if n == 2 {
    if a[ 1 ].1 < a[ 0 ].1 {
      a.swap( 0, 1 );
    }
  } else if n == 3 {
    if a[ 1 ].1 < a[ 0 ].1 {
      a.swap( 0, 1 );
    }
    if a[ 2 ].1 < a[ 1 ].1 {
      a.swap( 1, 2 );
    }
    if a[ 1 ].1 < a[ 0 ].1 {
      a.swap( 0, 1 );
    }
  } else if n == 4 {
    if a[ 1 ].1 < a[ 0 ].1 {
      a.swap( 0, 1 );
    }
    if a[ 3 ].1 < a[ 2 ].1 {
      a.swap( 2, 3 );
    }
    if a[ 0 ].1 < a[ 2 ].1 {
      if a[ 2 ].1 < a[ 1 ].1 {
        a.swap( 1, 2 );

        if a[ 3 ].1 < a[ 2 ].1 {
          a.swap( 2, 3 );
        }
      }
    } else {
      a.swap( 0, 2 );
      a.swap( 1, 2 );
      
      if a[ 3 ].1 < a[ 1 ].1 {
        a.swap( 1, 3 );
        a.swap( 2, 3 );
      } else if a[ 3 ].1 < a[ 2 ].1 {
        a.swap( 2, 3 );
      }
    }
  }
}

/// Returns the distance from the ray to the AABB, but only if this hit occurs
/// before `max_dis`. If no hit occurs, or if the hit distance is negative, or
/// the hit is after `max_dis`, then None is returned.
fn aabb_distance( ray : &Ray, aabb : &AABB, max_dis : f32 ) -> Option< f32 > {
  if let Some( h ) = aabb.hit( ray ) {
    if h < max_dis {
      Some( h )
    } else {
      None
    }
  } else {
    None
  }
}

/// Returns the element with the lowest distance
fn closest< 'a, T >( a: Option< (f32, T) >
                   , b: Option< (f32, T) >
                   ) -> Option< (f32, T) > {
  if let Some( (av,_) ) = a {
    if let Some( (bv,_) ) = b {
      if av < bv {
        a
      } else {
        b
      }
    } else {
      a
    }
  } else {
    b
  }
}

/// Intersects the ray with all shapes in `shapes`, and returns the element
///   whose distance is closest (but not negative).
fn trace_shapes< 'a >( ray     : &Ray
                     , shapes  : &'a [Rc< dyn Tracable >]
                     ) -> Option< (f32, ShapeId) > {
  let mut best_hit = None;

  for i in 0..shapes.len( ) {
    let s = &shapes[ i ];
    if let Some( new_dis ) = s.trace_simple( ray ) {
      if let Some( ( bhd, _ ) ) = best_hit {
        if 0.0_f32 < new_dis && new_dis < bhd {
          best_hit = Some( ( new_dis, i ) );
        }
      } else {
        best_hit = Some( ( new_dis, i ) );
      }
    }
  }

  best_hit
}

/// Intersects the ray with all shapes in `shapes`, and returns the element
///   whose distance is closest (but not negative). If the found shape is
///   located beyond `max_dis`, then None is returned.
fn trace_shapes_md < 'a >( ray     : &Ray
                         , shapes  : &'a [Rc< dyn Tracable >]
                         , max_dis : f32
                         ) -> Option< (f32, ShapeId) > {
  let mut best_hit = None;

  for i in 0..shapes.len( ) {
    let s = &shapes[ i ];
    if let Some( new_dis ) = s.trace_simple( ray ) {
      if new_dis <= max_dis {
        if let Some( ( bhd, _ ) ) = best_hit {
          if 0.0_f32 < new_dis && new_dis < bhd {
            best_hit = Some( ( new_dis, i ) );
          }
        } else {
          best_hit = Some( ( new_dis, i ) );
        }
      }
    }
  }

  best_hit
}

// Returns only true if a hit occurs and it occurs within at most `sqrt(d_sq)` units
// `d_sq` is the square of the distance - for efficiency reasons
fn is_hit_within_sq( m_hit : Option< f32 >, d_sq : f32 ) -> bool {
  if let Some( h ) = m_hit {
    h * h < d_sq
  } else {
    false
  }
}
