// External imports
use std::collections::HashMap;
use std::f32::consts::PI;
use std::rc::Rc;
// Local imports
use crate::graphics::{ Color3, Material, Scene, Texture };
use crate::graphics::lights::Light;
use crate::graphics::primitives::{ AARect, Plane, Sphere, Triangle, Square, Torus };
use crate::graphics::ray::{ Tracable };
use crate::graphics::Mesh;
use crate::math::Vec3;
use crate::math;

static MESH_CLOUD_100  : u32 = 0;
static MESH_CLOUD_10K  : u32 = 1;
static MESH_CLOUD_100K : u32 = 2;

// A scene with refractive and semi-specular objects
// pub fn setup_scene( ) -> Scene {
//   let light = Light::point( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ), 60.0 );

//   let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );
//   // some random shapes
//   shapes.push( Rc::new( Sphere::new( Vec3::new(  0.0, 1.0, 5.0 ), 1.0, Material::refract( Vec3::new( 0.3, 0.6, 0.3 ), 1.5 ).set_specular( 0.008, 10.0 ) ) ) );
//   shapes.push( Rc::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ).set_specular( 0.008, 10.0 ) ) ) );
//   shapes.push( Rc::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ).set_specular( 0.008, 10.0 ) ) ) );
//   shapes.push( Rc::new( AARect::cube( Vec3::new(  -1.7, 0.0 + math::EPSILON * 2.0, 7.0 ), 1.0, Material::refract( Vec3::new( 0.7, 0.2, 0.1 ), 1.5 ) ) ) );
//   // 2 bounding planes
//   shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
//   shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );

//   Scene::new( Color3::BLACK, vec![ light ], shapes )
// }

// A simple scene with one blue sphere
// pub fn setup_scene_ball( ) -> Scene {
//   let dir_light = Light::directional( Vec3::unit( -1.0, -1.0, 0.0 ), Color3::new( 0.2, 0.2, 0.2 ) );
//   let light = Light::spot( Vec3::new( 0.0, 4.0, 5.0 ), Vec3::new( 0.0, -1.0, 0.0 ), PI / 6.0, Color3::new( 0.7, 0.7, 0.7 ), 11.0 );

//   let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );
//   shapes.push( Rc::new( Sphere::new( Vec3::new( 0.0, 0.0, 5.0 ), 0.3, Material::diffuse( Color3::new( 0.0, 0.0, 1.0 ) ) ) ) );
//   shapes.push( Rc::new( Torus::new( Vec3::new( 0.0, 0.0, 5.0 ), 0.7, 0.3, Material::diffuse( Color3::new( 1.0, 0.0, 1.0 ) ).set_specular( 0.03, 10.0 ) ) ) );
//   shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );

//   Scene::new( Color3::BLACK, vec![ light, dir_light ], shapes )
// }

// A scene with a glass cube that has a non-glass (filled with air) sphere inside
pub fn setup_scene_cubesphere( ) -> Scene {
  let light = Light::point( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ), 50.0 );

  let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );

  // 2 background spheres
  shapes.push( Rc::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ) ) ) );
  shapes.push( Rc::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ) ) ) );
  // The cube with the hole
  shapes.push( Rc::new( AARect::cube( Vec3::new(  0.0, 0.5 + math::EPSILON * 2.0, 4.0 ), 1.0, Material::refract( Vec3::new( 0.7, 0.2, 0.1 ), 1.5 ) ) ) );
  shapes.push( Rc::new( Sphere::new( Vec3::new(  0.0, 0.5, 4.0 ), 0.7, Material::refract( Vec3::new( 1.0, 0.0, 0.0 ), 1.0 ) ) ) );
  // 6 bounding planes
  shapes.push( Rc::new( Plane::new( Vec3::new(  0.0, -1.0,   0.0 ), Vec3::new(  0.0,  1.0,  0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new(  0.0,  8.0,   0.0 ), Vec3::new(  0.0, -1.0,  0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new( -6.0,  0.0,   0.0 ), Vec3::new(  1.0,  0.0,  0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new(  6.0,  0.0,   0.0 ), Vec3::new( -1.0,  0.0,  0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new(  0.0,  0.0,  13.0 ), Vec3::new(  0.0,  0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new(  0.0,  0.0, -13.0 ), Vec3::new(  0.0,  0.0,  1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );

  Scene::new( Color3::BLACK, vec![ light ], shapes )
}

// A scene with the .obj file loaded into it
pub fn setup_scene_cloud100( meshes : &HashMap< u32, Mesh > ) -> Scene {
  display_obj( meshes, MESH_CLOUD_100 )
}

pub fn setup_scene_cloud10k( meshes : &HashMap< u32, Mesh > ) -> Scene {
  display_obj( meshes, MESH_CLOUD_10K )
}

pub fn setup_scene_cloud100k( meshes : &HashMap< u32, Mesh > ) -> Scene {
  display_obj( meshes, MESH_CLOUD_100K )
}

fn display_obj( meshes : &HashMap< u32, Mesh >, mesh_id : u32 ) -> Scene {
  let light = Light::point( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ), 50.0 );
  
  let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );

  shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  
  if let Some( mesh ) = meshes.get( &mesh_id ) {
    for i in 0..(mesh.vertices.len()/3) {
      let mut triangle =
        Triangle::new( mesh.vertices[ i * 3 + 0 ] * 0.5, mesh.vertices[ i * 3 + 1 ] * 0.5, mesh.vertices[ i * 3 + 2 ] * 0.5
                     , Material::diffuse( Color3::new( 1.0, 0.4, 0.4 ) ) );
      triangle = triangle.translate( Vec3::new( 0.0, 0.0, 5.0 ) );
      shapes.push( Rc::new( triangle ) );
    }
  }

  Scene::new( Color3::BLACK, vec![ light ], shapes )
}

// Turner Whitted's scene
// pub fn setup_scene_texture( textures : &HashMap< u32, Texture > ) -> Scene {
//   let light = Light::point( Vec3::new( 0.0, 6.0, -3.0 ), Color3::new( 0.7, 0.7, 0.7 ), 50.0 );
  
//   let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );

//   if let Some( t ) = textures.get( &0 ) {
//     shapes.push( Rc::new( Square::new( Vec3::new( 0.0, -1.0, 4.0 ), 8.0, Material::diffuse_texture( t.clone( ) ) ) ) );
//   }
//   shapes.push( Rc::new( Sphere::new( Vec3::new( -1.3, 1.0, -0.2 ), 0.7, Material::refract( Vec3::new( 0.5, 1.0, 0.5 ), 1.02 ).set_specular( 0.008, 10.0 ) ) ) );
//   shapes.push( Rc::new( Sphere::new( Vec3::new( -0.4, 0.0, 1.0 ), 0.6, Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.3 ).set_specular( 0.008, 10.0 ) ) ) );


//   Scene::new( Color3::new( 135.0 / 255.0, 206.0 / 255.0, 250.0 / 255.0 )
//             , vec![ light ]
//             , shapes
//             )
// }
