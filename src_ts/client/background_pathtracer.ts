import { Camera }    from '@s/graphics/camera';
import { Triangles } from '@s/graphics/triangles';
import { Texture }   from '@s/graphics/texture';
import { MsgC2WInit, MsgC2WPause, MsgC2WResume, MsgC2WUpdateViewport, MsgC2WUpdateScene
       , MsgC2WUpdateCamera, MsgC2WStoreMesh, MsgC2WStoreTexture, MsgC2WUpdateSettings, MsgC2WUpdateViewType } from '@s/worker_messages';
import { Observable, XObservable } from '@s/event/observable';

// Renders a scene on a background WebWorker.
// As these computations can take place externally (e.g. in other threads)
//   no guarantees exist on when this result is produced, but it is always
//   eventually produced once requested.
// Note that the actual path tracer is written in Rust (and defined in
// `src`). This is just the interface to a WebWorker running that module.
// The advantage of running it in the background is that the main thread
// UI interaction is not interrupted by long computations.
export class BackgroundPathTracer {
  // The continually updated result of the path tracer of the current scene.
  // scene. The result is a RGBA pixel buffer.
  // (Alpha is always 255, but this is convenient when pushing Canvas ImageData)
  public           buffer : Uint8Array;

  private readonly _worker   : Worker;
  private readonly _onUpdate : XObservable< void >;

  public constructor( // Viewport size
                      width      : number
                    , height     : number
                      // Scene ids uniquely identify hardcoded scenes. Only used
                      // to communicate it with Rust
                    , sceneId    : number
                      // The compiled WebAssembly module.
                      // *must* be the module obtained from the `src` directory
                    , mod        : WebAssembly.Module
                      // Scene camera
                    , camera     : Camera ) {
    this._worker  = new Worker( 'worker.js' );

    let buffer  = new SharedArrayBuffer( width * height * 4 );
    this.buffer = new Uint8Array( buffer );

    let initMsg : MsgC2WInit =
      { type: 'init'
      , mod
      , sceneId
      , buffer
      , width
      , height
      , camera
      };
    
    this._worker.postMessage( initMsg );

    this._onUpdate = new XObservable( );

    this._worker.addEventListener( 'message', ev => {
      let msg = ev.data;
      if ( msg.type === 'compute_done' ) {
        // Notify listeners, such that the screen can be updated
        this._onUpdate.next( );
      }
    } );
  }

  // Gets notified whenever the render buffer has updated
  public onUpdate( ): Observable< void > {
    return this._onUpdate.observable;
  }

  // Pauses path tracing execution
  public pause( ): void {
    let pauseMsg : MsgC2WPause = { type: 'pause' };
    this._worker.postMessage( pauseMsg );
  }

  // Resume path tracing execution
  public resume( ): void {
    let resumeMsg : MsgC2WResume = { type: 'resume' };
    this._worker.postMessage( resumeMsg );
  }

  // Destroys the entire instance. Should always be called when disposing it.
  //   (Otherwise WebWorkers may remain lingering zombies)
  public destroy( ): void {
    this._worker.terminate( );
  }

  // Updates the render settings. This restarts the renderer
  public updateSettings(
        leftType : number, rightType : number
      , isLeftAdaptive : boolean, isRightAdaptive : boolean
      , isLightDebug : boolean ) {
    let msg : MsgC2WUpdateSettings = { type: 'update_settings', leftType, rightType, isLeftAdaptive, isRightAdaptive, isLightDebug };
    this._worker.postMessage( msg );
  }

  // Updates the buffer that is shown to the screen
  // Either the normal diffuse buffer (if false), or the light-selection debug buffer
  public updateViewType( isShowingSamplingStrategy : boolean ) {
    let msg : MsgC2WUpdateViewType = { type: 'update_view_type', isShowingSamplingStrategy };
    this._worker.postMessage( msg );
  }

  // Updates the scene that is rendered.
  // Affects *following* render calls (so not any currently active calls)
  public updateScene( sceneId : number ): void {
    let msg : MsgC2WUpdateScene = { type: 'update_scene', sceneId };
    this._worker.postMessage( msg );
  }

  // Updates the viewport
  public updateViewport( width : number, height : number ): void {
    let buffer  = new SharedArrayBuffer( width * height * 4 );
    this.buffer = new Uint8Array( buffer );

    let msg : MsgC2WUpdateViewport = { type: 'update_viewport', width, height, buffer };
    this._worker.postMessage( msg );
  }

  // Updates the camera
  // It *first* rotates around the x-axis, and then the y-axis. And then translation is applied
  public updateCamera( camera : Camera ): void {
    let msg : MsgC2WUpdateCamera = { type: 'update_camera', camera };
    this._worker.postMessage( msg );
  }

  // Meshes are obtained (e.g. read from a file) externally, and provided to
  // the raytracer through this method.
  public storeMesh( id : number, mesh : Triangles ): void {
    let msg : MsgC2WStoreMesh = { type: 'store_mesh', id, mesh };
    this._worker.postMessage( msg );
  }

  // Textures are obtained externally, and sent to the raytracer through this
  // method.
  public storeTexture( id : number, texture : Texture ): void {
    let msg : MsgC2WStoreTexture = { type: 'store_texture', id, texture };
    this._worker.postMessage( msg );
  }
}
