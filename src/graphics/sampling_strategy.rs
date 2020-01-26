// Stdlib imports
use std::rc::Rc;
use std::cell::RefCell;
// Local imports
use crate::data::stack::Stack;
use crate::rng::Rng;
use crate::render_target::RenderTarget;

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
  pub fn new( x : usize, y : usize, width : usize, height : usize, rng : Rc< RefCell< Rng > > ) -> RandomSamplingStrategy {
    RandomSamplingStrategy { x, y, width, height, rng }
  }
}

impl SamplingStrategy for RandomSamplingStrategy {
  fn next( &mut self ) -> (usize, usize) {
    let mut rng = self.rng.borrow_mut( );
    ( self.x + rng.next_in_range( 0, self.width ), self.y + rng.next_in_range( 0, self.height ) )
  }

  /// Assigns a new viewport-region to the sampler
  fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.x      = x;
    self.y      = y;
    self.width  = width;
    self.height = height;
  }

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
  next_samples : Stack< ( usize, usize ) >
}

impl AdaptiveSamplingStrategy {
  pub fn new(
        x      : usize
      , y      : usize
      , width  : usize
      , height : usize
      , target : Rc< RefCell< RenderTarget > >
      , rng    : Rc< RefCell< Rng > >
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
      };
    strat.reset( );
    strat
  }
}

impl SamplingStrategy for AdaptiveSamplingStrategy {
  fn next( &mut self ) -> (usize, usize) {
    if let Some( v ) = self.next_samples.pop( ) {
      self.num_sampled += 1;
      v
    } else {
      // The adaptive sampling occurs here
      // Perform the adaptive sampling. Check which pixels need it the most
      let target = self.target.borrow_mut( );

      // Estimate the error of the pixels
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

      // Assign between 1 and 32 samples to each pixel.
      let num_samples = self.width * self.height * 7; // Scale by 7, then add one

      for y in 0..self.height {
        for x in 0..self.width {
          let spp = ( 1 + ( num_samples as f32 * mse[ y * self.width + x ] / mse_sum ).ceil( ) as usize ).max( 1 ).min( 32 );
          for _i in 0..spp {
            self.next_samples.push( ( self.x + x, self.y + y ) );
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
  
  fn resize( &mut self, x : usize, y : usize, width : usize, height : usize ) {
    self.x      = x;
    self.y      = y;
    self.width  = width;
    self.height = height;
    self.reset( );
  }

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

    self.next_samples.shuffle( &mut self.rng.borrow_mut( ) );
  }
}
