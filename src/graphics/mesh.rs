use crate::math::{Vec3};
use crate::graphics::ray::{ Tracable };
use std::rc::Rc;

pub enum Mesh {
  Preload( Vec< Vec3 > ),
  // After loading, put the triangles into Rc boxes
  // This avoids having to do this upon scene construction
  Triangled( Vec< Rc< dyn Tracable > > )
}
