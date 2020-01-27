use crate::rng::Rng;

pub struct Stack< T > {
  default_val : T,
  data        : Vec< T >,
  size        : usize
}

impl< T: Clone + Copy > Stack< T > {
  pub fn new( default_val : T ) -> Stack< T > {
    Stack {
      default_val: default_val
    , data: vec![ default_val; 1024 ]
    , size: 0
    }
  }

  pub fn clear( &mut self ) {
    self.size = 0;
  }

  pub fn shuffle( &mut self, rng : &mut Rng ) {
    for i in 0..self.size {
      let new_i = rng.next_in_range( 0, self.size );
      self.swap( i, new_i );
    }
  }

  pub fn push( &mut self, v : T ) {
    if self.size >= self.data.len( ) {
      self.data.reserve( self.size );
      for _i in 0..self.size {
        self.data.push( self.default_val );
      }
    }
    self.data[ self.size ] = v;
    self.size += 1;
  }

  pub fn pop( &mut self ) -> Option< T > {
    if self.size == 0 {
      None
    } else {
      self.size -= 1;
      Some( self.data[ self.size ] )
    }
  }

  pub fn len( &self ) -> usize {
    self.size
  }

  pub fn swap( &mut self, i : usize, j : usize ) {
    self.data.swap( i, j );
  }
}

// /// A Stack
// pub struct DefaultStack< T > {
//   default_val : T,
//   data        : Vec< T >
// }

// impl< T: Clone + Copy > DefaultStack< T > {
//   /// Constructs a new Stack with the given capacity
//   /// The `default_val` is used to fill the stack, which is necessary
//   ///   as the whole array is allocated upon stack construction
//   pub fn new( initial_capacity : usize, default_val : T ) -> DefaultStack< T > {
//     DefaultStack { default_val, data: Vec::with_capacity( initial_capacity ) }
//   }

//   // Constructs a new Stack with the given capacity, and already one default value
//   // pushed unto the stack
//   pub fn new1( initial_capacity : usize, default_val : T ) -> DefaultStack< T > {
//     let mut s = DefaultStack::new( initial_capacity, default_val );
//     s.push( default_val );
//     s
//   }

//   /// Pushes an element to the stack
//   /// Does *not* check capacity violations
//   pub fn push( &mut self, v : T ) {
//     self.data.push( v );
//   }

//   /// Only pop if there are *more* than 1 object on the stack
//   ///
//   /// For raytracing: Useful to always keep the air material on the stack
//   pub fn pop_until1( &mut self ) -> Option< T > {
//     if self.data.len( ) > 1 {
//       self.data.pop( )
//     } else {
//       None
//     }
//   }

//   /// Returns a reference to the top of the stack
//   /// Returns Nothing if no such element
//   pub fn top< 'a >( &'a self ) -> Option< &'a T > {
//     let len = self.data.len( );
//     if len > 0 {
//       Some( &self.data[ len - 1 ] )
//     } else {
//       None
//     }
//   }
// }
