import { Observable, XObservable } from '@s/event/observable';
import { clamp } from '@s/math';

// This file contains the on-screen target to which the raytraced pixels
// are pushed. There are two classes that contribute to this:
// * RenderTarget: An off-screen canvas target of the viewport's size
// * CanvasElement: An on-screen canvas of screen size, which renders the
//                  off-screen target to the screen

// An off-screen target at the size of the viewport
// Whenever the raytraced buffer (as an RGBA array) is updated, it should be
//   pushed to this target. This `RenderTarget` converts that RGBA array to
//   an (off-screen) `HTMLCanvasElement`. Any update-listeners will be notified
//   of this update. (The `CanvasElement` class below is one such listener)
export class RenderTarget {
  // Viewport width
  public readonly width  : number;
  // Viewport height
  public readonly height : number;
  // The off-screen target
  public readonly target : HTMLCanvasElement;

  private readonly _ctx      : CanvasRenderingContext2D;
  // An interface to push the updated pixels to
  private readonly _imgData  : ImageData;
  // Used to notify listeners upon the target changing
  private readonly _onUpdate : XObservable< void >;

  // Constructs an off-screen render target with the given viewport size
  public constructor( width : number, height : number ) {
    this.width         = width;
    this.height        = height;
    this.target        = document.createElement( 'canvas' );
    this.target.width  = width;
    this.target.height = height;
    this._ctx          = <CanvasRenderingContext2D> this.target.getContext( '2d' );
    this._imgData      = this._ctx.createImageData( width, height );
    this._onUpdate     = new XObservable( );
  }

  // Listen to this `Observable` to be notified of any changes to the render
  // target
  public onUpdate( ): Observable< void > {
    return this._onUpdate.observable;
  }

  // Update the pixels in the render target
  // `pixels` must be an RGBA buffer of byte size `width * height * 4`
  public update( pixels : Uint8Array ): void {
    this._imgData.data.set( pixels );
    this._ctx.putImageData( this._imgData, 0, 0 );
    this._onUpdate.next( );
  }
}

// Manages an on-screen `HTMLCanvasElement`
// Whenever the assigned `RenderTarget` is updated, this element is rerendered
// It also allows dragging the render target across the screen
export class CanvasElement {
  // The on-screen canvas
  private readonly _canvas : HTMLCanvasElement;
  private readonly _ctx    : CanvasRenderingContext2D;
  // The target to which pixel-updates are pushed
  private          _target : RenderTarget;

  // The offset of the target within the on-screen canvas. Updates by dragging
  // the viewport around
  private _xOff : number;
  private _yOff : number;

  // Constructs a new manager for the provided canvas, which listens for updates
  // to the provided `RenderTarget`.
  public constructor( canvas : HTMLCanvasElement, target : RenderTarget ) {
    this._canvas = canvas;
    this._ctx    = <CanvasRenderingContext2D> canvas.getContext( '2d' );
    this._target = target;

    this._xOff = 0;
    this._yOff = 0;

    // Code for dragging the target across the screen
    let prevDownX = 0;
    let prevDownY = 0;

    this._canvas.addEventListener( 'mousedown', ev => { prevDownX = ev.x; prevDownY = ev.y; } );
    this._canvas.addEventListener( 'mousemove', ev => {
      if ( ( ev.buttons & 1 ) !== 0 ) { // left mouse down
        let dx = ev.x - prevDownX;
        let dy = ev.y - prevDownY;
        prevDownX = ev.x;
        prevDownY = ev.y;
        this._xOff = this._xOff + dx;
        this._yOff = this._yOff + dy;
        this.reclamp( );
      }
    } );

    // Listen for updates
    target.onUpdate( ).subscribe( ( ) => { this._render( ); } );
  }

  // Recenters the target within the screen
  public recenter( ): void {
    if ( this._target ) {
      this._xOff = Math.round( ( this._canvas.width - this._target.width ) / 2 );
      this._yOff = Math.round( ( this._canvas.height - this._target.height ) / 2 );
      this._render( );
    }
  }

  // Reclamps the target within the screen.
  // If the viewport is smaller than the screen, its bounds the viewport within
  //   the screen. If the viewport is larger than the screen, a part of the
  //   viewport fully occupies the screen.
  public reclamp( ): void {
    let target = <RenderTarget> this._target;
    if ( target.width < this._canvas.width ) {
      this._xOff = clamp( this._xOff, 0, this._canvas.width - target.width );
    } else {
      this._xOff = clamp( this._xOff, this._canvas.width - target.width, 0 );
    }
    if ( target.height < this._canvas.height ) {
      this._yOff = clamp( this._yOff, 0, this._canvas.height - target.height );
    } else {
      this._yOff = clamp( this._yOff, this._canvas.height - target.height, 0 );
    }
    this._render( );
  }

  // Render the contents of the render target to the on-screen target
  private _render( ): void {
    this._ctx.fillStyle = '#3e3e3e';
    this._ctx.fillRect( 0, 0, this._canvas.width, this._canvas.height );
    this._ctx.drawImage( this._target.target, this._xOff, this._yOff );
  }
}
