// External imports
use std::collections::HashMap;
use std::rc::Rc;
// Local imports
use crate::graphics::{ Color3, Material, Scene };
use crate::graphics::lights::Light;
use crate::graphics::primitives::{ AARect, Plane, Sphere, Triangle, Torus };
use crate::graphics::ray::{ Tracable, Marchable };
use crate::graphics::Mesh;
use crate::math::Vec3;
use crate::math;
use crate::rng::Rng;

static MESH_BUNNY_HIGH : u32 = 1;

// A scene with a many lights and walls
pub fn setup_scene_museum( ) -> Scene {
  let mut shapes: Vec< Rc< dyn Tracable > > = Vec::new( );

  shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::diffuse( Color3::new( 0.7, 0.7, 0.7 ) ) ) ) );

  // ## Render the tori
  let xs = vec![ -16.0, -12.0, -8.0, -4.0, 0.0, 4.0, 8.0, 12.0, 16.0 ];
  let mut colors = vec![
      Color3::new( 1.0, 0.3, 0.3 )
    , Color3::new( 0.0, 1.0, 1.0 ), Color3::new( 0.3, 0.3, 1.0 ), Color3::RED
    , Color3::GREEN
    , Color3::BLUE, Color3::new( 1.0, 0.0, 1.0 ), Color3::new( 1.0, 1.0, 0.0 )
    , Color3::new( 0.3, 1.0, 0.3 )
    ];

  let mut rng = Rng::new( );
  rng.next( );
  rng.next( );

  for y in vec![ -7.5, 0.0, 7.5 ] {
    for i in 0..xs.len( ) {
      shapes.push( Rc::new( Torus::new(  Vec3::new(  xs[ i ], -0.5, y ), 1.3, 0.3, Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
      museum_lights( &mut shapes, xs[ i ], y, colors[ i ].to_vec3( ) * 2.5 );
    }
    rng.shuffle( &mut colors );
  }

  // ## Add the walls
  let xs = vec![ -14.0, -10.0, -6.0, -2.0, 2.0, 6.0, 10.0, 14.0 ];

  for x in xs {
    shapes.push( Rc::new( AARect::new( x - 0.1, x + 0.1, -1.0, 2.0, -20.0, 20.0, Material::diffuse( Color3::new( 0.7, 0.7, 0.7 ) ) ) ) );
  }
  shapes.push( Rc::new( AARect::new( -20.0, 20.0, -1.0, 2.0,  3.75 - 0.1,  3.75 + 0.1, Material::diffuse( Color3::new( 0.7, 0.7, 0.7 ) ) ) ) );
  shapes.push( Rc::new( AARect::new( -20.0, 20.0, -1.0, 2.0, -3.75 - 0.1, -3.75 + 0.1, Material::diffuse( Color3::new( 0.7, 0.7, 0.7 ) ) ) ) );

  Scene::new( Color3::BLACK, vec![ ], shapes )
}

fn museum_lights( dst : &mut Vec< Rc< dyn Tracable > >, x : f32, y : f32, color : Vec3 ) {
  let lc1 = Vec3::new( x - 1.0, 0.0, y + 2.8 );
  let lc2 = Vec3::new( x + 1.0, 0.0, y + 2.8 );
  let lc3 = Vec3::new( x + 1.0, 1.0, y + 2.5 );
  let lc4 = Vec3::new( x - 1.0, 1.0, y + 2.5 );
  dst.push( Rc::new( Triangle::new( lc3, lc2, lc1, Material::emissive( color ) ) ) );
  dst.push( Rc::new( Triangle::new( lc4, lc3, lc1, Material::emissive( color ) ) ) );
  
  let lc1 = Vec3::new( x - 1.0, 0.0, y - 2.8 );
  let lc2 = Vec3::new( x + 1.0, 0.0, y - 2.8 );
  let lc3 = Vec3::new( x + 1.0, 1.0, y - 2.5 );
  let lc4 = Vec3::new( x - 1.0, 1.0, y - 2.5 );
  dst.push( Rc::new( Triangle::new( lc3, lc2, lc1, Material::emissive( color ) ) ) );
  dst.push( Rc::new( Triangle::new( lc4, lc3, lc1, Material::emissive( color ) ) ) );
}

// A scene with the .obj file loaded into it
pub fn setup_scene_bunny_high( meshes : &HashMap< u32, Mesh > ) -> Scene {
  display_obj( meshes, MESH_BUNNY_HIGH )
}

fn display_obj( meshes : &HashMap< u32, Mesh >, mesh_id : u32 ) -> Scene {
  // let light = Light::point( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ), 50.0 );
  // let light2 = Light::point( Vec3::new( 0.0, 10.0, 12.0 ), Color3::new( 0.8, 0.8, 0.8 ), 30.0 );

  let mut shapes : Vec< Rc< dyn Tracable > > =
    if let Some( Mesh::Triangled( ts ) ) = meshes.get( &mesh_id ) {
      let num_triangles = ts.len( );
      let mut shapes : Vec< Rc< dyn Tracable > > = Vec::with_capacity( num_triangles + 2 );
      shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
      shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 0.8, 1.0, 0.8 ) ) ) ) );
      
      for t in ts {
        shapes.push( t.clone( ) );
      }

      shapes
    } else {
      let mut shapes : Vec< Rc< dyn Tracable > > = Vec::new( );
      shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
      shapes.push( Rc::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 0.8, 1.0, 0.8 ) ) ) ) );
      shapes
    };

  // Light corners
  let lc1 = Vec3::new( -1.0, 7.0,  0.0 );
  let lc2 = Vec3::new(  1.0, 7.0,  0.0 );
  let lc3 = Vec3::new(  1.0, 7.0,  2.0 );
  let lc4 = Vec3::new( -1.0, 7.0,  2.0 );
  // let lc1 = Vec3::new( -1.0, 7.0,  5.0 );
  // let lc2 = Vec3::new(  1.0, 7.0,  5.0 );
  // let lc3 = Vec3::new(  1.0, 7.0,  7.0 );
  // let lc4 = Vec3::new( -1.0, 7.0,  7.0 );
  shapes.push( Rc::new( Triangle::new( lc3, lc2, lc1, Material::emissive( Vec3::new( 16.0, 16.0, 16.0 ) ) ) ) );
  shapes.push( Rc::new( Triangle::new( lc4, lc3, lc1, Material::emissive( Vec3::new( 16.0, 16.0, 16.0 ) ) ) ) );

  Scene::new( Color3::BLACK, vec![ /*light, light2*/ ], shapes )
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
