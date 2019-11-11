import { Observable, XObservable } from './observable';

// An off-screen render target
export class RaytraceTarget {
  public readonly width  : number;
  public readonly height : number;

  private readonly _canvas : HTMLCanvasElement;
  private readonly _ctx    : CanvasRenderingContext2D;

  public constructor( width : number, height : number ) {
    this.width          = width;
    this.height         = height;

    this._canvas        = document.createElement( 'canvas' );
    this._canvas.width  = width;
    this._canvas.height = height;
    this._ctx           = <CanvasRenderingContext2D> this._canvas.getContext( '2d' );
  }

  public addRect( x : number, y : number, width : number, height : number, pixels : Uint8Array ): void {
    let imgData   = this._ctx.getImageData( x, y, width, height );

    // Update pixels
    let arr = imgData.data;
    for ( let y = 0; y < height; y++ ) {
      for ( let x = 0; x < width; x++ ) {
        arr[ ( y * width + x ) * 4 + 0 ] = pixels[ ( y * width + x ) * 3 + 0 ];
        arr[ ( y * width + x ) * 4 + 1 ] = pixels[ ( y * width + x ) * 3 + 1 ];
        arr[ ( y * width + x ) * 4 + 2 ] = pixels[ ( y * width + x ) * 3 + 2 ];
        arr[ ( y * width + x ) * 4 + 3 ] = 255;
      }
    }

    this._ctx.putImageData( imgData, x, y );
  }

  public image( ): Readonly< HTMLCanvasElement > {
    return this._canvas;
  }
}
