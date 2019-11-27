import { Observable, XObservable } from './observable';
import { Elm } from './Main.elm';
import { parseObj, Triangles } from './obj_parser';
import { Vec3 } from './vec3';

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

// Meshes are loaded from external OBJ files
// but the scenes are hard-coded in the Rust application
// these identify meshes
enum MeshId {
  MESH_RABBIT
}

class Camera {
  public readonly location : Vec3;
  public readonly rotX     : number;
  public readonly rotY     : number;

  public constructor( location : Vec3, rotX : number, rotY : number ) {
    this.location = location;
    this.rotX = rotX;
    this.rotY = rotY;
  }
}

interface Renderer {
  render( ): Promise< Uint8Array >;
  destroy( ): void;
  updateParams( isDepth : boolean, maxRayDepth : number ): void;
  // It *first* rotates around the x-axis, and then the y-axis. And then translation is applied
  updateCamera( cam : Camera ): void;
}

class SingleRenderer implements Renderer {
  private readonly _mod : WebAssembly.Module;
  private readonly _ins : Promise< WebAssembly.Instance >;
  private readonly _width : number;
  private readonly _height : number;

  private _isDepth

  public constructor( width : number, height : number, mod : WebAssembly.Module, isDepth : boolean, rayDepth : number, camera : Camera ) {
    this._mod = mod;
    this._width = width;
    this._height = height;

    let importObject =
      { env: { abort: arg => console.log( 'abort' ) } };
      
    this._ins = WebAssembly.instantiate( mod, importObject ).then( ins => <any> ins ).then( ins => {
      // Pass stuff across WASM boundary
      ins.exports.init( width, height, isDepth, rayDepth, camera.location.x, camera.location.y, camera.location.z, camera.rotX, camera.rotY );
      let rayPtr = ins.exports.ray_store( width * height );
      let rays = new Uint32Array( ins.exports.memory.buffer, rayPtr, width * height * 2 );

      for ( let y = 0; y < height; y++ ) {
        for ( let x = 0; x < width; x++ ) {
          rays[ 2 * ( y * width + x ) + 0 ] = x;
          rays[ 2 * ( y * width + x ) + 1 ] = y;
        }
      }
      ins.exports.ray_store_done( );
      return ins;
    } );
  }

  public render( ): Promise< Uint8Array > {
    return this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.compute( );
      return new Uint8Array( exps.memory.buffer, exps.results( ), this._width * this._height * 4 );
    } );
  }

  public destroy( ): void { }

  public updateParams( isDepth : boolean, maxRayDepth : number ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.update_params( isDepth ? 1 : 0, maxRayDepth );
    } );
  }

  public updateCamera( cam : Camera ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.update_camera( cam.location.x, cam.location.y, cam.location.z, cam.rotX, cam.rotY );
    } );
  }
}

class MulticoreRenderer implements Renderer {
  private readonly _width        : number;
  private readonly _height       : number;
  private readonly _workers      : Worker[];
  //private readonly _buffer       : SharedArrayBuffer;
  private readonly _onInitDone   : Promise< void >;
  private          _onRenderDone : EmptyPromise< Uint8Array > | undefined;
  private          _numDone      : number;
  private          _hasUpdatedCamera : boolean;
  private          _camera           : Camera;

  public constructor( width : number, height : number, mod : WebAssembly.Module
                    , isDepth : boolean, rayDepth : number, camera : Camera, numWorkers : number ) {
    this._width   = width;
    this._height  = height;
    this._workers = new Array( numWorkers );
    let buffer  = new SharedArrayBuffer( width * height * 4 );
    let buffer8 = new Uint8Array( buffer );
    this._camera = camera;
    this._hasUpdatedCamera = false;
    for ( let i = 0; i < numWorkers; i++ ) {
      this._workers[ i ] = new Worker( 'worker.js' );
    }
    console.log( 'multirendering!' );

    let onInit = new EmptyPromise< void >( );
    let numInitDone = 0;
    this._onInitDone = onInit.promise;
    this._numDone = 0;

    let rays = new Array< Point2 >( width * height );
    for ( let y = 0; y < height; y++ ) {
      for ( let x = 0; x < width; x++ ) {
        rays[ y * width + x ] = new Point2( x, y );
      }
    }
    shuffle( rays );
    let bins = divideOver( rays, numWorkers );

    for ( let i = 0; i < numWorkers; i++ ) {
      this._workers[ i ].addEventListener( 'message', ev => {
        const msg = ev.data;

        if ( msg.type === 'init_done' ) {
          numInitDone++;
          if ( numInitDone === numWorkers ) {
            onInit.fulfil( );
          }
        } else if ( msg.type === 'compute_done' ) {
          this._numDone++;
          if ( this._numDone === numWorkers ) {
            ( <EmptyPromise< Uint8Array >> this._onRenderDone ).fulfil( buffer8 );
          }
        }
      } );
      this._workers[ i ].postMessage( { type: 'init', mod, pixels: bins[ i ], buffer, width, height, isDepth, rayDepth, camera } );
    }
  }

  public render( ): Promise< Uint8Array > {
    let prevPromise = this._onInitDone;
    if ( this._onRenderDone ) {
      prevPromise = this._onRenderDone.promise.then( ( ) => { } );
    }

    return prevPromise.then( ( ) => {
      this._numDone = 0;
      this._onRenderDone = new EmptyPromise< Uint8Array >( );

      if ( this._hasUpdatedCamera ) {
        for ( let i = 0; i < this._workers.length; i++ ) {
          this._workers[ i ].postMessage( { type: 'update_camera', camera: this._camera } );
        }
        this._hasUpdatedCamera = false;
      }

      for ( let i = 0; i < this._workers.length; i++ ) {
        this._workers[ i ].postMessage( { type: 'compute' } );
      }

      return this._onRenderDone.promise;
    } );
  }

  public destroy( ): void {
    for ( let i = 0; i < this._workers.length; i++ ) {
      this._workers[ i ].terminate( );
    }
    if ( this._numDone < this._workers.length ) {
      let dst = new Uint8Array( this._width * this._height * 4 );
      dst.fill( 255 );
      ( <EmptyPromise< Uint8Array >> this._onRenderDone ).fulfil( dst );
    }
  }

  public updateParams( isDepth : boolean, maxRayDepth : number ): void {
    for ( let i = 0; i < this._workers.length; i++ ) {
      this._workers[ i ].postMessage( { type: 'update_params', isDepth, maxRayDepth } );
    }
  }

  public updateCamera( camera : Camera ): void {
    this._hasUpdatedCamera = true;
    this._camera = camera;
  }
}

function divideOver< T >( x : T[], numBins: number ): T[][] {
  let dst = new Array< T[] >( numBins );
  let prevI = 0;
  for ( let i = 0; i < numBins; i++ ) {
    let size = ( x.length - prevI ) / ( numBins - i );
    dst[ i ] = x.slice( prevI, prevI + size );
    prevI += size;
  }
  return dst;
}

// class RenderTarget {
//   public readonly target : HTMLCanvasElement;

//   private readonly _instance : WebAssembly.Instance;
//   private readonly _ctx      : CanvasRenderingContext2D;
//   private readonly _imgData  : ImageData;
//   //private readonly _onDone   : XObservable< void >;
//   private          _isDepth  : boolean;
//   private          _maxReflect : number;

//   public constructor( instance : WebAssembly.Instance, width : number, height : number, isDepth : boolean ) {
//     this.target        = document.createElement( 'canvas' );
//     this.target.width  = width;
//     this.target.height = height;
//     this._instance     = instance;
//     this._ctx          = < CanvasRenderingContext2D > this.target.getContext( '2d' );
//     this._imgData      = this._ctx.createImageData( width, height );
//     //this._onDone       = new XObservable( );
//     this._isDepth      = isDepth;
//     this._maxReflect   = 1;
    
//     (<any>this._instance.exports).init( width, height, this._isDepth, this._maxReflect );
//     this._setupRays( );
//   }

//   public storeMesh( id : MeshId, triangles : Triangles ): void {
//     let idInt: number;
//     switch ( id ) {
//     case MeshId.MESH_RABBIT:
//       idInt = 0;
//       break;
//     default:
//       throw new Error( 'Invalid mesh' );
//     }

//     (<any> this._instance.exports ).allocate_mesh( idInt, triangles.vertices.length, triangles.normals.length );

//     let vPtr = (<any> this._instance.exports ).mesh_vertices( idInt );
//     let nPtr = (<any> this._instance.exports ).mesh_normals( idInt );

//     let dstVertices =
//       new Float32Array( (<any>this._instance.exports).memory.buffer, vPtr, triangles.vertices.length );
//     let dstNormals =
//       new Float32Array( (<any>this._instance.exports).memory.buffer, nPtr, triangles.normals.length );
    
//     dstVertices.set( triangles.vertices );
//     dstNormals.set( triangles.normals );

//     (<any> this._instance.exports ).notify_mesh_loaded( idInt );
//   }

//   public updateRenderType( isDepth : boolean ) {
//     this._isDepth = isDepth;
//     // TODO: Init again
//   }

//   public updateReflect( reflect : number ) {
//     this._maxReflect = reflect;
//     // TODO: Init again
//   }

//   public width( ): number {
//     return this.target.width;
//   }

//   public height( ): number {
//     return this.target.height;
//   }

//   // public onDone( ): Observable< void > {
//   //   return this._onDone.observable;
//   // }

//   public render( ): void {
//     let w = this.target.width;
//     let h = this.target.height;

//     (<any>this._instance.exports).compute( );

//     let pixels = new Uint8Array( (<any>this._instance.exports).memory.buffer, (<any>this._instance.exports).results( ), w * h * 4 );
//     this._imgData.data.set( pixels, 0 );
//     this._ctx.putImageData( this._imgData, 0, 0 );
//     //this._onDone.next( );
//     console.log( 'done' );
//   }

//   private _setupRays( ): void {
//     let w = this.target.width;
//     let h = this.target.height;
    
//     let rays = new Array< Point2 >( w * h );
//     for ( let y = 0; y < h; y++ ) {
//       for ( let x = 0; x < w; x++ ) {
//         rays[ y * w + x ] = new Point2( x, y );
//       }
//     }
//     shuffle( rays );

//     let rayDst = new Uint32Array((<any>this._instance.exports).memory.buffer, (<any>this._instance.exports).ray_store( w * h ), w * h * 2 );

//     for ( let i = 0; i < w * h; i++ ) {
//       rayDst[ i * 2 + 0 ] = rays[ i ].x;
//       rayDst[ i * 2 + 1 ] = rays[ i ].y;
//     }

//     (<any>this._instance.exports).ray_store_done( );    
//   }
// }

class RenderTarget {
  private readonly _canvas   : HTMLCanvasElement;
  private readonly _ctx      : CanvasRenderingContext2D;
  private readonly _imgData  : ImageData;
  private readonly _width    : number;
  private readonly _height   : number;
  private readonly _onUpdate : XObservable< void >;

  public constructor( width : number, height : number ) {
    this._width    = width;
    this._height   = height;
    this._canvas   = document.createElement( 'canvas' );
    this._canvas.width = width;
    this._canvas.height = height;
    this._ctx      = <CanvasRenderingContext2D> this._canvas.getContext( '2d' );
    this._imgData  = this._ctx.createImageData( width, height );
    this._onUpdate = new XObservable( );
  }

  public width( ): number {
    return this._width;
  }

  public height( ): number {
    return this._height;
  }

  public onUpdate( ): Observable< void > {
    return this._onUpdate.observable;
  }

  public update( pixels : Uint8Array ): void {
    this._imgData.data.set( pixels );
    this._ctx.putImageData( this._imgData, 0, 0 );
    this._onUpdate.next( );
  }

  public target( ): HTMLCanvasElement {
    return this._canvas;
  }
}

class CanvasElement {
  private readonly _canvas : HTMLCanvasElement;
  private readonly _ctx    : CanvasRenderingContext2D;
  private          _target : RenderTarget;

  private _xOff : number;
  private _yOff : number;

  public constructor( canvas : HTMLCanvasElement, target : RenderTarget ) {
    this._canvas = canvas;
    this._ctx    = <CanvasRenderingContext2D> canvas.getContext( '2d' );
    this._target = target;

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
      }
    } );

    target.onUpdate( ).subscribe( ( ) => { this._render( ); } );
  }

  public recenter( ): void {
    if ( this._target ) {
      this._xOff = Math.round( ( this._canvas.width - this._target.width( ) ) / 2 );
      this._yOff = Math.round( ( this._canvas.height - this._target.height( ) ) / 2 );
      this._render( );
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
    this._render( );
  }

  private _render( ): void {
    this._ctx.fillStyle = '#3e3e3e';
    this._ctx.fillRect( 0, 0, this._canvas.width, this._canvas.height );

    if ( this._target ) {
      this._renderGrid( );

      this._ctx.drawImage( this._target.target( ), this._xOff, this._yOff );
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
        this._ctx.fillRect( x * cellSize + this._xOff, y * cellSize + this._yOff
          , Math.min( gridWidth - x * cellSize, cellSize ), Math.min( gridHeight - y * cellSize, cellSize ) );
      }
    }
  }
}

class Runner {
  private _isTerminated : boolean;

  public constructor( f: ( ) => Promise< void > ) {
    this._isTerminated = false;
    go( this );

    function go( self ) {
      f( ).then( ( ) => {
        if ( !self._isTerminated ) {
          setTimeout( ( ) => go( self ), 0 );
        }
      } );
    }
  }

  public terminate( ): void {
    this._isTerminated = true;
  }
}

class Measurement {
  public time : number;
  public measurement : number;

  public constructor( time : number, measurement : number ) {
    this.time = time;
    this.measurement = measurement;
  }
}

class FpsTracker {
  private readonly _measurements : Measurement[];
  private _index : number;
  private _numMeasurements : number;

  public constructor( ) {
    this._measurements = [];
    this._index = 0;
    this._numMeasurements = 0;
  }

  public add( time : number, measurement : number ) {
    while ( this._numMeasurements > 0 && this._measurements[ this._index ].time + 1000 < time ) {
      this._index = ( this._index + 1 ) % this._measurements.length;
      this._numMeasurements--;
    }

    if ( this._numMeasurements < this._measurements.length ) {
      let nextIndex = ( this._index + this._numMeasurements ) % this._measurements.length;
      this._measurements[ nextIndex ].time = time;
      this._measurements[ nextIndex ].measurement = measurement;
      this._numMeasurements++;
    } else {
      this._index = 0;
      this._numMeasurements++;
      this._measurements.push( new Measurement( time, measurement ) );
    }
  }

  public clear( ) {
    this._numMeasurements = 0;
  }

  public low( ): number {
    let l = Number.POSITIVE_INFINITY;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      l = Math.min( l, this._measurements[ ( this._index + i ) % this._measurements.length ].measurement );
    }
    return l;
  }

  public high( ): number {
    let h = 0;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      h = Math.max( h, this._measurements[ ( this._index + i ) % this._measurements.length ].measurement );
    }
    return h;
  }

  public avg( ): number {
    let sum = 0;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      sum += this._measurements[ ( this._index + i ) % this._measurements.length ].measurement;
    }
    return Math.round( sum / this._numMeasurements );
  }
}

function keyTicker( keys : Set< number > ): Observable< [ number, number ] > {
  return new Observable< [ number, number ] >( observer => {
    let downKeys = new Map< number, any >( );
    let lastTickTime = 0;

    window.addEventListener( 'keydown', ev => {
      if ( keys.has( ev.keyCode ) ) {
        if ( !downKeys.has( ev.keyCode ) ) {
          let ival = setInterval( ( ) => tick( ), 10 );
          downKeys.set( ev.keyCode, ival );
          observer.next( [ ev.keyCode, 1 ] );
          lastTickTime = Date.now( );
  
          function tick( ) {
            let currTime = Date.now( );
            let numTicks = Math.floor( ( currTime - lastTickTime ) / 10 );
            lastTickTime += numTicks * 10;
            observer.next( [ ev.keyCode, numTicks ] );
          }
        }

        ev.preventDefault( );
      }
    } );
    window.addEventListener( 'keyup', ev => {
      if ( keys.has( ev.keyCode ) ) {
        if ( downKeys.has( ev.keyCode ) ) {
          let ival = downKeys.get( ev.keyCode );
          clearInterval( ival );
          downKeys.delete( ev.keyCode );
        }
        ev.preventDefault( );
      }
    } );
  } );
}

class CameraController {
  private readonly _onUpdate : XObservable< Camera >;
  private _camera : Camera;

  public constructor( camera : Camera ) {
    this._onUpdate = new XObservable( );
    this._camera = this._cloneCamera( camera );

    let keys = new Set( [ 87, 68, 83, 65, 37, 38, 39, 40, 33, 34 ] );

    keyTicker( keys ).subscribe( ( [ code, count ] ) => {
      let translation: Vec3 | null = null;
      switch ( code ) {
      case 87: // W
        translation = new Vec3( 0, 0, 0.03 * count );
        break;
      case 68: // D
        translation = new Vec3( 0.03 * count, 0, 0 );
        break;
      case 83: // S
        translation = new Vec3( 0, 0, -0.03 * count );
        break;
      case 65: // A
        translation = new Vec3( -0.03 * count, 0, 0 );
        break;
      case 37: // left
        this.rotY -= 0.001 * count * Math.PI;
        break;
      case 38: // up
        this.rotX -= 0.001 * count * Math.PI;
        break;
      case 39: // right
        this.rotY += 0.001 * count * Math.PI;
        break;
      case 40: // down
        this.rotX += 0.001 * count * Math.PI;
        break;
      case 33: // page up
        translation = new Vec3( 0, 0.03 * count, 0 );
        break;
      case 34: // page down
        translation = new Vec3( 0, -0.03 * count, 0 );
        break;
      }

      if ( translation != null ) {
        translation = translation.rotY( this.rotY );
        let c = this._camera;
        this._camera = new Camera( c.location.add( translation ), c.rotX, c.rotY );
        this._onUpdate.next( this._camera );
      }
    } );
  }

  public onUpdate( ): Observable< Camera > { return this._onUpdate.observable; }

  public get x( ): number { return this._camera.location.x; }
  public get y( ): number { return this._camera.location.y; }
  public get z( ): number { return this._camera.location.z; }
  public get rotX( ): number { return this._camera.rotX; }
  public get rotY( ): number { return this._camera.rotY; }

  public set( c : Camera ) {
    this._camera = this._cloneCamera( c );
    this._onUpdate.next( this._camera );
  }

  public set x( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setX( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }
  
  public set y( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setY( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set z( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setZ( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set rotX( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location, v, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set rotY( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location, c.rotX, v );
    this._onUpdate.next( this._camera );
  }

  private _cloneCamera( c : Camera ): Camera {
    return new Camera( c.location, c.rotX, c.rotY );
  }
}

document.addEventListener( 'DOMContentLoaded', ev => {
  const width  = 512;
  const height = 512;

  const canvas  = document.getElementsByTagName( 'canvas' )[ 0 ];

  // Settings
  let isRunning        = true;
  let isMulticore      = false;
  let rayDepth         = 1;
  let isRenderingDepth = false; // depth-map vs color
  let camera           = new Camera( new Vec3( 0.0, 0.0, -1.0 ), 0, 0 );
  let cameraController = new CameraController( camera );

  (<any>WebAssembly).compileStreaming(fetch('pkg/index_bg.wasm'), { } ).then( compiledMod => {
    //const instance = compiledMod.instance;
    let target = new RenderTarget( width, height );
    let canvasElem = new CanvasElement( canvas, target );
    //let renderer = new SingleRenderer( width, height, compiledMod );
    let renderer: Renderer = new SingleRenderer( width, height, compiledMod, isRenderingDepth, rayDepth, camera );
    let fpsTracker = new FpsTracker( );

    cameraController.onUpdate( ).subscribe( c => {
      camera = c;
      renderer.updateCamera( c );
    } );
    
    onResize( );
    setTimeout( ( ) => { onResize( ); canvasElem.recenter( ); }, 0 );
    window.addEventListener( 'resize', ev => onResize( ) );

    let runner = new Runner( ( ) => render( ) );

    let settingsPanel = document.getElementById( 'sidepanel' );
    const app = Elm.Main.init( { node: settingsPanel } );
    app.ports.updateRenderType.subscribe( t => {
      isRenderingDepth = ( t == 1 );
      renderer.updateParams( isRenderingDepth, rayDepth );
      fpsTracker.clear( );
    } );
    app.ports.updateReflectionDepth.subscribe( d => {
      rayDepth = d;
      renderer.updateParams( isRenderingDepth, rayDepth );
      fpsTracker.clear( );
    } );
    app.ports.updateRunning.subscribe( r => {
      isRunning = r;
      if ( !isRunning ) {
        runner.terminate( );
      } else {
        fpsTracker.clear( );
        runner = new Runner( ( ) => render( ) );
      }
    } );
    app.ports.updateMulticore.subscribe( r => {
      isMulticore = r;
      renderer.destroy( );
      if ( isRunning ) {
        fpsTracker.clear( );
      }
      if ( isMulticore ) {
        renderer = new MulticoreRenderer( width, height, compiledMod, isRenderingDepth, rayDepth, camera, 8 );
      } else {
        renderer = new SingleRenderer( width, height, compiledMod, isRenderingDepth, rayDepth, camera );
      }
    } );

    function render( ): Promise< void > {
      let startTime = Date.now( );
      return renderer.render( ).then( res => {
        target.update( res );
        let currTime = Date.now( );
        fpsTracker.add( currTime, currTime - startTime );
        app.ports.updatePerformance.send( [ fpsTracker.avg( ), fpsTracker.low( ), fpsTracker.high( ) ] );
      } );
    }

    function onResize( ) {
      canvas.height = document.body.clientHeight;
      canvas.width = document.body.clientWidth - 250 / (3 / 4);
      canvasElem.reclamp( );
    }

    /*fetch( 'torus.obj' ).then( f => f.text( ) ).then( s => {
      let triangles = parseObj( s );
      target.storeMesh( MeshId.MESH_RABBIT, triangles );
      console.log( triangles );
    } );*/
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
