
// Some random math utilities

pub static EPSILON : f32 = 0.0002;

pub fn clamp( x : f32, min_val : f32, max_val : f32 ) -> f32 {
  max_val.min( min_val.max( x ) )
}
