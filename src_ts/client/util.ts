
// Randomly shuffles an array in place
export function shuffle< T >( arr : T[] ): void {
  for ( let i = 0; i < arr.length; i++ ) {
    const newI  = Math.floor( Math.random( ) * arr.length );
    const tmp   = arr[ i ];
    arr[ i ]    = arr[ newI ];
    arr[ newI ] = tmp;
  }
}

// Divides elements in the provided array into (almost) equally-sized bins
// If the array-length is not divisible by the number of bins, some bins
// may be one element larger than the smaller bins.
export function divideOver< T >( x : T[], numBins: number ): T[][] {
  let dst = new Array< T[] >( numBins );
  let prevI = 0;
  for ( let i = 0; i < numBins; i++ ) {
    let size = Math.round( ( x.length - prevI ) / ( numBins - i ) );
    dst[ i ] = x.slice( prevI, prevI + size );
    prevI += size;
  }
  return dst;
}
