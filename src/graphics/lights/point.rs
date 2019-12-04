use crate::graphics::Color3;
use crate::math::Vec3;

/// A point light
pub struct PointLight {
  pub location : Vec3,
  // The color of the light source (channels may be greater than 1)
  pub color    : Vec3
}

impl PointLight {
  /// Constructs a new light of the given color at the provided location
  pub fn new( location : Vec3, color : Color3, strength : f32 ) -> PointLight {
    PointLight { location, color: color.to_vec3( ) * strength }
  }
}
