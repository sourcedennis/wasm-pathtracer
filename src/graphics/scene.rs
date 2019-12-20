use crate::graphics::{Color3};
use crate::graphics::ray::{Ray, Hit, Tracable};
use crate::graphics::lights::Light;
use crate::math::{Vec3, EPSILON};
use crate::graphics::{BVHNode};
use std::f32::INFINITY;
use std::rc::Rc;

// enum BVHEnum {
//   // It is a 4-way BVH when bounds.x=INFINITE
//   // This skips the parent check
//   BVH2( usize, Vec< BVHNode > ),
//   //BVH4( usize, Vec< BVHNode4 > ),
//   BVHNone
// }

// A Scene consists of shapes and lights
// The camera is *not* part of the scene
//
// (For specific scenes, look at the `/scenes.rs` file)
pub struct Scene {
  pub background : Color3,
  pub lights     : Vec< Light >,
  pub shapes     : Vec< Rc< dyn Tracable > >,
      bvh        : Option< ( usize, Vec< BVHNode > ) >
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
    Scene { background, lights, bvh: None, shapes }
  }

  pub fn rebuild_bvh( &mut self, num_bins : usize ) {
    let (num_inf, bvh) = BVHNode::build( &mut self.shapes, num_bins );
    self.bvh = Some( ( num_inf, bvh ) );
    //self.bvh = BVHEnum::BVH4( num_inf, BVHNode4::from_bvh( &bvh ) );
    // if verify_bvh( &shapes, numinf, &bvh) {
    //   // OK
    // } else {
    //   panic!( "WHAT" );
    // }
  }

  pub fn disable_bvh( &mut self ) {
    self.bvh = None;
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
    if let Some( ( numinf, bvh ) ) = &self.bvh {
      if let Some( h1 ) = trace_shapes( ray, &self.shapes[..*numinf] ) {
        let (d2, h2) = traverse_bvh_guarded( ray, *numinf, &bvh, &self.shapes, 0, h1.0 );
        (d2, closest( Some( h1 ), h2 ))
      } else {
        traverse_bvh_guarded( ray, *numinf, &bvh, &self.shapes, 0, INFINITY )
      }
    } else {
      (0, trace_shapes( ray, &self.shapes ))
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
      traverse_bvh( ray, num_inf, bvh, shapes, node_i, max_dis )
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

    (1, trace_shapes( ray, &shapes[(num_inf+offset)..(num_inf+offset+size)] ))
  } else { // node
    let left_index = node.left_first as usize;

    if let Some( left_dis ) = aabb_distance( ray, &bvh[ left_index ], max_dis ) {
      if let Some( right_dis ) = aabb_distance( ray, &bvh[ left_index + 1 ], max_dis ) {
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

fn aabb_distance( ray : &Ray, bvh : &BVHNode, max_dis : f32 ) -> Option< f32 > {
  /*match bvh {
    BVHNode::Leaf { bounds, .. } => bounds.hit( ray ),
    BVHNode::Node { bounds, .. } => bounds.hit( ray )
  }*/
  if let Some( h ) = bvh.bounds.hit( ray ) {
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

fn trace_shapes< 'a >( ray : &Ray
                     , shapes : &'a [Rc< dyn Tracable >]
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

// Returns only true if a hit occurs and it occurs within at most `sqrt(d_sq)` units
// `d_sq` is the square of the distance - for efficiency reasons
fn is_hit_within_sq( m_hit : Option< f32 >, d_sq : f32 ) -> bool {
  if let Some( h ) = m_hit {
    h * h < d_sq
  } else {
    false
  }
}
