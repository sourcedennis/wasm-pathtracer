use crate::data::stack::DefaultStack;
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
  Adaptive
}

pub struct RenderInstance {
  option       : RenderType,
  camera       : Rc< RefCell< Camera > >,
  scene        : Rc< RefCell< Scene > >,
  rng          : Rc< RefCell< Rng > >,
  num_bvh_hits : usize,
  x            : usize,
  y            : usize,
  width        : usize,
  height       : usize,
  target       : Rc< RefCell< RenderTarget > >,

  // Used by importance samples
  total_samples_done : usize,
  next_pixels        : Vec< ( usize, usize ) >
}

type ShapeId = usize;

impl RenderInstance {
  pub fn new( scene : Rc< RefCell< Scene > >, camera : Rc< RefCell< Camera > >, rng : Rc< RefCell< Rng > >
            , x : usize, y : usize, width : usize, height : usize, target : Rc< RefCell< RenderTarget > >
            , option : RenderType ) -> RenderInstance {
    let mut ins = RenderInstance {
        option, scene, camera, rng, num_bvh_hits: 0, x, y, width, height, target
      , total_samples_done: 0
      , next_pixels:        Vec::with_capacity( 1024 * 64 )
      };
    ins.reset( );
    ins
  }

  pub fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.x      = x;
    self.y      = y;
    self.width  = width;
    self.height = height;
    self.reset( );
  }

  pub fn reset( &mut self ) {
    // Note: The `target` is reset externally
    self.num_bvh_hits       = 0;
    self.total_samples_done = 0;
    self.next_pixels.clear( );

    if self.option == RenderType::Adaptive {
      for y in 0..self.height {
        for x in 0..self.width {
          // Before adaptive filtering an "initialization phase" happens, which
          // involves rendering 4 samples-per-pixel.
          for _i in 0..4 {
            self.next_pixels.push( (self.x + x, self.y + y) );
          }
        }
      }
      self.rng.borrow_mut( ).shuffle( &mut self.next_pixels );
    }
  }

  fn next_pixel( &mut self ) -> ( usize, usize ) {
    if self.option == RenderType::Adaptive {
      if let Some( ( x, y ) ) = self.next_pixels.pop( ) {
        ( x, y )
      } else {
        // Perform the adaptive sampling. Check which pixels need it the most
        let target = self.target.borrow_mut( );

        // Estimate the mean-square-error of pixels
        let mut mse = vec![ 0.0; self.width * self.height ];
        let mut mse_sum = 0.0;

        for y in 0..self.height {
          for x in 0..self.width {
            let v0 = target.read( self.x + x, self.y + y );
            let v1 = target.gaussian3( self.x + x, self.y + y );
            let v2 = target.gaussian5( self.x + x, self.y + y );

            mse[ y * self.width + x ] = ( v0 - v1 ).len_sq( ).max( ( v0 - v2 ).len_sq( ) );
            mse_sum += mse[ y * self.width + x ];
          }
        }

        let num_samples = self.width * self.height * 7;

        for y in 0..self.height {
          for x in 0..self.width {
            let spp = 1 + ( num_samples as f32 * mse[ y * self.width + x ] / mse_sum ).ceil( ) as usize;
            for _i in 0..spp {
              self.next_pixels.push( ( self.x + x, self.y + y ) );
            }
          }
        }

        //self.rng.borrow_mut( ).shuffle( &mut self.next_pixels );

        ( self.x, self.y )
      }
    } else {
      let mut rng = self.rng.borrow_mut( );
      let x       = self.x + rng.next_in_range( 0, self.width as usize );
      let y       = self.y + rng.next_in_range( 0, self.height as usize );
      ( x, y )
    }
  }

  pub fn compute( &mut self, num_ticks : usize ) {
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
      let (x,y) = self.next_pixel( );

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

      self.total_samples_done += 1;
    }
  }

  /// Traces an original ray, and produces a gray-scale value for that ray
  /// White values are close, black are far away
  pub fn trace_original_depth( &mut self, ray : &Ray ) -> f32 {
    let scene = self.scene.borrow( );
    let (d, res) = scene.trace_simple( ray );
    self.num_bvh_hits += d;
    if let Some( v ) = res {
      v
    } else {
      INFINITY
    }
  }

  /// Trace the original ray into the scene (without bounces)
  pub fn trace_original_bvh( &mut self, ray : &Ray ) {
    let scene = self.scene.borrow( );
    let (d, _) = scene.trace( ray );
    self.num_bvh_hits += d;
  }

  /// Traces an original ray, and produces a color for that ray
  /// Note that the returned value can exceed (1,1,1), but it's *expected value*
  ///   is always between (0,0,0) and (1,1,1)
  pub fn trace_original_color( &mut self, original_ray : &Ray ) -> Vec3 {
    let scene   = self.scene.borrow( );
    let mut rng = self.rng.borrow_mut( );
    let has_nee = self.option == RenderType::NormalNEE || self.option == RenderType::Adaptive;

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
            let wi = hit.mat.sample_hemisphere( &mut rng, &wo, &hit.normal );
            let pdf = hit.mat.pdf( &hit.normal, &wo, &wi );
            //let (wi, pdf) = self.sample_hemisphere( &mut (*self.rng.borrow_mut( )), &wo, &hit.normal );
            // The contribution of the path
            let brdf = hit.mat.brdf( &hit.normal, &wo, &wi ); //brdf( hit.normal, wo, wi );
            let cos_i = wi.dot( hit.normal ); // Geometry term
            throughput = throughput * brdf.to_vec3( ) * cos_i / pdf;
            ray = Ray::new( hit_point + wi * EPSILON, wi );

            has_diffuse_bounced = true;

            if has_nee {
              // // Pick a random light source
              let num_lights = scene.lights.len( );
              let light_id = rng.next_in_range( 0, num_lights );

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

                      color += throughput * intensity * solid_angle * cos_i * ( num_lights as f32 );
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
