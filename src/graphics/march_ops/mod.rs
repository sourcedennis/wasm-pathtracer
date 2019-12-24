use crate::graphics::ray::{Bounded, Marchable};
use crate::graphics::{AABB, Color3};
use crate::math::Vec3;

// Operators for ray marching

/// Union between two marchable shapes
/// Its SDF is the closest of the two shapes.
#[derive(Debug)]
pub struct Union {
  a : Box< dyn Marchable >,
  b : Box< dyn Marchable >
}

/// Intersections between two marchable shapes
/// Its SDF is the furthest of the two shapes.
#[derive(Debug)]
pub struct Intersection {
  a : Box< dyn Marchable >,
  b : Box< dyn Marchable >
}

/// Difference between two marchable shapes
/// Its SDF is that which is closest to the first, but furthest from the
///   second.
#[derive(Debug)]
pub struct Difference {
  a : Box< dyn Marchable >,
  b : Box< dyn Marchable >
}

impl Difference {
  pub fn new( a : Box< dyn Marchable >, b : Box< dyn Marchable > ) -> Difference {
    Difference { a, b }
  }
}

impl Bounded for Union {
  fn location( &self ) -> Option< Vec3 > {
    if let Some( a_loc ) = self.a.location( ) {
      if let Some( b_loc ) = self.b.location( ) {
        Some( ( a_loc + b_loc ) * 0.5 )
      } else {
        None
      }
    } else {
      None
    }
  }

  fn aabb( &self ) -> Option< AABB > {
    if let Some( a_aabb ) = self.a.aabb( ) {
      if let Some( b_aabb ) = self.b.aabb( ) {
        Some( a_aabb.join( &b_aabb ) )
      } else {
        None
      }
    } else {
      None
    }
  }
}

impl Bounded for Intersection {
  fn location( &self ) -> Option< Vec3 > {
    if let Some( a_loc ) = self.a.location( ) {
      if let Some( b_loc ) = self.b.location( ) {
        Some( ( a_loc + b_loc ) * 0.5 )
      } else {
        None
      }
    } else {
      None
    }
  }

  fn aabb( &self ) -> Option< AABB > {
    if let Some( a_aabb ) = self.a.aabb( ) {
      if let Some( b_aabb ) = self.b.aabb( ) {
        // TODO
        Some( a_aabb.join( &b_aabb ) )
      } else {
        None
      }
    } else {
      None
    }
  }
}

impl Bounded for Difference {
  fn location( &self ) -> Option< Vec3 > {
    if let Some( a_loc ) = self.a.location( ) {
      if let Some( b_loc ) = self.b.location( ) {
        Some( ( a_loc + b_loc ) * 0.5 )
      } else {
        None
      }
    } else {
      None
    }
  }

  fn aabb( &self ) -> Option< AABB > {
    self.a.aabb( )
  }
}

impl Marchable for Union {
  fn sdf( &self, p : &Vec3 ) -> f32 {
    self.a.sdf( p ).min( self.b.sdf( p ) )
  }
  
  fn color( &self, p : &Vec3 ) -> Color3 {
    let d1 = self.a.sdf( p );
    let d2 = self.b.sdf( p );

    if d1 < d2 {
      self.a.color( p )
    } else {
      self.b.color( p )
    }
  }
}

impl Marchable for Intersection {
  fn sdf( &self, p : &Vec3 ) -> f32 {
    self.a.sdf( p ).max( self.b.sdf( p ) )
  }
  
  fn color( &self, p : &Vec3 ) -> Color3 {
    let d1 = self.a.sdf( p );
    let d2 = self.b.sdf( p );

    if d1 < d2 {
      self.a.color( p )
    } else {
      self.b.color( p )
    }
  }
}

impl Marchable for Difference {
  fn sdf( &self, p : &Vec3 ) -> f32 {
    self.a.sdf( p ).max( -self.b.sdf( p ) )
  }
  
  fn color( &self, p : &Vec3 ) -> Color3 {
    let d2 = self.b.sdf( p );

    if d2 < 0.00001 {
      self.b.color( p )
    } else {
      self.a.color( p )
    }
  }
}
