import { RaytraceTarget } from './offscreen_target';
import { RenderManager, BlockRenderer, RenderConfig } from './render_manager';
import { Rect4 } from 'rect4';
import { Elm } from './Main.elm';

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
      if ( ( ev.buttons & 1 ) !== 0 ) { // left mouse down
        let dx = ev.x - prevDownX;
        let dy = ev.y - prevDownY;
        prevDownX = ev.x;
        prevDownY = ev.y;
        this._xOff = this._xOff + dx;
        this._yOff = this._yOff + dy;
        this.reclamp( );
        this.render( );
      }
    } );
  }

  public recenter( ): void {
    if ( this._target ) {
      this._xOff = Math.round( ( this._canvas.width - this._target.width ) / 2 );
      this._yOff = Math.round( ( this._canvas.height - this._target.height ) / 2 );
      this.render( );
    }
  }

  public reclamp( ): void {
    let target = <RaytraceTarget> this._target;
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
    this.render( );
  }

  public updateTarget( target : RaytraceTarget ): void {
    this._target = target;
    this._xOff = Math.round( ( this._canvas.width - target.width ) / 2 );
    this._yOff = Math.round( ( this._canvas.height - target.height ) / 2 );
    this.render( );
  }

  public mark( rect : Rect4 ): void {
    this._markedRegions.push( rect );
    this.render( );
  }

  public unmark( rect : Rect4 ): void {
    for ( let i = 0; i < this._markedRegions.length; i++ ) {
      // Only use '===' here because I know they'll always be the same reference here
      // Otherwise deep checking should be used
      if ( this._markedRegions[ i ] === rect ) {
        this._markedRegions.splice( i, 1 );
        this.render( );
        return;
      }
    }
  }

  public unmarkAll( ): void {
    this._markedRegions = [];
    this.render( );
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

class WorkerRenderer implements BlockRenderer {
  private readonly _worker : Worker;

  private          _jobNr : number;
  private readonly _jobs  : Map< number, ( _ : any ) => any >;

  public constructor( wasmModule : any ) {
    this._worker = new Worker( 'worker.js' );
    this._worker.postMessage( { type: 'init', module: wasmModule } );
    this._worker.addEventListener( 'message', ev => {
      let msg = ev.data;
      let fResolve = <any> this._jobs.get( msg.jobId );
      this._jobs.delete( msg.jobId );
      fResolve( msg.data );
    } );

    this._jobNr = 0;
    this._jobs  = new Map( );
  }

  public setScene( width : number, height : number ): Promise< void > {
    return new Promise( ( fResolve, fReject ) => {
      let jobId = this._jobNr;
      this._jobs.set( jobId, fResolve );
      this._jobNr++;

      this._worker.postMessage( { type: 'call_setup_scene', jobId, width, height } );
    } );
  }

  public renderBlock( x : number, y : number, width : number, height : number, antiAlias : number ): Promise< Uint8Array > {
    return new Promise( ( fResolve, fReject ) => {
      let jobId = this._jobNr;
      this._jobs.set( jobId, fResolve );
      this._jobNr++;

      this._worker.postMessage( { type: 'call_compute', jobId, x, y, width, height, antiAlias } );
    } );
  }

  public terminate( ): void {
    this._worker.terminate( );
  }
}

(<any>WebAssembly).compileStreaming(fetch('pkg/index_bg.wasm')).then( compiledMod => {
  
  let blockSize = 128;
  let width     = 800;
  let height    = 500;
  let antiAlias = 1;
  let numCores  = 1;
  let isBandless = false;

  let manager = new RenderManager( ( ) => new WorkerRenderer( compiledMod ), numCores );
  //let manager = new RenderManager( ( ) => new SimpleRenderer( ) );
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
    console.log( 'duration', ev.duration );
  } );
  manager.start( new RenderConfig( blockSize, width, height, antiAlias, isBandless ) );

  canvasElem.updateTarget( <RaytraceTarget> manager.target );

  timeout( 0 ).then( ( ) => {
    canvas.height = document.body.clientHeight;
    canvas.width = document.body.clientWidth - 250 / (3 / 4);
    canvasElem.recenter( );
  } );
  canvas.height = document.body.clientHeight;
  canvas.width = document.body.clientWidth - 250 / (3 / 4);

  window.addEventListener( 'resize', ev => {
    canvas.height = document.body.clientHeight;
    canvas.width = document.body.clientWidth - 250 / (3 / 4);
    canvasElem.reclamp( );
  } );

  let settingsPanel = document.getElementById( 'sidepanel' );
  const app = Elm.Main.init( { node: settingsPanel } );
  app.ports.setAntiAlias.subscribe( aa => {
    antiAlias = aa;
    canvasElem.unmarkAll( );
    manager.start( new RenderConfig( blockSize, width, height, antiAlias, isBandless ) );
    canvasElem.updateTarget( <RaytraceTarget> manager.target );
  });
  app.ports.setBlockSize.subscribe( bs => {
    blockSize = bs;
    canvasElem.unmarkAll( );
    manager.start( new RenderConfig( blockSize, width, height, antiAlias, isBandless ) );
    canvasElem.updateTarget( <RaytraceTarget> manager.target );
  } );
  app.ports.setBanding.subscribe( b => {
    isBandless = b;
    ( < RaytraceTarget > manager.target ).enableBandless( b );
    canvasElem.render( );
  } );
  app.ports.setNumCores.subscribe( c => {
    numCores = c;
    manager.setNumCores( numCores );
  } );
} );