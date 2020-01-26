use crate::math::Vec3;
use crate::graphics::AABB;
use std::collections::HashMap;

pub struct PhotonTree {
  // Have one tree per light source
  roots : HashMap< usize, Box< KDTreeNode > >
}

struct KDTreeNode {
  value         : Vec3,
  bounds        : AABB,
  sum_intensity : f32,
  left          : Option< Box< KDTreeNode > >,
  right         : Option< Box< KDTreeNode > >
}

impl PhotonTree {
  pub fn new( ) -> PhotonTree {
    PhotonTree { roots: HashMap::new( ) }
  }

  pub fn insert( &mut self, v : Vec3, intensity : f32, light_id : usize ) {
    let mut some_tree : &mut Option<Box<KDTreeNode>>;

    if let Some( tree ) = self.roots.get_mut( &light_id ) {
      tree.bounds = tree.bounds.include( v );
      tree.sum_intensity += intensity;
      if v.x < tree.value.x {
        some_tree = &mut tree.left;
      } else {
        some_tree = &mut tree.right;
      }

      for _i in 0..15 { // Set a depth cap
        // Insert y
        if let Some( ref mut node ) = some_tree {
          node.bounds = node.bounds.include( v );
          node.sum_intensity += intensity;
          if v.y < node.value.y {
            some_tree = &mut tree.left;
          } else {
            some_tree = &mut tree.right;
          }
        } else {
          *some_tree = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) );
          return;
        }
  
        // Insert z
        if let Some( ref mut node ) = some_tree {
          node.bounds = node.bounds.include( v );
          node.sum_intensity += intensity;
          if v.z < node.value.z {
            some_tree = &mut tree.left;
          } else {
            some_tree = &mut tree.right;
          }
        } else {
          *some_tree = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) );
          return;
        }
  
        // Insert x
        if let Some( ref mut node ) = some_tree {
          node.bounds = node.bounds.include( v );
          node.sum_intensity += intensity;
          if v.x < node.value.x {
            some_tree = &mut tree.left;
          } else {
            some_tree = &mut tree.right;
          }
        } else {
          *some_tree = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) );
          return;
        }
      }
    } else {
      self.roots.insert( light_id, Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) );
    }
  }

  pub fn query_cdf( &self, dst : &mut Vec< (usize, f32) >, v : &Vec3 ) {
    let mut sum = 0.0;

    dst.clear( );

    for light_id in self.roots.keys( ) {
      if let Some( tree ) = self.roots.get( &light_id ) {
        if tree.bounds.contains_point( v ) {
          let c = ( tree.bounds.area( ) / tree.sum_intensity as f32 ).max( find_contribution( &tree.left, v ) ).max( find_contribution( &tree.right, v ) );
          dst.push( ( *light_id, c ) );
          sum += c;
        }
      }
    }

    if sum > 0.0 {
      let mut offset = 0.0;
      for i in 0..dst.len( ) {
        let (light_id, weight) = dst[ i ];
        dst[ i ] = (light_id, offset);
        offset += weight / sum;
      }
    }
  }
}

fn find_contribution( node: &Option< Box< KDTreeNode > >, v : &Vec3 ) -> f32 {
  match node {
    None => 0.0,
    Some( n ) => {
      if n.bounds.contains_point( v ) {
        // Return the photon density in the area around `v`, for this particular light
        ( n.bounds.area( ) / n.sum_intensity as f32 ).max( find_contribution( &n.left, v ) ).max( find_contribution( &n.right, v ) )
      } else {
        0.0
      }
    }
  }
}

// fn insert_x( m_node : &mut Option< Box< KDTreeNode > >, v : Vec3, intensity : f32 ) {
//   if let Some( ref mut node ) = m_node {
//     node.bounds = node.bounds.include( v );
//     node.sum_intensity += intensity;
//     if v.x < node.value.x {
//       insert_y( &mut node.left, v, intensity );
//     } else {
//       insert_y( &mut node.right, v, intensity );
//     }
//   } else {
//     *m_node = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) )
//   }
// }

// fn insert_y( m_node : &mut Option< Box< KDTreeNode > >, v : Vec3, intensity : f32 ) {
//   if let Some( ref mut node ) = m_node {
//     node.bounds = node.bounds.include( v );
//     node.sum_intensity += intensity;
//     if v.y < node.value.y {
//       insert_z( &mut node.left, v, intensity );
//     } else {
//       insert_z( &mut node.right, v, intensity );
//     }
//   } else {
//     *m_node = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) )
//   }
// }

// fn insert_z( m_node : &mut Option< Box< KDTreeNode > >, v : Vec3, intensity : f32 ) {
//   if let Some( ref mut node ) = m_node {
//     node.bounds = node.bounds.include( v );
//     node.sum_intensity += intensity;
//     if v.z < node.value.z {
//       insert_x( &mut node.left, v, intensity );
//     } else {
//       insert_x( &mut node.right, v, intensity );
//     }
//   } else {
//     *m_node = Some( Box::new( KDTreeNode { value: v, sum_intensity: intensity, bounds: AABB::new1( v.x, v.y, v.z, v.x, v.y, v.z ), left: None, right: None } ) )
//   }
// }
