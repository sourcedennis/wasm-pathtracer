import { EmptyPromise } from '@s/event/promise';
import { Camera }       from '@s/graphics/camera';
import { Vec2 }         from '@s/math/vec2';
import { Triangles }    from '@s/graphics/triangles';
import { Msg, MsgC2WInit, MsgC2WUpdateCamera, MsgC2WUpdateParams
       , MsgC2WUpdateScene, MsgC2WCompute, MsgC2WStoreMesh, MsgC2WStoreTexture, MsgC2WRebuildBVH, MsgC2WDisableBVH } from '@s/worker_messages';
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
  private readonly _workers      : MsgController;
  private          _queue        : JobQueue;
  private readonly _buffer8      : Uint8Array;

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
    let workers = new Array( numWorkers );
    for ( let i = 0; i < numWorkers; i++ ) {
      workers[ i ] = new Worker( 'worker.js' );
    }
    this._workers = new MsgController( workers );

    this._camera           = camera;
    this._hasUpdatedCamera = false;

    let buffer    = new SharedArrayBuffer( width * height * 4 );
    this._buffer8 = new Uint8Array( buffer );


    let onInit = new EmptyPromise< void >( );
    let numInitDone  = 0;
    this._queue      = new JobQueue( );

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

    let initDone = Promise.all( this._workers.awaitAll( 'init_done' ) );
    this._queue.add( ( ) => initDone );

    // This initialises the workers, and listens for their messages
    for ( let i = 0; i < numWorkers; i++ ) {
      let initMsg : MsgC2WInit = { type: 'init', mod, sceneId, pixels: bins[ i ], buffer, width, height, renderType, rayDepth, camera };
      workers[ i ].postMessage( initMsg );
    }
  }

  // See `Raytracer#render()`
  public render( ): Promise< Uint8Array > {
    return this._queue.add( ( ) => {
      if ( this._hasUpdatedCamera ) {
        let cameraMsg : MsgC2WUpdateCamera = { type: 'update_camera', camera: this._camera };
        this._workers.send( cameraMsg );
        this._hasUpdatedCamera = false;
      }

      let computeMsg : MsgC2WCompute = { type: 'compute' };
      this._workers.send( computeMsg );

      return Promise.all( this._workers.awaitAll( 'compute_done' ) )
        .then( ( ) => this._buffer8 );
    } );
  }

  // See `Raytracer#updateScene()`
  public updateScene( sceneId : number ): void {
    let msg: MsgC2WUpdateScene = { type: 'update_scene', sceneId };
    this._queue.add( ( ) => {
      this._workers.send( msg );
      return Promise.all( this._workers.awaitAll( 'update_scene_done' ) );
    } );
  }

  // See `Raytracer#destroy()`
  public destroy( ): void {
    this._workers.destroy( );
  }

  // See `Raytracer#updateParams()`
  public updateParams( renderType : number, maxRayDepth : number ): void {
    let msg : MsgC2WUpdateParams = { type: 'update_params', renderType, maxRayDepth };
    this._workers.send( msg );
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
    this._queue.add( ( ) => {
      this._workers.send( msg );
      return Promise.all( this._workers.awaitAll( 'mesh_done' ) );
    } );
  }

  // See `Raytracer#storeTexture()`
  public storeTexture( id : number, texture : Texture ): void {
    let msg : MsgC2WStoreTexture = { type: 'store_texture', id, texture };
    this._queue.add( ( ) => {
      this._workers.send( msg );
      return Promise.all( this._workers.awaitAll( 'texture_done' ) );
    } );
  }

  // See `Raytracer#rebuildBVH()`
  public rebuildBVH( numBins : number ): Promise< number > {
    let duration = Number.MAX_SAFE_INTEGER;
    this._queue.add( ( ) => {
      let startTime = Date.now( );
      let msg : MsgC2WRebuildBVH = { type: 'rebuild_bvh', numBins };
      this._workers.send1( msg );
      return this._workers.await1( 'bvh_done' ).then( ( ) => {
        console.log( 'first done' );
        duration = Date.now( ) - startTime;
      } );
    } );
    return this._queue.add( ( ) => {
      console.log( 'start all' );
      let msg : MsgC2WRebuildBVH = { type: 'rebuild_bvh', numBins };
      this._workers.send( msg );
      return Promise.all( this._workers.awaitAll( 'bvh_done' ) )
        .then( ( ) => duration );
    } );
  }

  // See `Raytracer#disableBVH()`
  public disableBVH( ): void {
    let msg : MsgC2WDisableBVH = { type: 'disable_bvh' };
    this._workers.send( msg );
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

class WorkerDesc {
  worker : Worker;
  queue  : Map< string, [EmptyPromise<any>] >;

  public constructor( worker : Worker ) {
    this.worker = worker;
    this.queue  = new Map( );
  }
}

class MsgController {
  private readonly _workers : WorkerDesc[];

  public constructor( workers : Worker[] ) {
    this._workers = new Array( workers.length );
    for ( let i = 0; i < workers.length; i++ ) {
      this._workers[ i ] = new WorkerDesc( workers[ i ] );
      this._workers[ i ].worker.addEventListener( 'message', ev => {
        let msg = ev.data;
        let q = this._workers[ i ].queue.get( msg.type );
        if ( q ) {
          if ( msg.type !== 'compute_done' ) {
            console.log( 'received', msg );
          }
          let h = <EmptyPromise<any>> q.shift( );
          h.fulfil( msg );
        } else {
          console.error( 'Discarded message', ev.type );
        }
      } );
    }
  }

  public send( msg : Msg ) {
    if ( msg.type !== 'compute' ) {
      console.log( 'sent', msg );
    }
    for ( let i = 0; i < this._workers.length; i++ ) {
      this._workers[ i ].worker.postMessage( msg );
    }
  }

  public send1( msg : Msg ) {
    if ( msg.type !== 'compute' ) {
      console.log( 'sent1', msg );
    }
    this._workers[ 0 ].worker.postMessage( msg );
  }

  public awaitAll< TMsg >( msgType : string ): Promise<TMsg|undefined>[] {
    let out: Promise< TMsg >[] = [];
    for ( let i = 0; i < this._workers.length; i++ ) {
      let w = this._workers[ i ];
      let newPromise = new EmptyPromise< TMsg >( );
      if ( w.queue.has( msgType ) ) {
        let q = <[EmptyPromise<any>]> w.queue.get( msgType );
        q.push( newPromise );
      } else {
        w.queue.set( msgType, [ newPromise ] )
      }
      out.push( newPromise.promise );
    }
    return out;
  }

  public await1< TMsg >( msgType : string ): Promise<TMsg|undefined> {
    let newPromise = new EmptyPromise< TMsg >( );
    let w = this._workers[ 0 ];
    if ( w.queue.has( msgType ) ) {
      let q = <[EmptyPromise<any>]> w.queue.get( msgType );
      q.push( newPromise );
    } else {
      w.queue.set( msgType, [ newPromise ] )
    }
    return newPromise.promise;
  }

  public destroy( ): void {
    for ( let i = 0; i < this._workers.length; i++ ) {
      this._workers[ i ].worker.terminate( );
    }
    for ( let w of this._workers ) {
      for ( let [x,ps] of w.queue ) {
        for ( let p of ps ) {
          p.fulfil( undefined );
        }
      }
    }
  }
}
