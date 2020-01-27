// Stdlib imports
use crate::math::Vec3;

/// A pixel buffer
pub struct RenderTarget {
  pub viewport_width  : usize,
  pub viewport_height : usize,
  acc_buffer          : Vec< Vec3 >,
  acc_count           : Vec< usize >,
  result              : Vec< u8 >
}

/// A 3x3 Gaussian filter (should be divided by 16)
static GAUSS3: [f32; 9] =
  [ 1.0, 2.0, 1.0
  , 2.0, 4.0, 2.0
  , 1.0, 2.0, 1.0
  ];
  
/// A 5x5 Gaussian filter (should be divided by 256)
static GAUSS5: [f32; 25] =
  [ 1.0,  4.0,  6.0,  4.0, 1.0
  , 4.0, 16.0, 24.0, 16.0, 4.0
  , 6.0, 24.0, 36.0, 24.0, 6.0
  , 4.0, 16.0, 24.0, 16.0, 4.0
  , 1.0,  4.0,  6.0,  4.0, 1.0
  ];

impl RenderTarget {
  /// Constructs a new render target with the given viewport size
  pub fn new( viewport_width : usize, viewport_height : usize ) -> RenderTarget {
    let acc_buffer = vec![ Vec3::ZERO; viewport_width * viewport_height ];
    let acc_count  = vec![ 0; viewport_width * viewport_height ];
    let mut result = vec![ 0; viewport_width * viewport_height * 4 ];

    for i in 0..(viewport_width * viewport_height) {
      result[ i * 4 + 3 ] = 255;
    }

    RenderTarget { viewport_width, viewport_height, acc_buffer, acc_count, result }
  }

  /// Clears the render target
  pub fn clear( &mut self ) {
    for i in 0..(self.viewport_width * self.viewport_height) {
      self.acc_buffer[ i ] = Vec3::ZERO;
      self.acc_count[ i ]  = 0;
      self.result[ i * 4 + 0 ] = 0;
      self.result[ i * 4 + 1 ] = 0;
      self.result[ i * 4 + 2 ] = 0;
    }
  }

  /// Writes the given value *for a single sample* to the target
  pub fn write( &mut self, x : usize, y : usize, v : Vec3 ) {
    let i = self.viewport_width * y + x;
    self.acc_buffer[ i ] += v;
    self.acc_count[ i ]  += 1;

    let v     = self.acc_buffer[ i ];
    let count = self.acc_count[ i ];
    self.result[ i * 4 + 0 ] = ( ( v.x / count as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
    self.result[ i * 4 + 1 ] = ( ( v.y / count as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
    self.result[ i * 4 + 2 ] = ( ( v.z / count as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
  }

  /// Reads the averaged value (over all samples) for the given pixel
  pub fn read( &self, x : usize, y : usize ) -> Vec3 {
    let i = self.viewport_width * y + x;
    self.acc_buffer[ i ] / self.acc_count[ i ] as f32
  }

  /// Reads the averaged value (over all samples) for the given pixel
  pub fn read_clamped( &self, x : usize, y : usize ) -> Vec3 {
    let i = self.viewport_width * y + x;
    clamp( self.acc_buffer[ i ] / self.acc_count[ i ] as f32 )
  }

  /// Returns a reference to the averaged pixel buffer
  pub fn results< 'a >( &'a self ) -> &'a Vec< u8 > {
    &self.result
  }

  // Applies the 3x3 Guassian kernel to the pixel at (x,y)
  // [1 2 1]
  // [2 4 2]
  // [1 2 1]
  pub fn gaussian3( &self, x : usize, y : usize ) -> Vec3 {
    let ix = x as i32;
    let iy = y as i32;

    let mut sum = 0.0;
    let mut acc = Vec3::ZERO;

    for vy in 0..3usize {
      for vx in 0..3usize {
        let (m, res) = self.read_mul( ix + vx as i32 - 1, iy + vy as i32 - 1, GAUSS3[ vy * 3 + vx ] );
        acc += res;
        sum += m;
      }
    }

    acc / sum
  }

  // Applies the 3x3 Guassian kernel to the pixel at (x,y)
  // [1  4  6  4 1]
  // [4 16 24 16 4]
  // [6 24 36 24 6]
  // [4 16 24 16 4]
  // [1  4  6  4 1]
  pub fn gaussian5( &self, x : usize, y : usize ) -> Vec3 {
    let ix = x as i32;
    let iy = y as i32;

    let mut sum = 0.0;
    let mut acc = Vec3::ZERO;

    for vy in 0..5usize {
      for vx in 0..5usize {
        let (m, res) = self.read_mul( ix + vx as i32 - 2, iy + vy as i32 - 2, GAUSS5[ vy * 5 + vx ] );
        acc += res;
        sum += m;
      }
    }

    acc / sum
  }

  // A helper function. Tries to multiply the value at pixel (x,y) with the
  // given multiplier.
  fn read_mul( &self, x : i32, y : i32, mul : f32 ) -> (f32, Vec3) {
    if x < 0 || y < 0 || x >= self.viewport_width as i32 || y >= self.viewport_height as i32 {
      ( 0.0, Vec3::ZERO )
    } else {
      ( mul, mul * self.read_clamped( x as usize, y as usize ) )
    }
  }
}

/// A pixel buffer
pub struct SimpleRenderTarget {
  pub viewport_width  : usize,
  pub viewport_height : usize,
  result              : Vec< u8 >
}

impl SimpleRenderTarget {
  /// Constructs a new render target with the given viewport size
  pub fn new( viewport_width : usize, viewport_height : usize ) -> SimpleRenderTarget {
    let mut result = vec![ 0; viewport_width * viewport_height * 4 ];

    for i in 0..(viewport_width * viewport_height) {
      result[ i * 4 + 3 ] = 255;
    }

    SimpleRenderTarget { viewport_width, viewport_height, result }
  }

  /// Clears the render target
  pub fn clear( &mut self ) {
    for i in 0..(self.viewport_width * self.viewport_height) {
      self.result[ i * 4 + 0 ] = 0;
      self.result[ i * 4 + 1 ] = 0;
      self.result[ i * 4 + 2 ] = 0;
    }
  }

  /// Writes the given value *for a single sample* to the target
  pub fn write( &mut self, x : usize, y : usize, v : Vec3 ) {
    let i = self.viewport_width * y + x;

    self.result[ i * 4 + 0 ] = ( ( v.x as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
    self.result[ i * 4 + 1 ] = ( ( v.y as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
    self.result[ i * 4 + 2 ] = ( ( v.z as f32 ).min( 1.0 ).max( 0.0 ) * 255.0 ) as u8;
  }

  /// Returns a reference to the averaged pixel buffer
  pub fn results< 'a >( &'a self ) -> &'a Vec< u8 > {
    &self.result
  }
}

fn clamp( v : Vec3 ) -> Vec3 {
  Vec3::new( v.x.max( 0.0 ).min( 1.0 ), v.y.max( 0.0 ).min( 1.0 ), v.z.max( 0.0 ).min( 1.0 ) )
}
