use crate::graphics::Color3;
use crate::math::Vec3;

/// A point light
pub struct DirectionalLight {
  // Direction from the light source to the scene
  // For directional lights this is the same for every point in the scene
  pub direction : Vec3,
  // The color of the light source
  pub color     : Color3
}

impl DirectionalLight {
  pub fn new( direction : Vec3, color : Color3 ) -> DirectionalLight {
    DirectionalLight { direction, color }
  }
}
