use crate::rng::Rng;

/// An empiral Probability Distribution Function, with a fixed bin count
pub struct EmpiralPDF {
  // Chances per bin
  bins     : Vec< f32 >,
  // Cumulative chance per bin
  // These are only updated whenever requested
  // This is intended to speed up insertion/quering time, as - in practice -
  //   these PDFS are first constructed and only then updated
  cum_bins : Vec< f32 >,
  has_updated_bins : bool
}

impl EmpiralPDF {
  // Constructs a new empiral PDF
  pub fn new( num_bins : usize ) -> EmpiralPDF {
    EmpiralPDF {
      bins:             vec![ 1.0; num_bins ]
    , cum_bins:         vec![ 1.0 / num_bins as f32; num_bins ]
    , has_updated_bins: false
    }
  }

  // Sets a (relative) scale for one particular bin
  pub fn set( &mut self, bin_id : usize, val : f32 ) {
    self.bins[ bin_id ]   = val;
    self.has_updated_bins = true;
  }

  // Randomly samples a bin, based on its probability
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
    let bin_prob =
      if i + 1 == self.bins.len( ) {
        1.0 - self.bins[ i ]
      } else {
        self.bins[ i + 1 ] - self.bins[ i ]
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
