import { Observable, XObservable } from './observable';

// An off-screen render target
export class RaytraceTarget {
  public readonly width  : number;
  public readonly height : number;

  private readonly _canvas         : HTMLCanvasElement;
  private readonly _canvasBandless : HTMLCanvasElement;
  private readonly _ctx         : CanvasRenderingContext2D;
  private readonly _bandlessCtx : CanvasRenderingContext2D;
  
  private          _isBandless : boolean;

  public constructor( width : number, height : number, isBandless : boolean ) {
    this.width          = width;
    this.height         = height;

    this._canvas        = document.createElement( 'canvas' );
    this._canvas.width  = width;
    this._canvas.height = height;
    this._canvasBandless        = document.createElement( 'canvas' );
    this._canvasBandless.width  = width;
    this._canvasBandless.height = height;
    this._ctx = <CanvasRenderingContext2D> this._canvas.getContext( '2d' );
    this._bandlessCtx =
      <CanvasRenderingContext2D> this._canvasBandless.getContext( '2d' );
    this._isBandless = isBandless;
  }

  public enableBandless( b : boolean ): void {
    if ( this._isBandless && !b ) {
      this._isBandless = false;
    } else if ( !this._isBandless && b ) {
      this._isBandless = true;

      let w = this._canvas.width;
      let h = this._canvas.height;

      let imgData  = this._ctx.getImageData( 0, 0, w, h );
      let imgData2 = this._bandlessCtx.getImageData( 0, 0, w, h );
      let arr  = imgData.data;
      let arr2 = imgData2.data;

      for ( let i = 0; i < w * h; i++ ) {
        bandless( arr2, arr, i * 4 );
      }
      this._bandlessCtx.putImageData( imgData2, 0, 0 );
    }
  }

  public addRect( x : number, y : number, width : number, height : number, pixels : Uint8Array ): void {
    let imgData   = this._ctx.getImageData( x, y, width, height );

    // Update pixels
    let arr = imgData.data;
    for ( let y = 0; y < height; y++ ) {
      for ( let x = 0; x < width; x++ ) {
        // This random() noise stuff reduces banding
        // Green band the worst
        let red   = pixels[ ( y * width + x ) * 3 + 0 ];
        let green = pixels[ ( y * width + x ) * 3 + 1 ];
        let blue  = pixels[ ( y * width + x ) * 3 + 2 ];
        arr[ ( y * width + x ) * 4 + 0 ] = red;
        arr[ ( y * width + x ) * 4 + 1 ] = green;
        arr[ ( y * width + x ) * 4 + 2 ] = blue;
        arr[ ( y * width + x ) * 4 + 3 ] = 255;
      }
    }
    this._ctx.putImageData( imgData, x, y );

    if ( this._isBandless ) {
      let imgData2 = this._bandlessCtx.getImageData( x, y, width, height );
      let arr2 = imgData2.data;

      for ( let y = 0; y < height; y++ ) {
        for ( let x = 0; x < width; x++ ) {
          bandless( arr2, arr, ( y * width + x ) * 4 )
        }
      }
      this._bandlessCtx.putImageData( imgData2, x, y );
    }
  }

  public image( ): Readonly< HTMLCanvasElement > {
    if ( this._isBandless ) {
      return this._canvasBandless;
    } else {
      return this._canvas;
    }
  }
}

function bandless( dst : Uint8ClampedArray, src : Uint8ClampedArray, off : number ): void {
  let red   = src[ off + 0 ];
  let green = src[ off + 1 ];
  let blue  = src[ off + 2 ];

  let greenness = Math.max( 0, Math.min( 1, green / ( 0.5 * ( red + blue ) ) ) );
  let darkness  = 1 - ( 0.2126 * red / 255 + 0.7152 * green / 255 + 0.0722 * blue / 255 );

  let bandFactor = greenness * darkness;
  let scale = 0.08 * bandFactor;
  let mean  = 1 - scale / 2;
  dst[ off + 0 ] = Math.min( 255, ( Math.random( ) * scale + mean ) * red );
  dst[ off + 1 ] = Math.min( 255, ( Math.random( ) * scale + mean ) * green );
  dst[ off + 2 ] = Math.min( 255, ( Math.random( ) * scale + mean ) * blue );
  dst[ off + 3 ] = src[ off + 3 ];
}
