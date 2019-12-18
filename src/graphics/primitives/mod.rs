mod aa_rect;
mod plane;
mod sphere;
mod square;
mod torus;
mod triangle;

pub use aa_rect::AARect;
pub use plane::Plane;
pub use sphere::Sphere;
pub use square::Square;
pub use torus::Torus;
pub use triangle::Triangle;

use crate::math::{Vec3};
use crate::graphics::ray::{Ray, Tracable, Bounded, Hit};
use crate::graphics::{Material, AABB};

// // A union type over all primitives
// // Avoids having to Box (individually allocate) each of them
// #[derive(Debug, Clone)]
// pub enum TracablePrimitive {
//   AARect( AARect ),
//   Plane( Plane ),
//   Sphere( Sphere ),
//   Square( Square ),
//   Torus( Torus ),
//   Triangle( Triangle )
// }

// impl TracablePrimitive {
//   pub fn aa_rect( x_min : f32, x_max : f32, y_min : f32, y_max : f32, z_min : f32, z_max : f32, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::AARect( AARect::new( x_min, x_max, y_min, y_max, z_min, z_max, mat ) )
//   }

//   pub fn cube( center : Vec3, half_len : f32, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::AARect( AARect::cube( center, half_len, mat ) )
//   }

//   pub fn plane( location : Vec3, normal : Vec3, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::Plane( Plane::new( location, normal, mat ) )
//   }

//   pub fn sphere( location : Vec3, radius : f32, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::Sphere( Sphere::new( location, radius, mat ) )
//   }

//   pub fn square( location : Vec3, size : f32, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::Square( Square::new( location, size, mat ) )
//   }

//   pub fn torus( location : Vec3, big_r : f32, small_r : f32, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::Torus( Torus::new( location, big_r, small_r, mat ) )
//   }

//   pub fn triangle( v0 : Vec3, v1 : Vec3, v2 : Vec3, mat : Material ) -> TracablePrimitive {
//     TracablePrimitive::Triangle( Triangle::new( v0, v1, v2, mat ) )
//   }

//   fn me< 'a >( &'a self ) -> &'a dyn Tracable {
//     match self {
//       TracablePrimitive::AARect( r )   => r
//     , TracablePrimitive::Plane( r )    => r
//     , TracablePrimitive::Sphere( r )   => r
//     , TracablePrimitive::Square( r )   => r
//     , TracablePrimitive::Torus( r )    => r
//     , TracablePrimitive::Triangle( r ) => r
//     }
//   }
// }

// impl Bounded for TracablePrimitive {
//   fn location( &self ) -> Option< Vec3 > {
//     self.me( ).location( )
//   }

//   fn aabb( &self ) -> Option< AABB > {
//     self.me( ).aabb( )
//   }
// }

// impl Tracable for TracablePrimitive {
//   fn trace( &self, ray: &Ray ) -> Option< Hit > {
//     self.me( ).trace( ray )
//   }

//   fn trace_simple( &self, ray : &Ray ) -> Option< f32 > {
//     self.me( ).trace_simple( ray )
//   }
// }
