import { Observable, XObservable }     from '@s/event/observable';
import { Vec3 }                        from '@s/math/vec3';
import { Camera }                      from '@s/graphics/camera';
import { Triangles }                   from '@s/graphics/triangles';
import { CHECKER_RED_YELLOW, Texture } from '@s/graphics/texture';
import { RenderTarget, CanvasElement } from './render_target';
import { CameraController }            from './input/camera_controller';
import { BackgroundPathTracer }        from './background_pathtracer';
import { Elm as ElmScene }             from './PanelScenes.elm';
import { Elm as ElmSettings }          from './PanelSettings.elm';
import { parseObj }                    from './obj_parser';
import { MeshId }                      from './meshes';

// An enclosing of the global raytracer state. Any interactions with the UI
//   can call methods on this environment, which coordinates it with the
//   relevant sub-components
class Global {
  // The compiled WASM module (which contains the actual raytracer)
  private readonly _mod              : WebAssembly.Module;
  // Provides camera movement through keyboard controls
  private readonly _cameraController : CameraController;
  // The off-screen RGBA render target
  private          _target           : RenderTarget;
  // The manager of the on-screen render target
  private readonly _canvasElem       : CanvasElement;
  // The actual path tracer (runs in a background WebWorker)
  private          _tracer           : BackgroundPathTracer;

  // The on-screen canvas
  private readonly _canvas : HTMLCanvasElement;

  // Constructs a new managing environment for the provided on-screen canvas
  public constructor( canvas : HTMLCanvasElement, mod : WebAssembly.Module ) {
    // An unique id for hard-coded scenes. (Defined in the Rust part)
    let sceneId = 0;
    let initialWidth  = 512;
    let initialHeight = 512;

    this._canvas  = canvas;
    this._mod     = mod;
    this._cameraController =
      new CameraController( sceneCamera( sceneId ));

    this._target     = new RenderTarget( initialWidth, initialHeight );
    this._canvasElem = new CanvasElement( canvas, this._target );
    this._tracer     = new BackgroundPathTracer(
        initialWidth
      , initialHeight
      , sceneId
      , this._mod
      , this._cameraController.get( )
      );

    // Initially center the target in the canvas. Make sure the canvas
    // properly remains within the screen upon size
    this._onResize( );
    setTimeout( ( ) => { this._onResize( ); this._canvasElem.recenter( ); }, 0 );
    window.addEventListener( 'resize', ev => this._onResize( ) );

    this._cameraController.onUpdate( ).subscribe( c => {
      this._tracer.updateCamera( c );
    } );

    this._tracer.onUpdate( ).subscribe( ( ) => {
      this._target.update( this._tracer.buffer );
    } );
  }

  // Starts or stops continuous raytracing
  // When running, it will render the next frame immediately after the previous
  // one is done.
  public updateRunning( r : boolean ) {
    if ( r ) {
      this._tracer.resume( );
    } else {
      this._tracer.pause( );
    }
  }

  // Selects another scene to be rendered
  // The `sid` refers to the id of the hard-coded scene in the raytracer source.
  public updateScene( sid : number ) {
    console.log( 'update scene', sid );
    this._tracer.updateScene( sid );

    this._cameraController.set( sceneCamera( sid ) );
    this._tracer.updateCamera( this._cameraController.get( ) );
  }

  // Updates the size of the viewport of the renderer
  public updateViewport( width : number, height : number ) {
    this._target = new RenderTarget( width, height );
    this._canvasElem.updateTarget( this._target );
    this._tracer.updateViewport( width, height );
  }

  // Meshes can only be loaded by JavaScript, yet they need to be passed
  //   to the WASM module. This stores it in the global environment
  //   any current and future renders will have these meshes available
  // Note that meshes are "hardcoded" to be part of scenes (by their id)
  public storeMesh( id : number, mesh : Triangles ): void {
    this._tracer.storeMesh( id, mesh );
  }

  // Stores a texture in the WASM module
  public storeTexture( id : number, texture : Texture ): void {
    this._tracer.storeTexture( id, texture );
  }

  // Gets notified when the camera updates
  public onCameraUpdate( ): Observable< Camera > {
    return this._cameraController.onUpdate( );
  }

  // Triggers the camera update with the current camera
  // Do this when a new camera listener is set, as otherwise it won't know the
  //   current state of the camera
  public triggerCameraUpdate( ): void {
    this._cameraController.set( this._cameraController.get( ) );
  }

  private _onResize( ) {
    const canvas  = this._canvas;
    canvas.height = document.body.clientHeight;
    canvas.width  = document.body.clientWidth - 2 * ( 250 / (3 / 4) );
    this._canvasElem.reclamp( );
  }
}

// I like different initial cameras for some scenes
// These are defined here
function sceneCamera( sceneId : number ): Camera {
  if ( sceneId === 0 ) { // cubes and spheres
    return new Camera( new Vec3( -4.5, 4.2, -2.1 ), 0.12, 0.53 );
  } else if ( sceneId === 1 || sceneId == 2 ) { // bunnies
    return new Camera( new Vec3( -0.9, 5.4, 0.4 ), 0.58, 0.0 );
  } else if ( sceneId === 3 || sceneId === 4 || sceneId === 5 ) { // clouds
    return new Camera( new Vec3( 0.0, 4.8, 2.6 ), 0.97, 0.0 );
  } else if ( sceneId === 6 ) { // marching
    return new Camera( new Vec3( 2.08, 1.29, 8.21 ), 0.39, -0.90 );
  } else {
    throw new Error( 'No Scene' );
  }
}

// Generates triangles in the box [-3,3]^3 around the origin
function triangleCloud( n : number ): Triangles {
  let vertices = new Float32Array( 9 * n );

  for ( let i = 0; i < n; i++ ) {
    let centerX = Math.random( ) * 5 - 2.5;
    let centerY = Math.random( ) * 5 - 2.5;
    let centerZ = Math.random( ) * 5;

    vertices[ i * 9 + 0 ] = centerX + Math.random( ) * 0.5;
    vertices[ i * 9 + 1 ] = centerY + Math.random( ) * 0.5;
    vertices[ i * 9 + 2 ] = centerZ + Math.random( ) * 0.5;
    vertices[ i * 9 + 3 ] = centerX + Math.random( ) * 0.5;
    vertices[ i * 9 + 4 ] = centerY + Math.random( ) * 0.5;
    vertices[ i * 9 + 5 ] = centerZ + Math.random( ) * 0.5;
    vertices[ i * 9 + 6 ] = centerX + Math.random( ) * 0.5;
    vertices[ i * 9 + 7 ] = centerY + Math.random( ) * 0.5;
    vertices[ i * 9 + 8 ] = centerZ + Math.random( ) * 0.5;
  }
  return new Triangles( vertices );
}

document.addEventListener( 'DOMContentLoaded', ev => {
  const canvas  = document.getElementsByTagName( 'canvas' )[ 0 ];

  // Download the compiled WebAssembly module, and construct the global
  // environment with it.
  (<any>WebAssembly).compileStreaming( fetch('pkg/index_bg.wasm'), { } )
    .then( compiledMod => {
      const env = new Global( canvas, compiledMod );

      const scenePanel = document.getElementById( 'scenepanel' );
      const appScenes = ElmScene.PanelScenes.init( { node: scenePanel } );
      appScenes.ports.updateScene.subscribe( sid => env.updateScene( sid ) );

      let settingsPanel = document.getElementById( 'settingspanel' );
      const appSettings = ElmSettings.PanelSettings.init( { node: settingsPanel } );
      appSettings.ports.updateRunning.subscribe( r => env.updateRunning( r ) );
      appSettings.ports.updateViewport.subscribe( vp => env.updateViewport( vp[ 0 ], vp[ 1 ] ) );

      env.triggerCameraUpdate( );

      fetch( 'bunny.obj' ).then( f => f.text( ) ).then( s => {
        let triangles = parseObj( s );
        let numVertices = triangles.vertices.length / 3;
        for ( let i = 0; i < numVertices; i++ ) {
          triangles.vertices[ i * 3 + 0 ] *= 8;
          triangles.vertices[ i * 3 + 1 ] *= 8;
          triangles.vertices[ i * 3 + 2 ] *= -8;
        }
        env.storeMesh( MeshId.BUNNY_LOW, triangles );
      } );
      fetch( 'bunny2.obj' ).then( f => f.text( ) ).then( s => {
        let triangles = parseObj( s );
        let numVertices = triangles.vertices.length / 3;
        for ( let i = 0; i < numVertices; i++ ) {
          triangles.vertices[ i * 3 + 0 ] *= 8;
          triangles.vertices[ i * 3 + 1 ] *= 8;
          triangles.vertices[ i * 3 + 2 ] *= -8;
        }
        env.storeMesh( MeshId.BUNNY_HIGH, triangles );
      } );

      env.storeMesh( MeshId.CLOUD_100,  triangleCloud( 100 ) );
      env.storeMesh( MeshId.CLOUD_10K,  triangleCloud( 10000 ) );
      env.storeMesh( MeshId.CLOUD_100K, triangleCloud( 100000 ) );

      env.storeTexture( 0, CHECKER_RED_YELLOW );
    } );
} );
