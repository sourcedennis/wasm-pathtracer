//use crate::data::stack::DefaultStack;
use crate::graphics::{Color3, PointMaterial, Scene, LightEnum};
use crate::graphics::ray::{Ray};
use crate::math::{EPSILON, Vec3};
use crate::math;
use crate::rng::Rng;
use std::f32::INFINITY;
use std::f32::consts::PI;
use std::rc::Rc;
use std::cell::RefCell;
use crate::render_target::RenderTarget;
use crate::data::stack::Stack;
use crate::data::PhotonTree;
use crate::graphics::{SamplingStrategy};

/// The scene camera.
/// It first rotates around the x-axis, then around the y-axis, then it translates
pub struct Camera {
  pub location : Vec3,
  pub rot_x    : f32,
  pub rot_y    : f32
}

impl Camera {
  pub fn new( location : Vec3, rot_x : f32, rot_y : f32 ) -> Camera {
    Camera { location, rot_x, rot_y }
  }
}

#[derive(PartialEq)]
pub enum RenderType {
  NoNEE,
  NormalNEE,
  PNEE
}

pub struct RenderInstance {
  option       : RenderType,
  camera       : Rc< RefCell< Camera > >,
  scene        : Rc< Scene >,
  rng          : Rc< RefCell< Rng > >,
  num_bvh_hits : usize,
  target       : Rc< RefCell< RenderTarget > >,

  sampling_strategy : Box< dyn SamplingStrategy >
}

type ShapeId = usize;

impl RenderInstance {
  pub fn new( scene             : Rc< Scene >
            , camera            : Rc< RefCell< Camera > >
            , rng               : Rc< RefCell< Rng > >
            , sampling_strategy : Box< dyn SamplingStrategy >
            , target            : Rc< RefCell< RenderTarget > >
            , option            : RenderType
            ) -> RenderInstance {
    let mut ins = RenderInstance {
        option, camera, scene, rng, num_bvh_hits: 0, target
      , sampling_strategy
      // , photons:            PhotonTree::new( )
      // , num_photons:        0
      };
    ins.reset( );
    ins
  }

  pub fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.sampling_strategy.resize( x, y, width, height );
    self.reset( );
  }

  /// Resets the rendering for the current scene. Does *not* throw away scene
  /// preprocessing data. This only happens after `update_scene()`
  pub fn reset( &mut self ) {
    // Note: The `target` is reset externally
    self.num_bvh_hits = 0;
    self.sampling_strategy.reset( );
  }

  pub fn update_scene( &mut self, scene : Rc< Scene > ) {
    self.scene = scene;
  }

  pub fn compute( &mut self, num_ticks : usize ) {
    // if self.option == RenderType::PNEE && self.num_photons < self.width * self.height * 8 {
    //   let num_to_compute = ( self.width * self.height * 8 - self.num_photons ); //.min( num_ticks * 2 );
    //   self.preprocess_photons( num_to_compute );
    //   self.num_photons += num_to_compute;

    //   if num_ticks * 2 > num_to_compute {
    //     self.compute_rays( num_ticks - num_to_compute / 2 );
    //   }
    // } else {
    //   self.compute_rays( num_ticks );
    // }
    self.compute_rays( num_ticks );
  }

  // fn preprocess_photons( &mut self, num_ticks : usize ) {
  //   let mut rng = self.rng.borrow_mut( );
  //   let scene = self.scene.borrow( );

  //   for _i in 0..num_ticks {
  //     let light_id = rng.next_in_range( 0, scene.lights.len( ) );
  //     match &scene.lights[ light_id ] {
  //       LightEnum::Point( _ ) => panic!( "Pointlight unsupported" ),
  //       LightEnum::Area( shape_id ) => {
  //         let light_shape = &scene.shapes[ *shape_id ];
  //         let (point_on_light, light_normal, intensity) = light_shape.pick_random( &mut rng );
  //         let ray = Ray::new( point_on_light + light_normal * EPSILON, light_normal );
  //         let (num_bvh_hits, m_hit) = scene.trace( &ray );
  //         self.num_bvh_hits += num_bvh_hits;

  //         if let Some( hit ) = m_hit {
  //           if hit.mat.is_diffuse( ) {
  //             self.photons.insert( ray.at( hit.distance ), intensity.x.max( intensity.y ).max( intensity.y ), light_id );
  //           }
  //         }
  //       }
  //     }
  //   }
  // }

  fn compute_rays( &mut self, num_ticks : usize ) {
    let origin;
    let w_inv;
    let h_inv;
    let ar;

    {
      let camera = self.camera.borrow( );
      let target = self.target.borrow( );

      origin = camera.location;
      let fw     = target.viewport_width as f32;
      let fh     = target.viewport_height as f32;

      w_inv = 1.0 / fw as f32;
      h_inv = 1.0 / fh as f32;
      ar    = fw / fh;
    }
    
    for _i in 0..num_ticks {
      let (x,y) = self.sampling_strategy.next( );

      let (fx, fy) =
        {
          let mut rng = self.rng.borrow_mut( );
          let fx = ( ( x as f32 + rng.next( ) ) * w_inv - 0.5_f32 ) * ar;
          let fy = 0.5_f32 - ( y as f32 + rng.next( ) ) * h_inv;
          (fx, fy)
        };
  
      let pixel = Vec3::new( fx, fy, 0.8 );
      let dir   = 
        {
          let camera = self.camera.borrow( );
          pixel.normalize( ).rot_x( camera.rot_x ).rot_y( camera.rot_y )
        };
      
      let ray = Ray::new( origin, dir );

      // Note that `mat_stack` already contains the "material" for air (so now it's a stack of air)
      let res = self.trace_original_color( &ray );

      let mut target = self.target.borrow_mut( );
      target.write( x, y, res );
    }
  }

  /// Traces an original ray, and produces a gray-scale value for that ray
  /// White values are close, black are far away
  pub fn trace_original_depth( &mut self, ray : &Ray ) -> f32 {
    let (d, res) = self.scene.trace_simple( ray );
    self.num_bvh_hits += d;
    if let Some( v ) = res {
      v
    } else {
      INFINITY
    }
  }

  /// Trace the original ray into the scene (without bounces)
  pub fn trace_original_bvh( &mut self, ray : &Ray ) {
    let (d, _) = self.scene.trace( ray );
    self.num_bvh_hits += d;
  }

  /// Traces an original ray, and produces a color for that ray
  /// Note that the returned value can exceed (1,1,1), but it's *expected value*
  ///   is always between (0,0,0) and (1,1,1)
  pub fn trace_original_color( &mut self, original_ray : &Ray ) -> Vec3 {
    let scene   = &self.scene;
    let mut rng = self.rng.borrow_mut( );
    let has_nee = self.option == RenderType::NormalNEE || self.option == RenderType::PNEE;

    // The acculumator
    let mut color      = Vec3::ZERO;
    let mut throughput = Vec3::new( 1.0, 1.0, 1.0 );

    // Other status structures
    let mut ray = *original_ray;
    let mut has_diffuse_bounced = false;

    loop {
      let (num_bvh_hits, m_hit) = scene.trace( &ray );
      self.num_bvh_hits += num_bvh_hits;
  
      if let Some( hit ) = m_hit {
        let hit_point = ray.at( hit.distance );

        match hit.mat {
          PointMaterial::Emissive { intensity } => {
            if !has_nee || !has_diffuse_bounced {
              color += throughput * intensity;
            } // otherwise NEE is enabled, so ignore it
            return color;
          },
          _ => {
            let wo = -ray.dir;
            // A random next direction, with the probability of picking that direction
            let (wi, pdf) = hit.mat.sample_hemisphere( &mut rng, &wo, &hit.normal );
            //let pdf = hit.mat.pdf( &hit.normal, &wo, &wi );
            //let (wi, pdf) = self.sample_hemisphere( &mut (*self.rng.borrow_mut( )), &wo, &hit.normal );
            // The contribution of the path
            let brdf = hit.mat.brdf( &hit.normal, &wo, &wi ); //brdf( hit.normal, wo, wi );
            let cos_i = wi.dot( hit.normal ); // Geometry term
            throughput = throughput * brdf.to_vec3( ) * cos_i / pdf;
            ray = Ray::new( hit_point + wi * EPSILON, wi );

            has_diffuse_bounced = true;

            if has_nee {
              // Pick a random light source

              let (light_id, light_chance) =
                if self.option == RenderType::PNEE {
                  // let mut cdf = Vec::with_capacity( scene.lights.len( ) );
                  // self.photons.query_cdf( &mut cdf, &hit_point );

                  // if cdf.len( ) == 0 {
                  //   let num_lights = scene.lights.len( );
                  //   ( rng.next_in_range( 0, num_lights ), ( 1.0 / num_lights as f32) )
                  // } else {
                  //   let light_id = lookup_cdf( &cdf, rng.next( ) );
                  //   let light_chance =
                  //     if light_id == 0 {
                  //       cdf[ light_id ].1
                  //     } else {
                  //       cdf[ light_id ].1 - cdf[ light_id - 1 ].1
                  //     };
                  //   (light_id, light_chance )
                  // }
                  panic!( "PNEE not supported" );
                } else {
                  let num_lights = scene.lights.len( );
                  (rng.next_in_range( 0, num_lights ), 1.0 / num_lights as f32)
                };

              match scene.lights[ light_id ] {
                LightEnum::Point { .. } => {
                  panic!( "TODO: Point" );
                },
                LightEnum::Area( light_shape_id ) => {
                  let light_shape = &scene.shapes[ light_shape_id ];

                  let (point_on_light, light_normal, intensity) = light_shape.pick_random( &mut rng );
                  let mut to_light = point_on_light - hit_point;
                  let dis_sq = to_light.len_sq( );
                  to_light = to_light / dis_sq.sqrt( );

                  let cos_i = to_light.dot( hit.normal );
                  let cos_o = (-to_light).dot( light_normal );

                  if cos_i > 0.0 && cos_o > 0.0 {
                    let (num_bvh_hits, is_occluded) = scene.shadow_ray( &hit_point, &point_on_light, Some( light_shape_id ) );
                    self.num_bvh_hits += num_bvh_hits;

                    if !is_occluded {
                      let solid_angle = ( light_shape.surface_area( ) * cos_o ) / dis_sq;

                      color += throughput * intensity * solid_angle * cos_i * ( 1.0 / light_chance );
                    }
                  }
                }
              }
            }
          }
        }

        // Russian roulette
        let keep_chance = throughput.x.max( throughput.y ).max( throughput.z ).min( 0.9 ).max( 0.1 );

        if rng.next( ) < keep_chance {
          throughput = throughput * ( 1.0 / keep_chance );
        } else {
          return color;
        }
      } else {
        color += throughput * scene.background.to_vec3( );
        return color;
      }
    }
  }
}

fn lookup_cdf< 'a, T >( cdf : &'a [(T, f32)], r : f32 ) -> usize {
  // let low = 0;
  // let high = cdf.len( );

  // while low + 1 < high {
  //   let mid = ( low + high ) / 2;
  //   if cdf
  // }

  // cdf[ low ]
  let mut i = 0;
  while i < cdf.len( ) && cdf[ i ].1 < r {
    i += 1;
  }
  if i == cdf.len( ) {
    cdf.len( ) - 1
  } else {
    i
  }
}
