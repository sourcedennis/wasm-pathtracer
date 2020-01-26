mod vec2;
mod vec3;
mod empiral_pdf;

pub use vec2::Vec2;
pub use vec3::Vec3;
pub use empiral_pdf::EmpiralPDF;

// Some arbitrary math utilities

pub static EPSILON : f32 = 0.0002;

pub fn clamp( x : f32, min_val : f32, max_val : f32 ) -> f32 {
  max_val.min( min_val.max( x ) )
}
