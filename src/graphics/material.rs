use crate::graphics::color3::Color3;

// Exports:
// * Material
// * PointMaterial

/// A description of visual characteristics for a 3d shape
#[derive(Clone,Copy)]
pub enum Material {
  // Reflect with `reflection` set to 0.0 is diffuse
  Reflect { color : Color3, reflection : f32 },
  // Note that refracting objects do *not* have a diffuse color,
  //   as their perceived color is obtained by the semi-transparent
  //   color of their material.
  // This absorption is provided to Beer's law, which gives the
  //   amount of light that is absorped by the material.
  //   It should be a positive amount, whose values are the
  //   "inverse" of the object's color. So if a color is blue (0,0,1)
  //   then it absorbs the color (1,1,0).
  Refract { absorption : Vec3, refractive_index : f32 }
}

impl Material {
  // Constructs a new diffuse material
  pub fn diffuse( color : Color3 ) -> Material {
    Material::Reflect { color, reflection: 0.0 }
  }

  // Constructs a new reflective material
  // Note that when `reflection` is 0, the material is diffuse
  pub fn reflect( color : Color3, reflection : f32 ) -> Material {
    Material::Reflect { color, reflection }
  }

  /// Constructs a new refractive material
  /// See also the `Material::Refract` constructor
  pub fn refract( absorption : Vec3, refractive_index : f32 ) -> Material {
    Material::Refract { absorption, refractive_index }
  }

  /// The way `Material`s are defined, they can be evaluated at a specific
  ///   point on their 2d-space (which supposedly corresponds to a 3d surface
  ///   point). The produces a `PointMaterial`.
  pub fn evaluate_at( &self, v : &Vec2 ) -> PointMaterial {
    match self {
      Material::Reflect { c, r } =>
        PointMaterial::reflect( c, r ),
      Material::Refract { a, ri } =>
        PointMaterial::refract( a, ri )
    }
  }
}

/// A `PointMaterial` defines the material at a *single* point
/// A `Material` defines visual properties over the entire surface
///   of a shape; where these properties may vary over individual locations
///   on the surface (such as with diffuse-/normal-/specular-maps).
/// The `PointMaterial` defines such a surface material evaluated at
///   *one specific point* on the surface
pub enum PointMaterial {
  /// See `Material::Reflect`
  Reflect { color : Color3, reflection : f32 },
  /// See `Material::Refract`
  Refract { absorption : Vec3, refractive_index : f32 }
}
