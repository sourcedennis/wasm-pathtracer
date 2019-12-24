// An RGB texture
export class Texture {
  public readonly width  : number;
  public readonly height : number;
  public readonly data   : Uint8Array;

  public constructor( width : number, height : number, data : Uint8Array ) {
    this.width  = width;
    this.height = height;
    this.data   = data;
  }
}

// Whitted's checker seems 16x16 squares
// As the renderer takes the nearest neighbour anyway, just 16x16 pixels are
// necessary
export const CHECKER_RED_YELLOW: Texture =
  ( ( ) => {
    let width  = 16;
    let height = 16;
    let data = new Uint8Array( width * height * 3 );
    for ( let y = 0; y < height; y++ ) {
      for ( let x = 0; x < width; x++ ) {
        if ( x % 2 === y % 2 ) { // red
          data[ ( y * width + x ) * 3 + 0 ] = 0xFF;
          data[ ( y * width + x ) * 3 + 1 ] = 0x00;
          data[ ( y * width + x ) * 3 + 2 ] = 0x00;
        } else { // yellow
          data[ ( y * width + x ) * 3 + 0 ] = 0xFF;
          data[ ( y * width + x ) * 3 + 1 ] = 0xFF;
          data[ ( y * width + x ) * 3 + 2 ] = 0x00;
        }
      }
    }
    return new Texture( width, height, data );
  } )( );
