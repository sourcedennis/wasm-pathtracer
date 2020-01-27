use std::fmt;
// Local imports
use crate::math::Vec3;
use crate::graphics::AABB;
use crate::math::EmpiricalPDF;
use crate::rng::Rng;

// This technique is adapted from Andreas Mikolajewski's master thesis:
// "Efficient data structures and sampling of many light sources for Next Event Estimation"
//
// An octree is built that contains photons. At each node in this tree a
// Cumulative Probability Distribution function is stored.
// Whenever a hit point is queried, it looks at the cell within which that
// point lies, as well as the 7 adjacent cells (at the same level).
// The sample is then taken by interpolating the probabilities in in these
// cells, proportional to "how much the vertex is in that cell".

#[derive(Debug)]
pub struct PhotonTree {
  num_lights : usize,
  root       : Octree,
  size       : f32
}

/// The identifier of a light within the scene
type LightId = usize;

/// Once the number of photons in a cell exceeds this amount, it is subdivided
static MAX_PHOTONS_IN_CELL : usize = 1024;

/// An octree node. Each internal node has 8 children
/// Note that all nodes have an associated CDF.
enum Octree {
  Node {
    cdf      : EmpiricalPDF,
    children : Vec< Octree >
  },
  Leaf {
    cdf    : EmpiricalPDF,
    values : Vec< ( LightId, Vec3, f32 ) >
  }
}

impl PhotonTree {
  /// Constructs a new PhotonTree
  /// It needs to know the number of lights in the scene, such that it can
  ///   some positive probability for each light, at least.
  pub fn new( num_lights : usize ) -> PhotonTree {
    PhotonTree {
      num_lights
    , root: Octree::Leaf { values: Vec::new( ), cdf: EmpiricalPDF::new( num_lights ) }
      // Place the octree around (-1024,-1024,-1024)-(1024, 1024, 1024)
      // This doesn't scale on infinitely sized scenes, but suffices for now
    , size: 1024.0
    }
  }

  /// Inserts a new photon into the tree
  /// The intensity represents the color by a single value
  ///   (typically max(r,g,b) is a good choice)
  pub fn insert( &mut self, light_id : LightId, location : Vec3, intensity : f32 ) -> bool {
    if location.x < -self.size && location.x > self.size &&
       location.y < -self.size && location.y > self.size &&
       location.z < -self.size && location.z > self.size {
      return false;
    }

    self.root.insert(
      self.num_lights
    , AABB::new1( -self.size, -self.size, -self.size, self.size, self.size, self.size )
    , light_id
    , location
    , intensity
    );
    true
  }

  /// Samples a light source for the point `v`. The probability of picking that
  /// particular light source is also returned.
  pub fn sample( &mut self, rng : &mut Rng, v : Vec3 ) -> (LightId, f32) {
    // Interpolate the CDFs

    if v.x < -self.size || v.y < -self.size || v.z < -self.size || v.x > self.size || v.y > self.size || v.z > self.size {
      return ( rng.next_in_range(0, self.num_lights), 1.0 / self.num_lights as f32 );
    }
    
    let self_bounds = AABB::new1( -self.size, -self.size, -self.size, self.size, self.size, self.size );
    let (_, bounds, depth) = self.root.find_leaf( self_bounds, 0, v );
    
    let (weight_x, weight_adj_x, x_off) =
      if v.x > bounds.center( ).x { // Go to the right
        let left_weight  = ( bounds.x_max - ( v.x - bounds.x_size( ) * 0.5 ) ) / bounds.x_size( );
        let right_weight = 1.0 - left_weight;
        ( left_weight, right_weight, 1.0 )
      } else { // Go to the left
        let right_weight = ( ( v.x + bounds.x_size( ) * 0.5 ) - bounds.x_min ) / bounds.x_size( );
        let left_weight  = 1.0 - right_weight;
        ( right_weight, left_weight, -1.0 )
      };
    assert!( weight_x >= 0.0 && weight_x <= 1.0 && weight_adj_x >= 0.0 && weight_adj_x <= 1.0 );
      
    let (weight_y, weight_adj_y, y_off) =
      if v.y > bounds.center( ).y { // Go to the right
        let left_weight  = ( bounds.y_max - ( v.y - bounds.y_size( ) * 0.5 ) ) / bounds.y_size( );
        let right_weight = 1.0 - left_weight;
        ( left_weight, right_weight, 1.0 )
      } else { // Go to the left
        let right_weight = ( ( v.y + bounds.y_size( ) * 0.5 ) - bounds.y_min ) / bounds.y_size( );
        let left_weight  = 1.0 - right_weight;
        ( right_weight, left_weight, -1.0 )
      };
    assert!( weight_y >= 0.0 && weight_y <= 1.0 && weight_adj_y >= 0.0 && weight_adj_y <= 1.0 );
      
    let (weight_z, weight_adj_z, z_off) =
      if v.z > bounds.center( ).z { // Go to the right
        let left_weight  = ( bounds.z_max - ( v.z - bounds.z_size( ) * 0.5 ) ) / bounds.z_size( );
        let right_weight = 1.0 - left_weight;
        ( left_weight, right_weight, 1.0 )
      } else { // Go to the left
        let right_weight = ( ( v.z + bounds.z_size( ) * 0.5 ) - bounds.z_min ) / bounds.z_size( );
        let left_weight  = 1.0 - right_weight;
        ( right_weight, left_weight, -1.0 )
      };
    assert!( weight_z >= 0.0 && weight_z <= 1.0 && weight_adj_z >= 0.0 && weight_adj_z <= 1.0 );
    
    // Sample from it's own Octree cell? (Or from an adjacent cell?)
    let sample_self_x = rng.next( ) <= weight_x;
    let sample_self_y = rng.next( ) <= weight_y;
    let sample_self_z = rng.next( ) <= weight_z;

    // Lookup `v` as if it were in an adjacent cell
    let sampled_v   =
      v +
      if sample_self_x { Vec3::ZERO } else { x_off * Vec3::new( bounds.x_size( ), 0.0, 0.0 ) } +
      if sample_self_y { Vec3::ZERO } else { y_off * Vec3::new( 0.0, bounds.y_size( ), 0.0 ) } +
      if sample_self_z { Vec3::ZERO } else { z_off * Vec3::new( 0.0, 0.0, bounds.z_size( ) ) };

    let sampled_cdf = self.root.find_node_cdf( self_bounds, depth, sampled_v );
    let res = sampled_cdf.sample( rng );

    // Now find the PDF weighted over all neighbours
    let mut pdf = 0.0;

    let ajx = bounds.x_size( ) * x_off;
    let ajy = bounds.y_size( ) * y_off;
    let ajz = bounds.z_size( ) * z_off;

    // Bilinear interpolation of the probability of picking `res` over the adjacent nodes
    pdf += self.root.find_node_cdf( self_bounds, depth, v ).bin_prob( res ) * weight_x * weight_y * weight_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( ajx, 0.0, 0.0 ) ).bin_prob( res ) * weight_adj_x * weight_y * weight_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( 0.0, ajy, 0.0 ) ).bin_prob( res ) * weight_x * weight_adj_y * weight_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( 0.0, 0.0, ajz ) ).bin_prob( res ) * weight_x * weight_y * weight_adj_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( ajx, ajy, 0.0 ) ).bin_prob( res ) * weight_adj_x * weight_adj_y * weight_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( 0.0, ajy, ajz ) ).bin_prob( res ) * weight_x * weight_adj_y * weight_adj_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( ajx, 0.0, ajz ) ).bin_prob( res ) * weight_adj_x * weight_y * weight_adj_z;
    pdf += self.root.find_node_cdf( self_bounds, depth, v + Vec3::new( ajx, ajy, ajz ) ).bin_prob( res ) * weight_adj_x * weight_adj_y * weight_adj_z;

    (res, pdf)
  }
}

impl Octree {
  /// Inserts a photon at `location` into the tree
  /// As octrees don't store their own bounds, this needs to be passed as well
  pub fn insert( &mut self, num_lights : usize, self_bounds : AABB, light_id : LightId, location : Vec3, intensity : f32 ) {
    match self {
      Octree::Node { cdf, children } => {
        cdf.add( light_id, intensity );

        let (child_index, child_bounds) =
          child( self_bounds, location );

        children[ child_index ].insert( num_lights, child_bounds, light_id, location, intensity );
      },
      Octree::Leaf { cdf, values } => {
        cdf.add( light_id, intensity );
        values.push( ( light_id, location, intensity ) );

        if values.len( ) > MAX_PHOTONS_IN_CELL {
          let mut children = Vec::with_capacity( 8 );
          for _i in 0..8 {
            children.push( Octree::Leaf { cdf: EmpiricalPDF::new( num_lights ), values: vec![] } );
          }

          let mut new_self =
            Octree::Node { cdf: EmpiricalPDF::new( num_lights ), children };

          for (lid, v, ins) in values {
            new_self.insert( num_lights, self_bounds, *lid, *v, *ins );
          }

          *self = new_self;
        }
      }
    }
  }

  /// Returns properties of the smallest cell containing `location`
  /// As nodes don't store their bounds or depth, these need to be provided
  ///   (start at depth 0)
  pub fn find_leaf< 'a >( &'a mut self, self_bounds : AABB, depth : usize, location : Vec3 ) -> ( &'a mut EmpiricalPDF, AABB, usize ) {
    match self {
      Octree::Node { children, .. } => {
        let (child_index, child_bounds) = child( self_bounds, location );
        children[ child_index ].find_leaf( child_bounds, depth + 1, location )
      },
      Octree::Leaf { cdf, .. } => {
        ( cdf, self_bounds, depth )
      }
    }
  }

  /// Returns the distribution function for a point located at `location` within
  /// the Octree. It will *not* look deeper than `depth`; if this finds an
  /// internal node instead, that node's CDF is returned.
  pub fn find_node_cdf< 'a >( &'a mut self, self_bounds : AABB, depth : usize, location : Vec3 ) -> &'a mut EmpiricalPDF {
    match self {
      Octree::Node { cdf, children } => {
        if depth == 0 {
          cdf
        } else {
          let (child_index, child_bounds) =
            child( self_bounds, location );
          children[ child_index ].find_node_cdf( child_bounds, depth - 1, location )
        }
      },
      Octree::Leaf { cdf, .. } => {
        cdf
      }
    }
  }
}

// Computes the child ID and AABB of an octree node
fn child( bounds : AABB, v : Vec3 ) -> ( usize, AABB ) {
  let c = bounds.center( );

  let i =
    if v.x < c.x { 0 } else { 4 } +
    if v.y < c.y { 0 } else { 2 } +
    if v.z < c.z { 0 } else { 1 };
  
  let (x_min, x_max) =
    if v.x < c.x { (bounds.x_min, c.x) } else { (c.x, bounds.x_max) };
  let (y_min, y_max) =
    if v.y < c.y { (bounds.y_min, c.y) } else { (c.y, bounds.y_max) };
  let (z_min, z_max) =
    if v.z < c.z { (bounds.z_min, c.z) } else { (c.z, bounds.z_max) };
  
  ( i, AABB::new1( x_min, y_min, z_min, x_max, y_max, z_max ) )
}

#[allow(unused_must_use)]
impl fmt::Debug for Octree {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Octree::Node { cdf, children } => {
        write!( f, "Node {{ cdf: {:?}, children: {:?} }}", cdf, children )
      },
      Octree::Leaf { cdf, values } => {
        if values.len( ) == 0 {
          write!( f, "Empty" )
        } else {
          write!( f, "Leaf {{ #v: {}, cdf: {:?} }}", values.len( ), cdf )
        }
      }
    }
  }
}
