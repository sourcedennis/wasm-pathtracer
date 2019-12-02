
/// A stack with a capacity limit (which will never be extended)
/// This is useful when an upperbound for the capacity is known,
///   but dynamic memory allocation at runtime should be avoided
///
/// For example, when tracking materials along a ray that has a
///   maximum bouncing depth =D
pub struct Stack< T > {
  data : Vec< T >,
  size : usize
}

impl< T: Clone + Copy > Stack< T > {
  /// Constructs a new Stack with the given capacity
  /// The `default_val` is used to fill the stack, which is necessary
  ///   as the whole array is allocated upon stack construction
  pub fn new( capacity : usize, default_val : T ) -> Stack< T > {
    Stack { data: vec![ default_val; capacity ], size: 0 }
  }

  // Constructs a new Stack with the given capacity, and already one default value
  // pushed unto the stack
  // Warning: Does not test whether `capacity >= 1`
  pub fn new1( capacity : usize, default_val : T ) -> Stack< T > {
    Stack { data: vec![ default_val; capacity ], size: 1 }
  }

  /// Pushes an element to the stack
  /// Does *not* check capacity violations
  pub fn push( &mut self, v : T ) {
    self.data[ self.size ] = v;
    self.size += 1;
  }

  /// Only pop if there are *more* than 1 objects on the stack
  ///
  /// For raytracing: Useful to always keep the air material on the stack
  pub fn pop_until1( &mut self ) -> Option< T > {
    if self.size > 1 {
      self.size -= 1;
      Some( self.data[ self.size ] )
    } else {
      None
    }
  }

  /// Returns a reference to the top of the stack
  /// Returns Nothing if no such element
  pub fn top< 'a >( &'a self ) -> Option< &'a T > {
    if self.size > 0 {
      Some( &self.data[ self.size - 1 ] )
    } else {
      None
    }
  }
}
