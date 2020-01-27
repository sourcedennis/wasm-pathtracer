//use crate::data::stack::DefaultStack;
use crate::graphics::{Color3, PointMaterial, Scene, LightEnum};
use crate::graphics::ray::{Ray};
use crate::graphics::{AABB};
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

  sampling_strategy : Box< dyn SamplingStrategy >,

  // If true, renders the selected photons in "debug-mode"
  // Which means at each sample, it renders the verbatim color of the selected
  // light source.
  is_debug_photons  : bool,

  photons     : PhotonTree,
  num_photons : usize
}

type ShapeId = usize;

impl RenderInstance {
  pub fn new( scene             : Rc< Scene >
            , camera            : Rc< RefCell< Camera > >
            , rng               : Rc< RefCell< Rng > >
            , sampling_strategy : Box< dyn SamplingStrategy >
            , is_debug_photons  : bool
            , target            : Rc< RefCell< RenderTarget > >
            , option            : RenderType
            ) -> RenderInstance {
    let num_lights = scene.lights.len( );
    let mut ins = RenderInstance {
        option, camera, scene, rng, num_bvh_hits: 0, target
      , sampling_strategy
      , is_debug_photons
      , photons:            PhotonTree::new( num_lights )
      , num_photons:        0
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
    self.num_photons = 0;
    self.photons     = PhotonTree::new( scene.lights.len( ) );
    self.scene       = scene;
    self.reset( );
  }

  pub fn compute( &mut self, num_ticks : usize ) {
    let total_photons_needed = 300000;

    if self.option == RenderType::PNEE && self.num_photons < total_photons_needed {
      let num_to_compute = ( total_photons_needed - self.num_photons ).min( num_ticks * 32 );
      // Note that calling this may not actually hit `num_to_compute` photons
      // it only shoots them, but they're only counted when hit
      self.preprocess_photons( num_to_compute );

      let mut ticks_left = num_ticks - num_to_compute / 32;
      while ticks_left > 0 && self.num_photons < total_photons_needed {
        let num_to_compute = ( total_photons_needed - self.num_photons ).min( ticks_left * 32 );
        self.preprocess_photons( num_to_compute );
        ticks_left -= num_to_compute / 32;
      }

      self.compute_rays( ticks_left );
    } else {
      self.compute_rays( num_ticks );
    }
  }

  fn preprocess_photons( &mut self, num_ticks : usize ) {
    let mut rng = self.rng.borrow_mut( );
    let scene   = &self.scene;

    //if let Some( b ) = self.scene.scene_bounds( ) {
      for _i in 0..num_ticks {
        let light_id = rng.next_in_range( 0, scene.lights.len( ) );
        match &scene.lights[ light_id ] {
          LightEnum::Point( _ ) => panic!( "Pointlight unsupported" ),
          LightEnum::Area( shape_id ) => {
            let light_shape = &scene.shapes[ *shape_id ];
            let (point_on_light, ln, intensity) = light_shape.pick_random( &mut rng );
            let light_normal = rng.next_hemisphere( &ln );
            let ray = Ray::new( point_on_light + light_normal * EPSILON, light_normal );
            let (num_bvh_hits, m_hit) = scene.trace( &ray );
            self.num_bvh_hits += num_bvh_hits;
  
            if let Some( hit ) = m_hit {
              let photon_hitpoint = ray.at( hit.distance ) + hit.normal * EPSILON;
              if hit.mat.is_diffuse( ) {
                self.photons.insert( light_id, photon_hitpoint, ln.dot( light_normal ) * intensity.x.max( intensity.y ).max( intensity.z ) );
                self.num_photons += 1;
              }
            }
          }
        }
      }
    //}
  }

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
            if self.is_debug_photons {
              if !has_diffuse_bounced {
                color += throughput * intensity;
              }
            } else if !has_nee || !has_diffuse_bounced {
              color += throughput * intensity;
            } // otherwise NEE is enabled, so ignore it
            return color;
          },
          _ => {
            let wo = -ray.dir;
            // A random next direction, with the probability of picking that direction
            let (wi, pdf) = hit.mat.sample_hemisphere( &mut rng, &wo, &hit.normal );
            // The contribution of the path
            let brdf = hit.mat.brdf( &hit.normal, &wo, &wi );
            let cos_i = wi.dot( hit.normal ); // Geometry term
            throughput = throughput * brdf.to_vec3( ) * cos_i / pdf;
            ray = Ray::new( hit_point + wi * EPSILON, wi );

            has_diffuse_bounced = true;

            if has_nee {
              // Pick a random light source

              let (light_id, light_chance) =
                if self.option == RenderType::PNEE {
                  self.photons.sample( &mut rng, hit_point )
                  // let num_lights = scene.lights.len( );
                  // (rng.next_in_range( 0, num_lights ), 1.0 / num_lights as f32)
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
                    if self.is_debug_photons {
                      // Physically *inaccurate* light-selection debug render
                      color += throughput * intensity;
                    } else {
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
