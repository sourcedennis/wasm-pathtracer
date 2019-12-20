use crate::graphics::Color3;
use crate::math::Vec3;

/// A spot light
/// Spot lights always originate in a single point, and shine in a cone toward
/// a direction
pub struct SpotLight {
  pub location  : Vec3,
  // The direction it is pointing at
  pub direction : Vec3,
  // The angle at which the spot falls off
  pub angle     : f32,
  // The color of the light source (channels may be greater than 1)
  pub color     : Vec3
}

impl SpotLight {
  /// Constructs a new light of the given color at the provided location
  pub fn new( location : Vec3, direction : Vec3, angle : f32, color : Color3, strength : f32 ) -> SpotLight {
    SpotLight { location, direction, angle, color: color.to_vec3( ) * strength }
  }
}
