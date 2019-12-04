mod directional;
mod point;
mod spot;

pub use point::{PointLight};
pub use directional::{DirectionalLight};
pub use spot::{SpotLight};

use crate::math::Vec3;
use crate::graphics::Color3;

pub enum Light {
  Directional( DirectionalLight ),
  Point( PointLight ),
  Spot( SpotLight )
}

impl Light {
  pub fn directional( direction : Vec3, color : Color3 ) -> Light {
    Light::Directional( DirectionalLight::new( direction, color ) )
  }

  pub fn point( location : Vec3, color : Color3, strength : f32 ) -> Light {
    Light::Point( PointLight::new( location, color, strength ) )
  }

  pub fn spot( location : Vec3, direction : Vec3, angle : f32, color : Color3, strength : f32 ) -> Light {
    Light::Spot( SpotLight::new( location, direction, angle, color, strength ) )
  }
}
