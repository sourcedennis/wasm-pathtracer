use crate::graphics::Color3;
use crate::math::Vec3;

/// A point light
pub struct PointLight {
  pub location : Vec3,
  pub color    : Color3
}

impl PointLight {
  /// Constructs a new light of the given color at the provided location
  pub fn new( location : Vec3, color : Color3 ) -> PointLight {
    PointLight { location, color }
  }
}
