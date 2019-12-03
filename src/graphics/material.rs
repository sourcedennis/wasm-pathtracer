use crate::graphics::Color3;
use crate::graphics::Texture;
use crate::math::{ Vec2, Vec3 };

// Exports:
// * Material
// * PointMaterial

/// A description of visual characteristics for a 3d shape
#[derive(Clone)]
pub enum Material {
  // Reflect with `reflection` set to 0.0 is diffuse
  Reflect { color : Color3, reflection : f32 },
  // A material with a texture
  // For now, store the textures within the material. Though, might want to make
  //   these references in the future, as this duplicates texture data somewhat
  //   unnecessarily. It does keep the interface/ownership management easier,
  //   and does not impact runtime performance, though.
  ReflectTexture { texture : Texture, reflection : f32 },
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

  // Constructs a new diffuse texture material
  pub fn diffuse_texture( texture : Texture ) -> Material {
    Material::ReflectTexture { texture, reflection: 0.0 }
  }

  // Constructs a new reflective material
  // Note that when `reflection` is 0, the material is diffuse
  pub fn reflect( color : Color3, reflection : f32 ) -> Material {
    Material::Reflect { color, reflection }
  }

  // Constructs a new reflective material with a texture
  // Note that when `reflection` is 0, the material is diffuse
  pub fn reflect_texture( texture : Texture, reflection : f32 ) -> Material {
    Material::ReflectTexture { texture, reflection }
  }

  /// Constructs a new refractive material
  /// See also the `Material::Refract` constructor
  pub fn refract( absorption : Vec3, refractive_index : f32 ) -> Material {
    Material::Refract { absorption, refractive_index }
  }

  pub fn evaluate_simple( &self ) -> Option< PointMaterial > {
    match self {
      Material::Reflect { .. }  =>
        Some( self.evaluate_at( &Vec2::ZERO ) ),
      Material::ReflectTexture { .. }  =>
        None,
      Material::Refract { .. } =>
        Some( self.evaluate_at( &Vec2::ZERO ) )
    }
  }

  /// The way `Material`s are defined, they can be evaluated at a specific
  ///   point on their 2d-space (which supposedly corresponds to a 3d surface
  ///   point). The produces a `PointMaterial`.
  /// `v` should be within the range (0,1)x(0,1)
  pub fn evaluate_at( &self, v : &Vec2 ) -> PointMaterial {
    match self {
      Material::Reflect { color, reflection } =>
        PointMaterial::reflect( *color, *reflection ),
      Material::ReflectTexture { texture, reflection } =>
        PointMaterial::reflect( texture.at( *v ), *reflection ),
      Material::Refract { absorption, refractive_index } =>
        PointMaterial::refract( *absorption, *refractive_index )
    }
  }
}

/// A `PointMaterial` defines the material at a *single* point
/// A `Material` defines visual properties over the entire surface
///   of a shape; where these properties may vary over individual locations
///   on the surface (such as with diffuse-/normal-/specular-maps).
/// The `PointMaterial` defines such a surface material evaluated at
///   *one specific point* on the surface
#[derive(Clone,Copy)]
pub enum PointMaterial {
  /// See `Material::Reflect`
  Reflect { color : Color3, reflection : f32 },
  /// See `Material::Refract`
  Refract { absorption : Vec3, refractive_index : f32 }
}

impl PointMaterial {
  /// See `Material::reflect`
  pub fn reflect( color : Color3, reflection : f32 ) -> PointMaterial {
    PointMaterial::Reflect { color, reflection }
  }

  /// See `Material::refract`
  pub fn refract( absorption : Vec3, refractive_index : f32 ) -> PointMaterial {
    PointMaterial::Refract { absorption, refractive_index }
  }
}
