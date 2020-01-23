
/// A Stack
/// For example, when tracking materials along a ray that has a
///   maximum bouncing depth =D
pub struct DefaultStack< T > {
  default_val : T,
  data        : Vec< T >
}

impl< T: Clone + Copy > DefaultStack< T > {
  /// Constructs a new Stack with the given capacity
  /// The `default_val` is used to fill the stack, which is necessary
  ///   as the whole array is allocated upon stack construction
  pub fn new( initial_capacity : usize, default_val : T ) -> DefaultStack< T > {
    DefaultStack { default_val, data: Vec::with_capacity( initial_capacity ) }
  }

  // Constructs a new Stack with the given capacity, and already one default value
  // pushed unto the stack
  pub fn new1( initial_capacity : usize, default_val : T ) -> DefaultStack< T > {
    let mut s = DefaultStack::new( initial_capacity, default_val );
    s.push( default_val );
    s
  }

  /// Pushes an element to the stack
  /// Does *not* check capacity violations
  pub fn push( &mut self, v : T ) {
    self.data.push( v );
  }

  /// Only pop if there are *more* than 1 object on the stack
  ///
  /// For raytracing: Useful to always keep the air material on the stack
  pub fn pop_until1( &mut self ) -> Option< T > {
    if self.data.len( ) > 1 {
      self.data.pop( )
    } else {
      None
    }
  }

  /// Returns a reference to the top of the stack
  /// Returns Nothing if no such element
  pub fn top< 'a >( &'a self ) -> Option< &'a T > {
    let len = self.data.len( );
    if len > 0 {
      Some( &self.data[ len - 1 ] )
    } else {
      None
    }
  }
}
