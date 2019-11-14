
pub fn clamp( x : f64, min_val : f64, max_val : f64 ) -> f64 {
  max_val.min( min_val.max( x ) )
}