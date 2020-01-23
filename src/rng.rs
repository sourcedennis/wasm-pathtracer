
/// It turns out the Rust `rand` module does not compile to WebAssembly
/// So I implemented my own, which is the XOR shift
pub struct Rng {
  state : u32
}

impl Rng {
  pub fn new( ) -> Rng {
    Rng { state: 0xBABABEBE }
  }

  /// Uniformly generates a f32 in the range of [0,1]
  pub fn next( &mut self ) -> f32 {
    self.next_u32( ) as f32 * ( 1.0 / 0xFFFFFFFF as f32 )
  }

  /// Returns a random element in the range [low, high)
  /// (So it includes the low value, and excludes the high one)
  pub fn nextInRange( &mut self, low : usize, high : usize ) -> usize {
    if high <= low {
      panic!( "Invalid range" );
    } else if high <= low + 1 {
      0
    } else {
      let f = self.next( );
      if f == 1.0 {
        high - 1
      } else {
        ( f as f32 * ( high - low ) as f32 ).floor( ) as usize + low
      }
    }
  }

  fn next_u32( &mut self ) -> u32 {
    let x = self.state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.state = x;
    x
  }
}
