/// A point light
pub struct PointLight {
  location : Vec3,
  color    : Color3
}

impl PointLight {
  /// Constructs a new light of the given color at the provided location
  pub fn new( location : Vec3, color : Color3 ) -> Light {
    Light { location, color }
  }
}
