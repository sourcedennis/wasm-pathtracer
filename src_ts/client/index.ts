import { Observable, XObservable }     from '@s/event/observable';
import { Vec3 }                        from '@s/math/vec3';
import { Camera }                      from '@s/graphics/camera';
import { Triangles }                   from '@s/graphics/triangles';
import { CHECKER_RED_YELLOW, Texture }          from '@s/graphics/texture';
import { Runner }                      from './control/runner';
import { RenderTarget, CanvasElement } from './control/render_target';
import { FpsTracker }                  from './control/fps_tracker';
import { CameraController }            from './input/camera_controller';
import { Raytracer }                   from './raytracer';
import { SinglecoreRaytracer }         from './raytracer/singlecore';
import { MulticoreRaytracer }          from './raytracer/multicore';
import { Elm }                         from './SidePanel.elm';
import { parseObj }                    from './obj_parser';
import { MeshId }                      from './meshes';

// A configuration for the raytracer
// It is modified by UI options
class Config {
  // Viewport width
  public width            : number;
  // Viewport height
  public height           : number;
  // True if it is currently running
  public isRunning        : boolean;
  // True if it is running using 8 WebWorkers. On the main thread otherwise
  public isMulticore      : boolean;
  // The maximum number of ray bounces
  public rayDepth         : number;
  // True if rendering a depth-buffer. Diffuse-buffer otherwise
  public isRenderingDepth : boolean;
  // An unique id for hard-coded scenes. (Defined in the Rust part)
  public sceneId          : number;

  public constructor( ) {
    this.width            = 512;
    this.height           = 512;
    this.isRunning        = true;
    this.isMulticore      = false;
    this.rayDepth         = 1;
    this.isRenderingDepth = false; // depth-map vs color
    this.sceneId          = 0;
  }
}

// An enclosing of the global raytracer state. Any interactions with the UI
//   can call methods on this environment, which coordinates it with the
//   relevant sub-components
class Global {
  // The active configuration
  private readonly _config           : Config;
  // The compiled WASM module (which contains the actual raytracer)
  private readonly _mod              : WebAssembly.Module;
  // Provides camera movement through keyboard controls
  private readonly _cameraController : CameraController;
  // The off-screen RGBA render target
  private          _target           : RenderTarget;
  // The manager of the on-screen render target
  private readonly _canvasElem       : CanvasElement;
  // The actual raytracer (either singlecore or multicore)
  private          _raytracer        : Raytracer;
  // This keeps track of the FPS of the active raytracer
  private readonly _fpsTracker       : FpsTracker;
  // This continuously calls the raytrace function
  private          _runner           : Runner | undefined;
  // All the meshes that were loaded (as polygon soup)
  private          _meshes           : Map< number, Triangles >;
  // All the textures that were loaded
  private          _textures         : Map< number, Texture >;

  // The on-screen canvas
  private readonly _canvas       : HTMLCanvasElement;
  // Gets called whenever a frame is rendered. The tuple are frame-render-time
  //   statistics from the last second of the form: [avg, min, max]
  private readonly _onRenderDone : XObservable< [number,number,number] >;

  // Constructs a new managing environment for the provided on-screen canvas
  public constructor( canvas : HTMLCanvasElement, mod : WebAssembly.Module ) {
    this._canvas  = canvas;
    this._mod     = mod;
    this._config  = new Config( );
    this._cameraController =
      new CameraController( sceneCamera( this._config.sceneId ));

    this._target       = new RenderTarget( this._config.width, this._config.height );
    this._canvasElem   = new CanvasElement( canvas, this._target );
    this._fpsTracker   = new FpsTracker( );
    this._onRenderDone = new XObservable( );
    this._meshes       = new Map( );
    this._textures     = new Map( );
    this._raytracer    = this._setupRaytracer( );
    this._runner       = new Runner( ( ) => this._render( ) ); // It's not running yet

    // Initially center the target in the canvas. Make sure the canvas
    // properly remains within the screen upon size
    this._onResize( );
    setTimeout( ( ) => { this._onResize( ); this._canvasElem.recenter( ); }, 0 );
    window.addEventListener( 'resize', ev => this._onResize( ) );

    this._cameraController.onUpdate( ).subscribe( c => {
      this._raytracer.updateCamera( c );
    } );
  }

  // Renders a depth-buffer if true. A diffuse-buffer otherwise
  public setRenderDepthBuffer( b : boolean ) {
    this._config.isRenderingDepth = b;
    this._raytracer.updateParams( this._config.isRenderingDepth, this._config.rayDepth );
    this._fpsTracker.clear( );
  }

  // Updates the maximum ray-depth of the renderer
  public setReflectionDepth( d : number ) {
    this._config.rayDepth = d;
    this._raytracer.updateParams( this._config.isRenderingDepth, this._config.rayDepth );
    this._fpsTracker.clear( );
  }

  // Starts or stops continuous raytracing
  // When running, it will render the next frame immediately after the previous
  // one is done.
  public updateRunning( r : boolean ) {
    this._config.isRunning = r;
    if ( !this._config.isRunning ) {
      if ( this._runner ) {
        this._runner.terminate( );
      }
    } else {
      this._fpsTracker.clear( );
      this._runner = new Runner( ( ) => this._render( ) );
    }
  }

  // If true, a multicore renderer (on 8 WebWorkers) is used
  // Otherwise, a singlecore renderer on the main thread is used
  public updateMulticore( isMulticore : boolean ) {
    this._config.isMulticore = isMulticore;
    this._raytracer.destroy( );
    if ( this._config.isRunning ) {
      this._fpsTracker.clear( );
    }
    this._raytracer = this._setupRaytracer( );
  }

  // Updates the scene that is currently rendered
  // The `sid` refers to the id of the hard-coded scene in the raytracer source.
  public updateScene( sid : number ) {
    console.log( 'update scene', sid );
    this._config.sceneId = sid;
    this._raytracer.updateScene( sid );

    this._cameraController.set( sceneCamera( sid ) );
    this._raytracer.updateCamera( this._cameraController.get( ) );
  }

  // Meshes can only be loaded by JavaScript, yet they need to be passed
  //   to the WASM module. This stores it in the global environment
  //   any current and future renders will have these meshes available
  // Note that meshes are "hardcoded" to be part of scenes (by their id)
  public storeMesh( id : number, mesh : Triangles ): void {
    this._meshes.set( id, mesh );
    this._raytracer.storeMesh( id, mesh );
  }

  public storeTexture( id : number, texture : Texture ): void {
    this._textures.set( id, texture );
    this._raytracer.storeTexture( id, texture );
  }

  // Gets notified when a frame is done rendering
  public onRenderDone( ): Observable< [ number, number, number ] > {
    return this._onRenderDone.observable;
  }

  // Constructs a new raytracer with the current configuration
  private _setupRaytracer( ): Raytracer {
    const c = this._config;

    let tracer : Raytracer;

    if ( c.isMulticore ) {
      tracer = new MulticoreRaytracer(
          c.width,
          c.height,
          c.sceneId,
          this._mod,
          c.isRenderingDepth,
          c.rayDepth,
          this._cameraController.get( ),
          8
        );
    } else {
      tracer = new SinglecoreRaytracer(
          c.width,
          c.height,
          c.sceneId,
          this._mod,
          c.isRenderingDepth,
          c.rayDepth,
          this._cameraController.get( )
        );
    }

    for ( let [id, mesh] of this._meshes ) {
      tracer.storeMesh( id, mesh );
    }
    for ( let [id, texture] of this._textures ) {
      tracer.storeTexture( id, texture );
    }

    return tracer;
  }

  private _onResize( ) {
    const canvas  = this._canvas;
    canvas.height = document.body.clientHeight;
    canvas.width  = document.body.clientWidth - 250 / (3 / 4);
    this._canvasElem.reclamp( );
  }

  // Renders a single frame asynchronously.
  // Upon completion a UInt8 RGBA pixel buffer is provided to the promise.
  private _render( ): Promise< void > {
    let startTime = Date.now( );
    return this._raytracer.render( ).then( res => {
      this._target.update( res );
      let currTime = Date.now( );
      this._fpsTracker.add( currTime, currTime - startTime );
      this._onRenderDone.next(
        [ this._fpsTracker.avg( )
        , this._fpsTracker.low( )
        , this._fpsTracker.high( )
        ] );
    } );
  }
}

// I like different initial cameras for some scenes
// These are defined here
function sceneCamera( sceneId : number ): Camera {
  if ( sceneId === 0 ) { // cubes and spheres
    return new Camera( new Vec3( -5.1, 4.6, 1.8 ), 0.54, 0.68 );
  } else if ( sceneId === 1 ) { // simple ball
    return new Camera( new Vec3( 0.0, 0.0, -1.0 ), 0, 0 );
  } else if ( sceneId === 2 ) { // air hole
    return new Camera( new Vec3( -3.7, 3.5, -0.35 ), 0.47, 0.54 );
  } else if ( sceneId === 3 ) { // .obj mesh
    return new Camera( new Vec3( 0, 0, 0 ), 0, 0 );
  } else if ( sceneId === 4 ) { // Whitted Turner's Scene
    return new Camera( new Vec3( -1.1, 1.4, -2.5 ), 0.1, 0.0 );
  } else {
    throw new Error( 'No Scene' );
  }
}

document.addEventListener( 'DOMContentLoaded', ev => {
  const canvas  = document.getElementsByTagName( 'canvas' )[ 0 ];

  // Download the compiled WebAssembly module, and construct the global
  // environment with it.
  (<any>WebAssembly).compileStreaming( fetch('pkg/index_bg.wasm'), { } )
    .then( compiledMod => {
      const env = new Global( canvas, compiledMod );

      let settingsPanel = document.getElementById( 'sidepanel' );
      const app = Elm.SidePanel.init( { node: settingsPanel } );
      app.ports.updateRenderType.subscribe( t => env.setRenderDepthBuffer( t == 1 ) );
      app.ports.updateReflectionDepth.subscribe( d => env.setReflectionDepth( d ) );
      app.ports.updateRunning.subscribe( r => env.updateRunning( r ) );
      app.ports.updateMulticore.subscribe( r => env.updateMulticore( r ) );
      app.ports.updateScene.subscribe( sid => env.updateScene( sid ) );

      env.onRenderDone( ).subscribe( res => app.ports.updatePerformance.send( res ) );

      fetch( 'torus.obj' ).then( f => f.text( ) ).then( s => {
        let triangles = parseObj( s );
        env.storeMesh( MeshId.MESH_TORUS, triangles );
      } );

      env.storeTexture( 0, CHECKER_RED_YELLOW );
    } );
} );
