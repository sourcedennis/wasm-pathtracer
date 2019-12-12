import { EmptyPromise } from '@s/event/promise';
import { Camera }       from '@s/graphics/camera';
import { Vec2 }         from '@s/math/vec2';
import { Triangles }    from '@s/graphics/triangles';
import { Msg, MsgC2WInit, MsgC2WUpdateCamera, MsgC2WUpdateParams
       , MsgC2WUpdateScene, MsgC2WCompute, MsgC2WStoreMesh, MsgC2WStoreTexture, MsgC2WRebuildBVH } from '@s/worker_messages';
import { Raytracer }           from './index';
import { shuffle, divideOver } from '../util';
import { Texture }             from '@s/graphics/texture';

// A raytracer that uses WebWorkers to raytrace (semi-hardcoded) scenes.
//
// This is an interface over the interaction with the WebWorkers. Where each
// WebWorker runs an WASM module, compiled from the Rust source in `src` in the
// project root. This Rust code is the actual raytracer.
//
// Note that instances of this class send messages to the WebWorkers. The
//   implementation of these workers is defined in `src_ts/worker`.
//
// Warning: A `SharedArrayBuffer` is used, which is the only way of obtaining
//   reasonable speed. However, this feature is *disabled* in some browsers.
//   Its use is necessary, as it is the fastest way of sharing raw data between
//   workers; thus, this is used for the output pixel buffer.
//   See: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer
//   (It works in Google Chrome)
export class MulticoreRaytracer implements Raytracer {
  private readonly _width        : number;
  private readonly _height       : number;
  private readonly _workers      : Worker[];
  private readonly _onInitDone   : Promise< void >;
  private          _onRenderDone : EmptyPromise< Uint8Array > | undefined;
  private          _bvhPromise   : EmptyPromise< number > | undefined;
  // Number of workers that have raytraced all their assigned pixels for the
  //   current frame.
  private          _numDone           : number;
  private          _queue             : JobQueue;

  // True if the camera has updated since the last render tick
  // This boolean is used, such that the camera is not updated more than once
  //   before each render tick; as that would be redundant. Only the last camera
  //   update remains.
  private          _hasUpdatedCamera : boolean;
  private          _camera           : Camera;

  public constructor( // Viewport size
                      width      : number
                    , height     : number
                      // Scene ids uniquely identify hardcoded scenes. Only used
                      // to communicate it with Rust
                    , sceneId    : number
                      // The compiled WebAssembly module.
                      // *must* be the module obtained from the `src` directory
                    , mod        : WebAssembly.Module
                      // True if a depth-buffer is rendered. Diffuse otherwise
                    , renderType : number
                      // Maximum number of ray bounces
                    , rayDepth   : number
                      // Scene camera
                    , camera     : Camera
                      // The number of webworkers to spawn
                    , numWorkers : number ) {
    this._width   = width;
    this._height  = height;
    this._workers = new Array( numWorkers );

    this._camera           = camera;
    this._hasUpdatedCamera = false;

    let buffer  = new SharedArrayBuffer( width * height * 4 );
    let buffer8 = new Uint8Array( buffer );

    for ( let i = 0; i < numWorkers; i++ ) {
      this._workers[ i ] = new Worker( 'worker.js' );
    }

    let onInit = new EmptyPromise< void >( );
    let numInitDone  = 0;
    this._queue      = new JobQueue( );
    this._onInitDone = this._queue.add( ( ) => onInit.promise );
    this._numDone    = 0;

    // Divide the pixels in the viewport randomly over the workers
    // On average this equally divides the work =)
    let rays = new Array< Vec2 >( width * height );
    for ( let y = 0; y < height; y++ ) {
      for ( let x = 0; x < width; x++ ) {
        rays[ y * width + x ] = new Vec2( x, y );
      }
    }
    shuffle( rays );
    let bins = divideOver( rays, numWorkers );

    // This initialises the workers, and listens for their messages
    for ( let i = 0; i < numWorkers; i++ ) {
      this._workers[ i ].addEventListener( 'message', ev => {
        const typelessMsg : Msg = ev.data;

        if ( typelessMsg.type === 'init_done' ) {
          numInitDone++;
          if ( numInitDone === numWorkers ) {
            onInit.fulfil( );
          }
        } else if ( typelessMsg.type === 'compute_done' ) {
          this._numDone++;
          if ( this._numDone === numWorkers ) {
            ( <EmptyPromise< Uint8Array >> this._onRenderDone ).fulfil( buffer8 );
          }
        } else if ( typelessMsg.type === 'bvh_done' ) {
          this._numDone++;
          if ( this._numDone === numWorkers ) {
            ( <EmptyPromise< number >> this._bvhPromise ).fulfil( 0 );
          }
        }
      } );

      let initMsg : MsgC2WInit = { type: 'init', mod, sceneId, pixels: bins[ i ], buffer, width, height, renderType, rayDepth, camera };
      this._workers[ i ].postMessage( initMsg );
    }
  }

  // See `Raytracer#render()`
  public render( ): Promise< Uint8Array > {
    let job = ( ) => {
      let prevPromise = this._onInitDone;
      if ( this._onRenderDone ) {
        prevPromise = this._onRenderDone.promise.then( ( ) => { } );
      }
  
      return prevPromise.then( ( ) => {
        this._numDone = 0;
        this._onRenderDone = new EmptyPromise< Uint8Array >( );
  
        if ( this._hasUpdatedCamera ) {
          let cameraMsg : MsgC2WUpdateCamera = { type: 'update_camera', camera: this._camera };
          this._postMsg( cameraMsg );
          this._hasUpdatedCamera = false;
        }
  
        let computeMsg : MsgC2WCompute = { type: 'compute' };
        this._postMsg( computeMsg );
  
        return this._onRenderDone.promise;
      } );
    };
    return this._queue.add( job );
  }

  // See `Raytracer#updateScene()`
  public updateScene( sceneId : number ): void {
    let msg: MsgC2WUpdateScene = { type: 'update_scene', sceneId };
    this._postMsg( msg );
  }

  // See `Raytracer#destroy()`
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

  // See `Raytracer#updateParams()`
  public updateParams( renderType : number, maxRayDepth : number ): void {
    let msg : MsgC2WUpdateParams = { type: 'update_params', renderType, maxRayDepth };
    this._postMsg( msg );
  }

  // See `Raytracer#updateCamera()`
  public updateCamera( camera : Camera ): void {
    this._hasUpdatedCamera = true;
    this._camera = camera;
  }

  // See `Raytracer#updateViewport()`
  // public updateViewport( width : number, height : number ): void {
  //   let msg : MsgC2WUpdateParams = { type: 'update_viewport', width, height };
  //   this._postMsg( msg );
  // }

  // See `Raytracer#storeMesh()`
  public storeMesh( id : number, mesh : Triangles ): void {
    let msg : MsgC2WStoreMesh = { type: 'store_mesh', id, mesh };
    this._postMsg( msg );
  }

  // See `Raytracer#storeTexture()`
  public storeTexture( id : number, texture : Texture ): void {
    let msg : MsgC2WStoreTexture = { type: 'store_texture', id, texture };
    this._postMsg( msg );
  }

  // See `Raytracer#rebuildBVH()`
  public rebuildBVH( numBins : number ): Promise< number > {
    return this._queue.add( ( ) => {
      this._bvhPromise = new EmptyPromise( );
      this._numDone = 0;
      let startTime = Date.now( );
      let msg : MsgC2WRebuildBVH = { type: 'rebuild_bvh', numBins };
      this._postMsg( msg );
      return this._bvhPromise.promise.then( ( ) => Date.now( ) - startTime );
    } );
  }

  // Sends a message to all the workers
  private _postMsg( msg : Msg ): void {
    for ( let i = 0; i < this._workers.length; i++ ) {
      this._workers[ i ].postMessage( msg );
    }
  }
}

class JobQueue {
  private readonly _queue     : ( ( ) => Promise< void > )[];
  private          _isRunning : boolean;

  public constructor( ) {
    this._queue = [];
    this._isRunning = false;
  }

  public add< T >( f : ( ) => Promise< T > ): Promise< T > {
    if ( !this._isRunning ) {
      this._isRunning = true;
      let ep = new EmptyPromise< T >( );
      f( ).then( v => { this._next( ); ep.fulfil( v ); } );
      return ep.promise;
    } else {
      let ep = new EmptyPromise< T >( );
      this._queue.push( ( ) => f( ).then( v => { ep.fulfil( v ); } ) );
      return ep.promise;
    }
  }

  private _next( ): void {
    if ( this._queue.length > 0 ) {
      let job = < ( ) => Promise< void > > this._queue.shift( );
      job( ).then( ( ) => this._next( ) );
    } else {
      this._isRunning = false;
    }
  }
}
