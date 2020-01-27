use crate::math::Vec3;

/// It turns out the Rust `rand` module does not compile to WebAssembly
/// So I implemented my own, which is the XOR shift
pub struct Rng {
  state : u32
}

impl Rng {
  pub fn new( ) -> Rng {
    Rng { state: 0xBABABEBE }
  }

  pub fn with_state( state : u32 ) -> Rng {
    Rng { state }
  }

  /// Uniformly generates a f32 in the range of [0,1]
  pub fn next( &mut self ) -> f32 {
    self.next_u32( ) as f32 * ( 1.0 / 0xFFFFFFFFu32 as f32 )
  }

  /// Returns a random element in the range [low, high)
  /// (So it includes the low value, and excludes the high one)
  pub fn next_in_range( &mut self, low : usize, high : usize ) -> usize {
    if high <= low {
      panic!( "Invalid range" );
    } else if high == low + 1 {
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
    let mut x = self.state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    self.state = x;
    x
  }

  // Returns a random point on the hemisphere, for which `normal` is the normal
  pub fn next_hemisphere( &mut self, normal : &Vec3 ) -> Vec3 {
    let (mut x, mut y, mut z) : (f32,f32,f32);

    while {
      x = self.next( ) * 2.0 - 1.0;
      y = self.next( ) * 2.0 - 1.0;
      z = self.next( ) * 2.0 - 1.0;
      let len_sq = x * x + y * y + z * z;
      len_sq > 1.0
    } { }
    
    let v = Vec3::unit( x, y, z );

    if v.dot( *normal ) < 0.0 {
      -v
    } else {
      v
    }
  }

  pub fn shuffle< T >( &mut self, xs : &mut [T] ) {
    for i in 0..xs.len( ) {
      let new_i = self.next_in_range( 0, xs.len( ) );
      xs.swap( i, new_i );
    }
  }
}
