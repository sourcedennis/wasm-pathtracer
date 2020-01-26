// External imports
use std::fmt;
use std::f32::consts::PI;
// Local imports
use crate::graphics::Color3;
//use crate::graphics::Texture;
use crate::math::{ Vec2, Vec3 };
use crate::rng::Rng;

// Exports:
// * Material
// * PointMaterial

/// A description of visual characteristics for a 3d shape
#[derive(Clone)]
pub enum Material {
  Diffuse { color : Color3 },
  Glossy { specular : Color3, alpha : f32 },
  // A light source. The intensity over its whole surface
  Emissive { intensity : Vec3 }
}

impl Material {
  // Constructs a new diffuse material
  pub fn diffuse( color : Color3 ) -> Material {
    //Material::Microfacet { color, alpha: 1.0 }
    Material::Diffuse { color }
  }

  pub fn glossy( specular : Color3, alpha : f32 ) -> Material {
    Material::Glossy { specular, alpha }
  }

  // Constructs a new emissive material
  pub fn emissive( intensity : Vec3 ) -> Material {
    Material::Emissive { intensity }
  }

  /// Returns true if the material is emissive
  pub fn is_emissive( &self ) -> bool {
    match self {
      Material::Emissive { .. } => true,
      _ => false
    }
  }

  /// Evaluates the material generally to a `PointMaterial` if possible.
  /// If a material cannot be generally evaluated (as they vary per
  ///   surface-point) it returns `None`.
  pub fn evaluate_simple( &self ) -> Option< PointMaterial > {
    Some( self.evaluate_at( &Vec2::ZERO ) )
  }

  /// The way `Material`s are defined, they can be evaluated at a specific
  ///   point on their 2d-space (which supposedly corresponds to a 3d surface
  ///   point). The produces a `PointMaterial`.
  /// `v` should be within the range (0,1)x(0,1)
  pub fn evaluate_at( &self, v : &Vec2 ) -> PointMaterial {
    match self {
      Material::Diffuse { color } =>
        PointMaterial::diffuse( *color ),
      Material::Glossy { specular, alpha } =>
        PointMaterial::glossy( *specular, *alpha ),
      Material::Emissive { intensity } =>
        PointMaterial::emissive( *intensity )
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
  /// See `Material::Diffuse`
  Diffuse { color : Color3 },
  /// See `Material::Glossy`
  Glossy { specular : Color3, alpha : f32 },
  /// See `Material::Refract`
  Emissive { intensity : Vec3 }
}

impl PointMaterial {
  /// See `Material::diffuse`
  pub fn diffuse( color : Color3 ) -> PointMaterial {
    PointMaterial::Diffuse { color }
  }

  pub fn glossy( specular : Color3, alpha : f32 ) -> PointMaterial {
    PointMaterial::Glossy { specular, alpha }
  }

  /// See `Material::refract`
  pub fn emissive( intensity : Vec3 ) -> PointMaterial {
    PointMaterial::Emissive { intensity }
  }

  pub fn is_diffuse( &self ) -> bool {
    match self {
      PointMaterial::Diffuse { .. } => true,
      _ => false
    }
  }

  /// Returns a random outgoing direction `wi`
  pub fn sample_hemisphere( &self, rng : &mut Rng, wo : &Vec3, normal : &Vec3 ) -> (Vec3, f32) {
    match self {
      PointMaterial::Diffuse { color } => {
        // Diffuse
        let r1 = rng.next( );
        let r2 = rng.next( );
    
        let x = ( 2.0 * PI * r1 ).cos( ) * ( 1.0 - r2 ).sqrt( );
        let y = r2.sqrt( );
        let z = ( 2.0 * PI * r1 ).sin( ) * ( 1.0 - r2 ).sqrt( );
        
        // The normal points along the y axis (in point space). Find some tangents
        let x_normal = normal.orthogonal( );
        let z_normal = normal.cross( x_normal );

        let wi = ( x * x_normal + y * (*normal) + z * z_normal ).normalize( );
    
        ( wi, wi.dot( *normal ) / PI )
      },
      PointMaterial::Glossy { specular, alpha } => {
        ( rng.next_hemisphere( &normal ), 1.0 / ( 2.0 * PI ) )
      },
      PointMaterial::Emissive { .. } => panic!( "Light source" )
    }
  }

  // pub fn pdf( &self, normal : &Vec3, wo : &Vec3, wi : &Vec3 ) -> f32 {
  //   match self {
  //     PointMaterial::Diffuse { .. } => {
  //       //wi.dot( *normal ) / PI
  //       1.0 / ( 2.0 * PI )
  //     },
  //     PointMaterial::Emissive { .. } => panic!( "Light source" )
  //   }
  // }

  pub fn brdf( &self, normal : &Vec3, wo : &Vec3, wi : &Vec3 ) -> Color3 {
    match self {
      PointMaterial::Diffuse { color } =>
        (*color) / PI,
      PointMaterial::Glossy { specular, alpha } => {
        let h = ( *wo + *wi ).normalize( );
        let dh = ( ( alpha + 2.0 ) / ( 2.0 * PI ) ) * ( normal.dot( h ) ** alpha );
        let fresnel = specular.to_vec3( ) + ( Vec3::new(1.0,1.0,1.0) - specular.to_vec3( ) ) * ( ( 1.0 - wi.dot( h ) ).powf( 5.0 ) );
        let geometry = 1.0;

        Color3::from_vec3( ( fresnel * geometry * dh ) / ( 4.0 * normal.dot( *wi ) * normal.dot( *wo ) ) )
      },
      PointMaterial::Emissive { .. } => panic!( "Light source" )
    }
  }

  /// A physically *inaccurate* color, can be used for testing non-lit scenes
  pub fn test_color( &self ) -> Color3 {
    match self {
      PointMaterial::Diffuse { color } =>
        *color,
      PointMaterial::Glossy { specular, alpha } =>
        *specular,
      PointMaterial::Emissive { intensity } =>
        Color3::from_vec3( intensity.normalize( ) )
    }
  }
}

/// Nicely prints a Material for debugging
/// Note that not all elements are printed in all cases. When no Phong components
///   are printed, it may be assumed they are absent.
impl fmt::Debug for Material {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Material::Diffuse { color } => {
        write!( f, "Material::Diffuse {{ color: {:?} }}", color )
      },
      Material::Glossy { specular, alpha } => {
        write!( f, "Material::Glossy {{ specular: {:?}, alpha: {:?} }}", specular, alpha )
      },
      Material::Emissive { intensity } => {
        write!( f, "Material::Emissive {{ intensity: {:?} }}", intensity )
      }
    }
  }
}
