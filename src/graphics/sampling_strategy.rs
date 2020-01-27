// Stdlib imports
use std::rc::Rc;
use std::cell::RefCell;
use std::f32::INFINITY;
// Local imports
use crate::math::Vec3;
use crate::data::stack::Stack;
use crate::rng::Rng;
use crate::render_target::{RenderTarget, SimpleRenderTarget};

// Sampling strategies for pixels

pub trait SamplingStrategy {
  /// Returns a new sample in *viewport* space
  ///   (It selects a point in it's assigned viewport-region, and transforms it
  ///    to viewport space)
  fn next( &mut self ) -> (usize, usize);

  /// Assigns a new viewport-region to the sampler
  fn resize( &mut self, x : usize, y : usize, width : usize, height : usize );

  /// Resets the sampling strategy
  fn reset( &mut self );
}

// ### Random Sampling Strategy ###

/// In the random sampling strategy, every pixel has equal probability of being
/// selected as the next pixel
pub struct RandomSamplingStrategy {
  x      : usize,
  y      : usize,
  width  : usize,
  height : usize,
  rng    : Rc< RefCell< Rng > >
}

impl RandomSamplingStrategy {
  /// Constructs a new random sampling strategy for the given region within the
  /// viewport
  #[allow(unused)]
  pub fn new( x : usize, y : usize, width : usize, height : usize, rng : Rc< RefCell< Rng > >, sampling_target : Rc< RefCell< SimpleRenderTarget > > ) -> RandomSamplingStrategy {
    let mut t = sampling_target.borrow_mut( );
    let c = Vec3::new( 0.0, 0.0, 1.0 );
    for vy in 0..height {
      for vx in 0..width {
        t.write( x + vx, y + vy, c );
      }
    }
    RandomSamplingStrategy { x, y, width, height, rng }
  }
}

impl SamplingStrategy for RandomSamplingStrategy {
  /// See `SamplingStrategy#next()`
  fn next( &mut self ) -> (usize, usize) {
    let mut rng = self.rng.borrow_mut( );
    ( self.x + rng.next_in_range( 0, self.width ), self.y + rng.next_in_range( 0, self.height ) )
  }

  /// See `SamplingStrategy#resize()`
  fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.x      = x;
    self.y      = y;
    self.width  = width;
    self.height = height;
  }

  /// See `SamplingStrategy#reset()`
  fn reset( &mut self ) { }
}

// ### Adaptive Sampling Strategy ###

/// The adaptive sampling strategy will assign more samples to pixels that need
/// it most. Typically, this is expected to reduce fireflies and other anomalies
pub struct AdaptiveSamplingStrategy {
  x      : usize,
  y      : usize,
  width  : usize,
  height : usize,
  target : Rc< RefCell< RenderTarget > >,
  rng    : Rc< RefCell< Rng > >,

  num_sampled  : usize,
  next_samples : Stack< ( usize, usize ) >,

  // A visualisation of the sampling strategy
  sampling_target : Rc< RefCell< SimpleRenderTarget > >
}

impl AdaptiveSamplingStrategy {
  #[allow(unused)]
  pub fn new(
        x      : usize
      , y      : usize
      , width  : usize
      , height : usize
      , target : Rc< RefCell< RenderTarget > >
      , rng    : Rc< RefCell< Rng > >
      , sampling_target : Rc< RefCell< SimpleRenderTarget > >
      ) -> AdaptiveSamplingStrategy {
    let mut strat =
      AdaptiveSamplingStrategy {
        x
      , y
      , width
      , height
      , target
      , rng
      , num_sampled:  0
      , next_samples: Stack::new( ( 0, 0 ) )
      , sampling_target
      };
    strat.reset( );
    strat
  }
}

impl SamplingStrategy for AdaptiveSamplingStrategy {
  /// See `SamplingStrategy#next()`
  fn next( &mut self ) -> (usize, usize) {
    if let Some( v ) = self.next_samples.pop( ) {
      self.num_sampled += 1;
      v
    } else {
      // The adaptive sampling occurs here
      // Perform the adaptive sampling. Check which pixels need it the most
      let target = self.target.borrow( );
      let mut sampling_target = self.sampling_target.borrow_mut( );

      // Estimate the error of the pixels
      let mut mse = vec![ 0.0; self.width * self.height ];
      let mut mse_sum = 0.0;
      let mut mse_min = INFINITY;
      let mut mse_max = -INFINITY;

      for y in 0..self.height {
        for x in 0..self.width {
          let v0 = target.read_clamped( self.x + x, self.y + y );
          let v1 = target.gaussian3( self.x + x, self.y + y );
          let v2 = target.gaussian5( self.x + x, self.y + y );

          mse[ y * self.width + x ] = v0.dis_sq( v1 ).max( v0.dis_sq( v2 ) );
          mse_sum += mse[ y * self.width + x ];
          mse_min = mse_min.min( mse[ y * self.width + x ] );
          mse_max = mse_max.max( mse[ y * self.width + x ] );
        }
      }

      // Queue the pixels based on their error, and fill the sampling visual buffer
      let mse_avg = mse_sum / ( self.width * self.height ) as f32;

      for y in 0..self.height {
        for x in 0..self.width {
          let mut scaled_mse = // scale to [0,1]
            if mse[ y * self.width + x ] < mse_avg {
              0.5 * ( ( mse[ y * self.width + x ] - mse_min ) / ( mse_avg - mse_min ) )
            } else {
              0.5 + 0.5 * ( ( mse[ y * self.width + x ] - mse_avg ) / ( mse_max - mse_avg ) )
            };
          scaled_mse = scaled_mse.min( 1.0 ).max( 0.0 );
          let spp = ( 1.0 + scaled_mse * 32.0 ).ceil( ) as usize;
          for _i in 0..spp {
            self.next_samples.push( ( self.x + x, self.y + y ) );
          }

          if mse_min == mse_max {
            sampling_target.write( self.x + x, self.y + y, Vec3::ZERO );
          } else {
            sampling_target.write( self.x + x, self.y + y, mix_color( scaled_mse ) );
          }
        }
      }

      if let Some( v ) = self.next_samples.pop( ) {
        v
      } else {
        panic!( "Sampling error" );
      }
    }
  }
  
  /// See `SamplingStrategy#resize()`
  fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.x      = x;
    self.y      = y;
    self.width  = width;
    self.height = height;
    self.reset( );
  }

  /// See `SamplingStrategy#reset()`
  fn reset( &mut self ) {
    self.next_samples.clear( );

    // The first w*h*4 samples are not adaptive, because there is nothing to
    // adapt to yet
    for vy in 0..self.height {
      for vx in 0..self.width {
        for _i in 0..4 {
          self.next_samples.push( ( self.x + vx, self.y + vy ) );
        }
      }
    }

    {
      // First, it's not adaptive, so show 1 sample per pixel
      let mut t = self.sampling_target.borrow_mut( );
      let c = Vec3::new( 0.0, 0.0, 1.0 );
      for vy in 0..self.height {
        for vx in 0..self.width {
          t.write( self.x + vx, self.y + vy, c );
        }
      }
    }

    self.next_samples.shuffle( &mut self.rng.borrow_mut( ) );
  }
}

/// Transforms a value in the range [0,1] to a sampling density color
/// The average (0.5) is blue. Below average is green. Above average is red
fn mix_color( v : f32 ) -> Vec3 {
  if v < 0.5 { // Green to blue
    Vec3::new( 0.0, 1.0, 0.0 ) * ( 1.0 - 2.0 * v ) + Vec3::new( 0.0, 0.0, 1.0 ) * 2.0 * v
  } else {
    Vec3::new( 0.0, 0.0, 1.0 ) * ( 1.0 - 2.0 * ( v - 0.5 ) ) + Vec3::new( 1.0, 0.0, 0.0 ) * 2.0 * ( v - 0.5 )
  }
}
