mod directional;
mod point;
mod spot;

pub use point::{PointLight};
pub use directional::{DirectionalLight};
pub use spot::{SpotLight};

use crate::math::Vec3;
use crate::graphics::Color3;

/// A general light class which encapsulates the other available light sources.
pub enum Light {
  Directional( DirectionalLight ),
  Point( PointLight ),
  Spot( SpotLight )
}

impl Light {
  /// Constructs a new directional light. See `DirectionalLight::new(..)`.
  pub fn directional( direction : Vec3, color : Color3 ) -> Light {
    Light::Directional( DirectionalLight::new( direction, color ) )
  }

  /// Constructs a new point light. See `PointLight::new(..)`.
  pub fn point( location : Vec3, color : Color3, strength : f32 ) -> Light {
    Light::Point( PointLight::new( location, color, strength ) )
  }

  /// Constructs a new spot light. See `SpotLight::new(..)`.
  pub fn spot( location : Vec3, direction : Vec3, angle : f32, color : Color3, strength : f32 ) -> Light {
    Light::Spot( SpotLight::new( location, direction, angle, color, strength ) )
  }
}
