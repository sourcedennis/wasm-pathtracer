use std::fmt;
use crate::rng::Rng;

/// An empirical Probability Distribution Function, with a fixed bin count
#[derive(Clone)]
pub struct EmpiricalPDF {
  // Chances per bin
  bins     : Vec< f32 >,
  // Cumulative chance per bin
  // These are only updated whenever requested
  // This is intended to speed up insertion/quering time, as - in practice -
  //   these PDFS are first constructed and only then updated
  cum_bins : Vec< f32 >,
  has_updated_bins : bool
}

impl EmpiricalPDF {
  /// Constructs a new empirical PDF
  pub fn new( num_bins : usize ) -> EmpiricalPDF {
    EmpiricalPDF {
        bins:             vec![ 1.0; num_bins ]
      , cum_bins:         vec![ 0.0 as f32; num_bins ]
      , has_updated_bins: true
      }
  }

  /// Sets a (relative) scale for one particular bin
  pub fn set( &mut self, bin_id : usize, val : f32 ) {
    self.bins[ bin_id ]   = val;
    self.has_updated_bins = true;
  }

  /// Add a value to the (relative) scale for one particular bin
  pub fn add( &mut self, bin_id : usize, val : f32 ) {
    self.bins[ bin_id ]   += val;
    self.has_updated_bins = true;
  }

  /// Randomly samples a bin, based on its probability
  pub fn sample( &mut self, rng : &mut Rng ) -> usize {
    self.recheck_cdf( );

    let r = rng.next( );

    // Binary search through the CDF
    let mut low  = 0;
    let mut high = self.bins.len( );

    while low + 1 < high {
      let mid = ( low + high ) / 2;
      if self.cum_bins[ mid ] <= r {
        low = mid;
      } else {
        high = mid;
      }
    }
    low
  }

  /// Returns the chance of hitting bin `i`
  pub fn bin_prob( &mut self, i : usize ) -> f32 {
    self.recheck_cdf( );

    let bin_prob =
      if i + 1 == self.cum_bins.len( ) {
        1.0 - self.cum_bins[ i ]
      } else {
        self.cum_bins[ i + 1 ] - self.cum_bins[ i ]
      };

    bin_prob
  }

  // Makes sure local CDF is up-to-date (which is necessary after a bin has
  // changed)
  fn recheck_cdf( &mut self ) {
    if self.has_updated_bins {
      // As typically modifications happen in a phase before sampling,
      // this is unlikely to be called often
      let mut bin_sum = 0.0;
      for p in &self.bins {
        bin_sum += p;
      }
      self.cum_bins[ 0 ] = 0.0;
      for i in 1..self.bins.len( ) {
        self.cum_bins[ i ] = self.cum_bins[ i - 1 ] + self.bins[ i - 1 ] / bin_sum;
      }
      self.has_updated_bins = false;
    }
  }
}

#[allow(unused_must_use)]
impl fmt::Debug for EmpiricalPDF {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut clone = self.clone( );
    clone.recheck_cdf( );

    write!( f, "EmpiricalPDF {{" );
    if clone.cum_bins.len( ) > 0 {
      write!( f, "{}", clone.cum_bins[ 0 ] );

      for i in 1..clone.cum_bins.len( ) {
        write!( f, ", {}", clone.cum_bins[ i ] );
      }
    }
    write!( f, "}}" )
  }
}
