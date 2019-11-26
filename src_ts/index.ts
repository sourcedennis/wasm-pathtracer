import { Observable, XObservable } from './observable';
import { Elm } from './Main.elm';
import { parseObj, Triangles } from './obj_parser';

function clamp( x : number, minVal : number, maxVal : number ): number {
  return Math.min( maxVal, Math.max( x, minVal ) );
}

class Point2 {
  public readonly x : number;
  public readonly y : number;

  public constructor( x : number, y : number ) {
    this.x = x;
    this.y = y;
  }
}

class EmptyPromise< T > {
  public  readonly promise : Promise< T >;
  private          _fResolve: ( v: T | PromiseLike< T > ) => void;

  public constructor( ) {
    this.promise = new Promise( ( fResolve, fReject ) => {
      this._fResolve = fResolve;
    } );
  }

  public fulfil( v: T ): void {
    this._fResolve( v );
  }
}

class Job {
  private          _isDone : boolean;
  private readonly _onDone : EmptyPromise< void >;

  public constructor( f: ( ) => boolean ) {
    this._isDone = false;
    this._onDone = new EmptyPromise( );

    const fJob = ( ) => {
      if ( !this._isDone ) {
        this._isDone = !f( );
        setTimeout( ( ) => fJob( ), 0 );
      }
      if ( this._isDone ) {
        this._onDone.fulfil( );
      }
    }

    fJob( );
  }

  public onDone( ): Promise< void > {
    return this._onDone.promise;
  }

  public terminate( ): void {
    this._isDone = true;
  }
}

// Meshes are loaded from external OBJ files
// but the scenes are hard-coded in the Rust application
// these identify meshes
enum MeshId {
  MESH_RABBIT
}

class RenderTarget {
  public readonly target : HTMLCanvasElement;

  private readonly _instance : WebAssembly.Instance;
  private readonly _ctx      : CanvasRenderingContext2D;
  private readonly _imgData  : ImageData;
  private readonly _onUpdate : XObservable< number >;
  private          _isInit   : boolean;
  private          _pixels   : Uint8Array;
  private          _job      : Job | null;
  private          _isDepth  : boolean;
  private          _maxReflect : number;

  public constructor( instance : WebAssembly.Instance, width : number, height : number, isDepth : boolean ) {
    this.target        = document.createElement( 'canvas' );
    this.target.width  = width;
    this.target.height = height;
    this._instance     = instance;
    this._ctx          = < CanvasRenderingContext2D > this.target.getContext( '2d' );
    this._imgData      = this._ctx.createImageData( width, height );
    this._onUpdate     = new XObservable( );
    this._isInit       = false;
    this._job          = null;
    this._isDepth      = isDepth;
    this._maxReflect   = 1;
  }

  public storeMesh( id : MeshId, triangles : Triangles ): void {
    let idInt: number;
    switch ( id ) {
    case MeshId.MESH_RABBIT:
      idInt = 0;
      break;
    default:
      throw new Error( 'Invalid mesh' );
    }

    (<any> this._instance.exports ).allocate_mesh( idInt, triangles.vertices.length, triangles.normals.length );

    let vPtr = (<any> this._instance.exports ).mesh_vertices( idInt );
    let nPtr = (<any> this._instance.exports ).mesh_normals( idInt );

    let dstVertices =
      new Float32Array( (<any>this._instance.exports).memory.buffer, vPtr, triangles.vertices.length );
    let dstNormals =
      new Float32Array( (<any>this._instance.exports).memory.buffer, nPtr, triangles.normals.length );
    
    dstVertices.set( triangles.vertices );
    dstNormals.set( triangles.normals );

    let hasSceneUpdated = (<any> this._instance.exports ).notify_mesh_loaded( idInt );

    this._revalidateMemory( );

    if ( hasSceneUpdated ) {
      this.restart( );
    }
  }

  public updateRenderType( isDepth : boolean ) {
    this._isDepth = isDepth;
    this._isInit  = false;
    this.restart( );
  }

  public updateReflect( reflect : number ) {
    this._maxReflect = reflect;
    this._isInit     = false;
    this.restart( );
  }

  public width( ): number {
    return this.target.width;
  }

  public height( ): number {
    return this.target.height;
  }

  public onUpdate( ): Observable< number > {
    return this._onUpdate.observable;
  }

  private _revalidateMemory( ): void {
    if ( this._isInit ) {
      this._pixels = new Uint8Array( (<any>this._instance.exports).memory.buffer, (<any>this._instance.exports).results( ), this.target.width * this.target.height * 4 );
    }
  }

  public restart( ): void {
    let isRestarting = this._isInit;

    if ( this._job ) {
      this._job.terminate( );
      this._job = null;
      this._pixels.fill( 0 );
      this._onUpdate.next( 0 );
    }

    let numDone = 0;
    let w = this.target.width;
    let h = this.target.height;

    this._job = new Job( ( ) => {
      if ( !this._isInit ) {
        (<any>this._instance.exports).init( w, h, this._isDepth, this._maxReflect );
        this._setupRays( );
        this._pixels = new Uint8Array( (<any>this._instance.exports).memory.buffer, (<any>this._instance.exports).results( ), w * h * 4 );
        this._isInit = true;
        return true;
      } else if ( isRestarting ) {
        isRestarting = false;
        (<any>this._instance.exports).reset( );
        return true;
      } else if ( numDone < w * h ) {
        let numInPack = Math.min( w * h - numDone, 10000 );
        (<any>this._instance.exports).compute( numInPack );
        numDone += numInPack;

        return numDone < w * h;
      } else {
        return false;
      }
    } );
    let renderInterval = setInterval( ( ) => {
      this._imgData.data.set( this._pixels, 0 );
      this._ctx.putImageData( this._imgData, 0, 0 );
      this._onUpdate.next( numDone / ( w * h ) );
    }, 100 );
    this._job.onDone( ).then( ( ) => {
      clearInterval( renderInterval );
      this._imgData.data.set( this._pixels, 0 );
      this._ctx.putImageData( this._imgData, 0, 0 );
      this._onUpdate.next( 1 );
    } );

  }

  private _setupRays( ): void {
    let w = this.target.width;
    let h = this.target.height;
    
    let rays = new Array< Point2 >( w * h );
    for ( let y = 0; y < h; y++ ) {
      for ( let x = 0; x < w; x++ ) {
        rays[ y * w + x ] = new Point2( x, y );
      }
    }
    shuffle( rays );

    let rayDst = new Uint32Array((<any>this._instance.exports).memory.buffer, (<any>this._instance.exports).ray_store( ), w * h * 2 );

    for ( let i = 0; i < w * h; i++ ) {
      rayDst[ i * 2 + 0 ] = rays[ i ].x;
      rayDst[ i * 2 + 1 ] = rays[ i ].y;
    }
  }
}

class CanvasElement {
  private readonly _canvas : HTMLCanvasElement;
  private readonly _ctx    : CanvasRenderingContext2D;

  private          _target : RenderTarget | null;

  private _xOff : number;
  private _yOff : number;

  public constructor( canvas : HTMLCanvasElement ) {
    this._canvas = canvas;
    this._ctx    = <CanvasRenderingContext2D> canvas.getContext( '2d' );
    this._target = null;

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
      this._xOff = Math.round( ( this._canvas.width - this._target.width( ) ) / 2 );
      this._yOff = Math.round( ( this._canvas.height - this._target.height( ) ) / 2 );
      this.render( );
    }
  }

  public reclamp( ): void {
    let target = <RenderTarget> this._target;
    if ( target.width( ) < this._canvas.width ) {
      this._xOff = clamp( this._xOff, 0, this._canvas.width - target.width( ) );
    } else {
      this._xOff = clamp( this._xOff, this._canvas.width - target.width( ), 0 );
    }
    if ( target.height( ) < this._canvas.height ) {
      this._yOff = clamp( this._yOff, 0, this._canvas.height - target.height( ) );
    } else {
      this._yOff = clamp( this._yOff, this._canvas.height - target.height( ), 0 );
    }
    this.render( );
  }

  public updateTarget( target : RenderTarget ): void {
    this._target = target;
    this._xOff = Math.round( ( this._canvas.width - target.width( ) ) / 2 );
    this._yOff = Math.round( ( this._canvas.height - target.height( ) ) / 2 );
    this.render( );
  }

  public render( ): void {
    this._ctx.fillStyle = '#3e3e3e';
    this._ctx.fillRect( 0, 0, this._canvas.width, this._canvas.height );

    if ( this._target ) {
      this._renderGrid( );

      this._ctx.drawImage( this._target.target, this._xOff, this._yOff );
    }
  }

  private _renderGrid( ): void {
    let gridWidth  = ( <RenderTarget> this._target ).width( );
    let gridHeight = ( <RenderTarget> this._target ).height( );

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

document.addEventListener( 'DOMContentLoaded', ev => {
  const width  = 800;
  const height = 500;

  const canvas  = document.getElementsByTagName( 'canvas' )[ 0 ];

  (<any>WebAssembly).instantiateStreaming(fetch('pkg/index_bg.wasm'), { } ).then( compiledMod => {
    const instance = compiledMod.instance;
    let target = new RenderTarget( instance, width, height, false );
    target.restart( );

    let canvasElem = new CanvasElement( canvas );
    canvasElem.updateTarget( target );
    
    onResize( );
    setTimeout( ( ) => { onResize( ); canvasElem.recenter( ); }, 0 );
    window.addEventListener( 'resize', ev => onResize( ) );

    let settingsPanel = document.getElementById( 'sidepanel' );
    let startTime = Date.now( );
    const app = Elm.Main.init( { node: settingsPanel } );
    app.ports.callRestart.subscribe( ( ) => {
      startTime = Date.now( );
      target.restart( );
    } );
    app.ports.updateRenderType.subscribe( t => {
      target.updateRenderType( t === 1 );
      startTime = Date.now( );
    } );
    app.ports.updateReflectionDepth.subscribe( d => {
      target.updateReflect( d );
      startTime = Date.now( );
    } );

    target.onUpdate( ).subscribe( p => {
      canvasElem.render( );
      if ( p === 1 ) {
        app.ports.doneProgress.send( ( Date.now( ) - startTime ) / 1000 );
      } else {
        app.ports.updateProgress.send( Math.round( p * 100 ) );
      }
    } );

    function onResize( ) {
      canvas.height = document.body.clientHeight;
      canvas.width = document.body.clientWidth - 250 / (3 / 4);
      canvasElem.reclamp( );
    }

    fetch( 'torus.obj' ).then( f => f.text( ) ).then( s => {
      let triangles = parseObj( s );
      target.storeMesh( MeshId.MESH_RABBIT, triangles );
      console.log( triangles );
    } );
  } );

} );

function shuffle< T >( arr : T[] ): void {
  for ( let i = 0; i < arr.length; i++ ) {
    const newI  = Math.floor( Math.random( ) * arr.length );
    const tmp   = arr[ i ];
    arr[ i ]    = arr[ newI ];
    arr[ newI ] = tmp;
  }
}
