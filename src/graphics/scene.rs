use crate::graphics::{Color3, AABB};
use crate::graphics::ray::{Ray, Hit, Tracable};
use crate::graphics::lights::Light;
use crate::math::{Vec3, EPSILON};
use crate::graphics::{BVHNode, BVHNode4};
use std::f32::INFINITY;
use std::rc::Rc;
use std::cmp::Ordering::Equal;

enum BVHEnum {
  BVH2( usize, Vec< BVHNode > ),
  // Same representation as 2-way. Skip a check when bounds.x_min=INFINITE
  BVH4( usize, Vec< BVHNode4 > ),
  //BVH4( usize, Vec< BVHNode4 > ),
  BVHNone
}

// A Scene consists of shapes and lights
// The camera is *not* part of the scene
//
// (For specific scenes, look at the `/scenes.rs` file)
pub struct Scene {
  pub background : Color3,
  pub lights     : Vec< Light >,
  pub shapes     : Vec< Rc< dyn Tracable > >,
      bvh        : BVHEnum
}

// A "hit" for a light source
// If such a hit exists, there is a non-occluded ray from a surface point to
//   the light source. (This is used for casting shadow rays)
pub struct LightHit {
  // The vector *to* the light source
  pub dir         : Vec3,
  // The color of the distance-attenuated light source
  pub color       : Vec3,
  // Some(..) if attenuation still needs to be applied, with squared distance
  pub distance_sq : Option< f32 >
}

impl Scene {
  // Constructs a new scene with the specified lights and shapes
  pub fn new( background : Color3
            , lights     : Vec< Light >
            , shapes : Vec< Rc< dyn Tracable > >
            ) -> Scene {
    Scene { background, lights, bvh: BVHEnum::BVHNone, shapes }
  }

  // Rebuilds the BVH, and returns the number of nodes
  pub fn rebuild_bvh( &mut self, num_bins : usize, is_bvh4 : bool ) -> u32 {
    let (num_inf, bvh) = BVHNode::build( &mut self.shapes, num_bins );
    let mut num_nodes = 0;

    if is_bvh4 {
      let bvh4 = BVHNode4::collapse( &bvh );
      num_nodes = BVHNode4::node_count( &bvh4 );
      
      if !BVHNode4::verify( &self.shapes, num_inf, &bvh4) {
        panic!( "WHAT" );
      }
      
      self.bvh = BVHEnum::BVH4( num_inf, bvh4 );
    } else {
      num_nodes = BVHNode::node_count( &bvh );
      self.bvh = BVHEnum::BVH2( num_inf, bvh );
    }


    //self.bvh = BVHEnum::BVH4( num_inf, BVHNode4::from_bvh( &bvh ) );
    // if verify_bvh( &shapes, numinf, &bvh) {
    //   // OK
    // } else {
    //   panic!( "WHAT" );
    // }
    num_nodes as u32
  }

  pub fn disable_bvh( &mut self ) {
    self.bvh = BVHEnum::BVHNone;
  }

  // Casts a shadow ray from the `hit_loc` to all lights in the scene
  // All non-occluded lights are returned by this function
  pub fn shadow_ray( &self, hit_loc : &Vec3, light_id : usize ) -> (usize, Option< LightHit >) {
    match &self.lights[ light_id ] {
      Light::Point( ref l ) => {
        let mut to_light : Vec3 = l.location - *hit_loc;
        let distance_sq = to_light.len_sq( );
        to_light = to_light / distance_sq.sqrt( );

        let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
        let (d,res) = self.trace_simple( &shadow_ray );
        if !is_hit_within_sq( res, distance_sq ) {
          (d, Some( LightHit { dir: to_light, color: l.color, distance_sq: Some( distance_sq ) } ) )
        } else {
          (d, None)
        }
      },
      Light::Directional( ref l ) => {
        let to_light   = -l.direction;
        let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
        let (d, res) = self.trace_simple( &shadow_ray );
        if let Some( _h ) = res {
          // A shadow occludes the lightsource
          (d, None)
        } else {
          // Note that no attenuation applies here, as the lightsource is at an
          // infinite distance anyway
          (d, Some( LightHit { dir: to_light, color: l.color.to_vec3( ), distance_sq: None } ))
        }
      },
      Light::Spot( ref l ) => {
        let mut to_light : Vec3 = l.location - *hit_loc;
        let distance_sq = to_light.len_sq( );
        to_light = to_light / distance_sq.sqrt( );
        let from_light : Vec3 = -to_light;

        let angle_diff = from_light.dot( l.direction ).acos( );

        if angle_diff < l.angle {
          let shadow_ray = Ray::new( *hit_loc + EPSILON * to_light, to_light );
          let (d, res) = self.trace_simple( &shadow_ray );
          if !is_hit_within_sq( res, distance_sq ) {
            ( d, Some( LightHit { dir: to_light, color: l.color, distance_sq: Some( distance_sq ) } ) )
          } else {
            // It's occluded
            ( d, None )
          }
        } else {
          // Outside the spot area
          ( 0, None )
        }
      }
    }
  }

  pub fn trace( &self, ray : &Ray ) -> (usize, Option< Hit >) {
    let (d, t) = self.trace_g( ray );
    if let Some( (_, s) ) = t {
      (d, s.trace( ray ))
    } else {
      (d, None)
    }
  }

  pub fn trace_simple( &self, ray : &Ray ) -> (usize, Option< f32 >) {
    let (d, res) = self.trace_g( ray );
    if let Some( (dis, _) ) = res {
      (d, Some( dis ))
    } else {
      (d, None)
    }
  }

  fn trace_g< 'a >( &'a self, ray : &Ray ) -> (usize, Option< (f32, &'a Rc< dyn Tracable >) >) {
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
          let (d2, h2) = traverse_bvh4_guarded( ray, *numinf, &bvh, &self.shapes, 0, h1.0 );
          (d2, closest( Some( h1 ), h2 ))
        } else {
          traverse_bvh4_guarded( ray, *numinf, &bvh, &self.shapes, 0, INFINITY )
        }
      },
      _ => {
        (0, trace_shapes( ray, &self.shapes ))
      }
    }
  }
}

#[allow(dead_code)]
fn traverse_bvh_guarded< 'a >(
      ray     : &Ray
    , num_inf : usize
    , bvh     : &[BVHNode]
    , shapes  : &'a [Rc< dyn Tracable >]
    , node_i  : usize
    , max_dis : f32 ) -> (usize, Option< (f32, &'a Rc< dyn Tracable >) >) {

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

// Assume the bounding box of `node_i` *does* intersect
fn traverse_bvh< 'a >(
      ray     : &Ray
    , num_inf : usize
    , bvh     : &[BVHNode]
    , shapes  : &'a [Rc< dyn Tracable >]
    , node_i  : usize
    , max_dis : f32 ) -> (usize, Option< (f32, &'a Rc< dyn Tracable >) >) {

  let node = &bvh[ node_i ];
  if node.count != 0 { // leaf
    let offset = node.left_first as usize;
    let size = node.count as usize;

    (1, trace_shapes_md( ray, &shapes[(num_inf+offset)..(num_inf+offset+size)], max_dis ))
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


#[allow(dead_code)]
fn traverse_bvh4_guarded< 'a >(
      ray     : &Ray
    , num_inf : usize
    , bvh     : &[BVHNode4]
    , shapes  : &'a [Rc< dyn Tracable >]
    , node_i  : i32
    , max_dis : f32 ) -> (usize, Option< (f32, &'a Rc< dyn Tracable >) >) {

  traverse_bvh4( ray, num_inf, bvh, shapes, node_i, max_dis )
}

// Assume the bounding box of `node_i` *does* intersect
//   and `node_i` is *not* a skipped node
#[allow(dead_code)]
fn traverse_bvh4< 'a >(
      ray         : &Ray
    , num_inf     : usize
    , bvh         : &[BVHNode4]
    , shapes      : &'a [Rc< dyn Tracable >]
    , node_i      : i32
    , mut max_dis : f32 ) -> (usize, Option< (f32, &'a Rc< dyn Tracable >) >) {
  

  if node_i < 0 { // leaf
    let ni = unsafe { std::mem::transmute::< i32, u32 >( node_i ) };
    let num_shapes = ( ( ni >> 27 ) & 0x3 ) as usize;
    let shape_index = ( ni & 0x7FFFFFF ) as usize;

    // (1, trace_shapes_md( ray, &shapes[(num_inf+offset)..(num_inf+offset+size)], max_dis ))
    (1, trace_shapes_md( ray, &shapes[(num_inf+shape_index)..(num_inf+shape_index+num_shapes)], max_dis ))
  } else { // node
    let node = &bvh[ node_i as usize ];
    let num_children  = node.num_children as usize;

    // ( num_traversed, res )
    let hits = node.child_bounds.hit( ray );
    
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

// fn aabb_distance_inf( ray : &Ray, aabb : &AABB ) -> f32 {
//   if let Some( h ) = aabb.hit( ray ) {
//     h
//   } else {
//     -INFINITY
//   }
// }

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

fn closest< 'a >( a: Option< (f32, &'a Rc< dyn Tracable >) >
                , b: Option< (f32, &'a Rc< dyn Tracable >) >
                ) -> Option< (f32, &'a Rc< dyn Tracable >) > {
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

fn trace_shapes< 'a >( ray     : &Ray
                     , shapes  : &'a [Rc< dyn Tracable >]
                     ) -> Option< (f32, &'a Rc< dyn Tracable >) > {
  let mut best_hit = None;

  for s in shapes {
    if let Some( new_dis ) = s.trace_simple( ray ) {
      if let Some( ( bhd, _ ) ) = best_hit {
        if 0.0_f32 < new_dis && new_dis < bhd {
          best_hit = Some( ( new_dis, s ) );
        }
      } else {
        best_hit = Some( ( new_dis, s ) );
      }
    }
  }

  best_hit
}

fn trace_shapes_md < 'a >( ray     : &Ray
                         , shapes  : &'a [Rc< dyn Tracable >]
                         , max_dis : f32
                         ) -> Option< (f32, &'a Rc< dyn Tracable >) > {
  let mut best_hit = None;

  for s in shapes {
    if let Some( new_dis ) = s.trace_simple( ray ) {
      if new_dis <= max_dis {
        if let Some( ( bhd, _ ) ) = best_hit {
          if 0.0_f32 < new_dis && new_dis < bhd {
            best_hit = Some( ( new_dis, s ) );
          }
        } else {
          best_hit = Some( ( new_dis, s ) );
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
