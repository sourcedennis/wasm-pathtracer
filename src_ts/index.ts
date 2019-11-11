const rust = import( '../pkg' );

import { RaytraceTarget } from './offscreen_target';
import { RenderManager, BlockRenderer, RenderConfig } from './render_manager';
import { Rect4 } from 'rect4';

class SimpleRenderer implements BlockRenderer {
  public setScene( vpWidth : number, vpHeight : number ): Promise< void > {
    return rust.then( r => {
      r.init( vpWidth, vpHeight );
    } );
  }

  public renderBlock( x : number, y : number, width : number, height : number ): Promise< Uint8Array > {
    return rust.then( r => timeout( 0 ).then( ( ) => r.compute_depths( x, y, width, height ) ) );
  }
}

function timeout( t : number ): Promise< void > {
  return new Promise( ( fResolve, fReject ) => {
    setTimeout( ( ) => fResolve( ), t );
  } );
}

function clamp( x : number, minVal : number, maxVal : number ): number {
  return Math.min( maxVal, Math.max( x, minVal ) );
}

class CanvasElement {
  private readonly _canvas : HTMLCanvasElement;
  private readonly _ctx    : CanvasRenderingContext2D;

  private          _target : RaytraceTarget | undefined;

  private          _markedRegions: Rect4[];

  private _xOff : number;
  private _yOff : number;

  public constructor( canvas : HTMLCanvasElement ) {
    this._canvas = canvas;
    this._ctx    = <CanvasRenderingContext2D> canvas.getContext( '2d' );
    this._target = undefined;
    this._markedRegions = [];

    this._xOff = 0;
    this._yOff = 0;


    let prevDownX = 0;
    let prevDownY = 0;

    this._canvas.addEventListener( 'mousedown', ev => { prevDownX = ev.x; prevDownY = ev.y; } );
    this._canvas.addEventListener( 'mousemove', ev => {
      let target = <RaytraceTarget> this._target;
      if ( ( ev.buttons & 1 ) !== 0 ) { // left mouse down
        let dx = ev.x - prevDownX;
        let dy = ev.y - prevDownY;
        prevDownX = ev.x;
        prevDownY = ev.y;
        this._xOff = clamp( this._xOff + dx, 0, this._canvas.width - target.width );
        this._yOff = clamp( this._yOff + dy, 0, this._canvas.height - target.height );
        this.render( );
      }
    } );
  }

  public updateTarget( target : RaytraceTarget ): void {
    this._target = target;
    this._xOff = ( this._canvas.width - target.width ) / 2;
    this._yOff = ( this._canvas.height - target.height ) / 2;
    this.render( );
  }

  public mark( rect : Rect4 ): void {
    this._markedRegions.push( rect );
    this.render( );
  }

  public unmark( rect : Rect4 ): void {
    for ( let i = 0; i < this._markedRegions.length; i++ ) {
      // Only use '===' here because I know they'll always be the same reference here
      if ( this._markedRegions[ i ] === rect ) {
        this._markedRegions.splice( i, 1 );
        this.render( );
        return;
      }
    }
  }

  public render( ): void {
    this._ctx.fillStyle = '#3e3e3e';
    this._ctx.fillRect( 0, 0, this._canvas.width, this._canvas.height );

    if ( this._target ) {
      this._renderGrid( );

      this._ctx.drawImage( this._target.image( ), this._xOff, this._yOff );

      for ( let r of this._markedRegions ) {
        this._ctx.strokeStyle = 'red';
        this._ctx.strokeRect( this._xOff + r.x + .5, this._yOff + r.y + .5, r.width - 1, r.height - 1 );
      }
    }
  }

  private _renderGrid( ): void {
    let gridWidth  = ( <RaytraceTarget> this._target ).width;
    let gridHeight = ( <RaytraceTarget> this._target ).height;

    let cellSize = 10;

    let numX = Math.floor( gridWidth / cellSize );
    let numY = Math.floor( gridHeight / cellSize );

    for ( let y = 0; y < numY; y++ ) {
      for ( let x = 0; x < numX; x++ ) {
        if ( x % 2 == y % 2 ) {
          this._ctx.fillStyle = '#808080';
        } else {
          this._ctx.fillStyle = '#A0A0A0';
        }
        this._ctx.fillRect( x * cellSize + this._xOff, y * cellSize + this._yOff, Math.min( gridWidth - x * cellSize, cellSize ), Math.min( gridHeight - y * cellSize, cellSize ) );
      }
    }
  }
}

rust
  .then(m => {
    let manager = new RenderManager( ( ) => new SimpleRenderer( ) );
    let canvas  = document.getElementsByTagName( 'canvas' )[ 0 ];
    let canvasElem = new CanvasElement( canvas );

    manager.on( 'queued' ).subscribe( ev => {
      canvasElem.mark( ev.rect );
    } );
    manager.on( 'unqueued' ).subscribe( ev => {
      canvasElem.unmark( ev.rect );
    } );
    manager.on( 'progress' ).subscribe( ev => {
      canvasElem.unmark( ev.rect );
    } );
    manager.on( 'done' ).subscribe( ev => {
      //timeout( 10 ).then( ( ) => alert( 'done!' ) );
    } );
    manager.start( new RenderConfig( 128, 800, 500 ) );

    canvasElem.updateTarget( <RaytraceTarget> manager.target );
  } )
  .catch(console.error);

