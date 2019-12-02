use crate::graphics::{ Color3, Material, Scene };
use crate::graphics::lights::PointLight;
use crate::graphics::primitives::{ AARect, Plane, Sphere };
use crate::graphics::ray::{ Tracable };
use crate::math::Vec3;
use crate::math;

// A simple scene with one blue sphere
pub fn setup_ball_scene( ) -> Scene {
  let light = PointLight::new( Vec3::new( -0.5, 2.0, 1.0 ), Color3::new( 0.7, 0.7, 0.7 ) );

  let mut shapes: Vec< Box< dyn Tracable > > = Vec::new( );
  shapes.push( Box::new( Sphere::new( Vec3::new( 0.0, 0.0, 5.0 ), 1.0, Material::diffuse( Color3::new( 0.0, 0.0, 1.0 ) ) ) ) );

  Scene::new( vec![ light ], shapes )
}

// A scene with a glass cube that has a non-glass (filled with air) sphere inside
pub fn setup_scene_ballsphere( ) -> Scene {
  let light = PointLight::new( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ) );

  let mut shapes: Vec< Box< dyn Tracable > > = Vec::new( );

  // 2 background spheres
  shapes.push( Box::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ) ) ) );
  // The cube with the hole
  shapes.push( Box::new( AARect::cube( Vec3::new(  0.0, 0.5 + math::EPSILON * 2.0, 4.0 ), 1.0, Material::refract( Vec3::new( 0.7, 0.2, 0.1 ), 1.5 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new(  0.0, 0.5, 4.0 ), 0.7, Material::refract( Vec3::new( 1.0, 0.0, 0.0 ), 1.0 ) ) ) );
  // 6 bounding planes
  shapes.push( Box::new( Plane::new( Vec3::new(  0.0, -1.0,   0.0 ), Vec3::new(  0.0,  1.0,  0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new(  0.0,  8.0,   0.0 ), Vec3::new(  0.0, -1.0,  0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( -6.0,  0.0,   0.0 ), Vec3::new(  1.0,  0.0,  0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new(  6.0,  0.0,   0.0 ), Vec3::new( -1.0,  0.0,  0.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new(  0.0,  0.0,  13.0 ), Vec3::new(  0.0,  0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new(  0.0,  0.0, -13.0 ), Vec3::new(  0.0,  0.0,  1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );

  Scene::new( vec![ light ], shapes )
}

// A scene with refractive and semi-specular objects
pub fn setup_scene( ) -> Scene {
  let light = PointLight::new( Vec3::new( 0.0, 6.0, 2.0 ), Color3::new( 0.7, 0.7, 0.7 ) );

  let mut shapes: Vec< Box< dyn Tracable > > = Vec::new( );
  // some random shapes
  shapes.push( Box::new( Sphere::new( Vec3::new(  0.0, 1.0, 5.0 ), 1.0, Material::refract( Vec3::new( 0.3, 0.6, 0.3 ), 1.5 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new( -1.2, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 1.0, 0.0 ), 0.2 ) ) ) );
  shapes.push( Box::new( Sphere::new( Vec3::new(  1.0, 0.0, 10.0 ), 1.0, Material::reflect( Color3::new( 0.0, 0.0, 1.0 ), 0.3 ) ) ) );
  shapes.push( Box::new( AARect::cube( Vec3::new(  -1.7, 0.0 + math::EPSILON * 2.0, 7.0 ), 1.0, Material::refract( Vec3::new( 0.7, 0.2, 0.1 ), 1.5 ) ) ) );
  // 2 bounding planes
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, -1.0, 0.0 ), Vec3::new( 0.0, 1.0, 0.0 ), Material::reflect( Color3::new( 1.0, 1.0, 1.0 ), 0.1 ) ) ) );
  shapes.push( Box::new( Plane::new( Vec3::new( 0.0, 0.0, 13.0 ), Vec3::new( 0.0, 0.0, -1.0 ), Material::diffuse( Color3::new( 1.0, 1.0, 1.0 ) ) ) ) );

  Scene::new( vec![ light ], shapes )
}
