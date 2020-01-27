import { Msg, MsgC2WInit, MsgC2WUpdateCamera
       , MsgC2WUpdateScene, MsgW2CInitDone, MsgW2CComputeDone
       , MsgC2WStoreMesh, MsgC2WStoreTexture
       , MsgC2WPause, MsgC2WResume, MsgC2WUpdateViewport, MsgC2WUpdateSettings
       , MsgC2WUpdateViewType
       } from '@s/worker_messages';
import { MsgHandler } from './msg_handler';
import { Camera } from '@s/graphics/camera';

declare function postMessage( msg : any ): void;

// ### Global State (to the worker) ###
let instance  : any; // WebAssembly.Instance
let width     : number;
let height    : number;
let buffer    : Uint8Array;
let isRunning : boolean;
// Check if the "run-loop" actually stopped, to avoid double run loops
let hasRegisteredStopping : boolean;
// The number of rays that are rendered before the screen is updated
// This amount is adjusted to be approx 50ms
let numRaysPerTick : number = 500;

let hasUpdatedCamera   : boolean = false;
let hasUpdatedViewport : boolean = false;
let camera             : Camera;

// If false, it shows the normal diffuse pixel buffer. If true, the
// sample-frequency of pixels is shown instead.
// (green is below average, blue is average, red is above average)
let isShowingSamplingStrategy : boolean = false;

// The worker handles messages from the main thread
// These typically pass information along to the WASM module
//   and return a confirmation message.
const handlers = new MsgHandler( );
handlers.register( 'init',             handleInit );
handlers.register( 'update_viewport',  handleUpdateViewport );
handlers.register( 'update_camera',    handleUpdateCamera );
handlers.register( 'update_scene',     handleUpdateScene );
handlers.register( 'update_settings',  handleUpdateSettings );
handlers.register( 'update_view_type', handleUpdateViewType );
handlers.register( 'store_mesh',       handleStoreMesh );
handlers.register( 'store_texture',    handleStoreTexture );
handlers.register( 'pause',            handlePause );
handlers.register( 'resume',           handleResume );

onmessage = ev => {
  handlers.handle( ev.data );
};

// ## The message handlers ##

// Performs a single "tick" of computation
function run( ) {
  if ( !isRunning ) {
    hasRegisteredStopping = true;
    return;
  }

  if ( hasUpdatedCamera ) {
    let c = camera;
    instance.exports.update_camera( c.location.x, c.location.y, c.location.z, c.rotX, c.rotY );
    hasUpdatedCamera = false;
  }
  if ( hasUpdatedViewport ) {
    instance.exports.update_viewport( width, height );
    hasUpdatedViewport = false;
  }

  let startTime = Date.now( );
  instance.exports.compute( numRaysPerTick );
  let endTime = Date.now( );

  if ( startTime == endTime ) {
    numRaysPerTick = 1000;
  } else {
    // Make sure on tick takes around 50ms
    let scale = 50 / ( endTime - startTime );
    numRaysPerTick = Math.floor( numRaysPerTick * scale );
  }

  // Store the result in shared memory
  let resPtr = instance.exports.results( isShowingSamplingStrategy );
  let mem8 = new Uint8Array( instance.exports.memory.buffer, resPtr, width * height * 4 );
  buffer.set( mem8, 0 );

  // And notify the main thread, to write the shared buffer to the screen
  postMessage( < MsgW2CComputeDone > { type: 'compute_done' } );

  // Make sure to give back control to the JavaScript event loop, which handles
  // new messages
  isRunning = true;
  setTimeout( ( ) => run( ), 0 );
}

// Initialises the worker state and WASM module
function handleInit( msg : MsgC2WInit ): void {
  let mod = msg.mod;

  buffer = new Uint8Array( msg.buffer );
  width  = msg.width;
  height = msg.height;

  let importObject =
    { env: {
        abort: arg => {
          console.log( 'abort' );
        }
      }
    };

  let cam = msg.camera;
  camera = msg.camera;

  ( <any> WebAssembly ).instantiate( mod, importObject ).then( ins => {
    instance = ins;

    // Pass all the primitives to initialisation
    ins.exports.init( msg.width, msg.height, msg.sceneId,
      cam.location.x, cam.location.y, cam.location.z, cam.rotX, cam.rotY );

    postMessage( <MsgW2CInitDone> { type: 'init_done' } );

    console.log( 'init done' );

    isRunning = true;
    setTimeout( ( ) => run( ), 0 );
  } );
}

// Updates the viewport. Note that this is only performed in the next `run()` call
function handleUpdateViewport( msg : MsgC2WUpdateViewport ) {
  width  = msg.width;
  height = msg.height;
  buffer = new Uint8Array( msg.buffer );
  hasUpdatedViewport = true;
}

// Updates the camera location. Note that this is only performed in the next `run()` call
function handleUpdateCamera( msg : MsgC2WUpdateCamera ) {
  camera = msg.camera;
  hasUpdatedCamera = true;
}

// Selects a new scene. `msg.sceneId` is a (magic) integer that is shared
//   between the Elm front-end and WASM module for communication.
function handleUpdateScene( msg : MsgC2WUpdateScene ) {
  instance.exports.update_scene( msg.sceneId );
  // Updating the scene makes it black. Redraw it
  postMessage( < MsgW2CComputeDone > { type: 'compute_done' } );
}

function handleUpdateSettings( msg : MsgC2WUpdateSettings ) {
  instance.exports.update_settings( msg.leftType, msg.rightType, msg.isLeftAdaptive ? 1 : 0, msg.isRightAdaptive ? 1 : 0, msg.isLightDebug ? 1 : 0 );
}

function handleUpdateViewType( msg : MsgC2WUpdateViewType ) {
  isShowingSamplingStrategy = msg.isShowingSamplingStrategy;

  // Store the result in shared memory
  let resPtr = instance.exports.results( isShowingSamplingStrategy ? 1 : 0 );
  let mem8 = new Uint8Array( instance.exports.memory.buffer, resPtr, width * height * 4 );
  buffer.set( mem8, 0 );

  // And notify the main thread, to write the shared buffer to the screen
  postMessage( < MsgW2CComputeDone > { type: 'compute_done' } );
}

// Passes a mesh to the WASM client
function handleStoreMesh( msg : MsgC2WStoreMesh ) {
  let exps = <any> instance.exports;
  let numVertices = msg.mesh.vertices.length / 3;
  exps.allocate_mesh( msg.id, numVertices );
  let ptrVertices = exps.mesh_vertices( msg.id );
  let dst = new Float32Array( exps.memory.buffer, ptrVertices, msg.mesh.vertices.length );
  dst.set( msg.mesh.vertices );
  exps.notify_mesh_loaded( msg.id );
}

// Passes a texture to the WASM client
function handleStoreTexture( msg : MsgC2WStoreTexture ) {
  let exps = <any> instance.exports;
  let ptrRgb = exps.allocate_texture( msg.id, msg.texture.width, msg.texture.height );
  let dst = new Uint8Array( exps.memory.buffer, ptrRgb, msg.texture.width * msg.texture.height * 3 );
  dst.set( msg.texture.data );
  exps.notify_texture_loaded( msg.id );
}

// Pauses the continuous render calls
function handlePause( msg : MsgC2WPause ) {
  isRunning = false;
}

// Resumes the continuous render calls
function handleResume( msg : MsgC2WResume ) {
  if ( !isRunning ) {
    if ( hasRegisteredStopping ) {
      // The previous loop terminated, so start a new one
      hasRegisteredStopping = false;
      isRunning = true;
      setTimeout( ( ) => run( ), 0 );
    } else {
      // It was marked to stop, but didn't actually stop. So reuse the old
      // render loop
      isRunning = true;
    }
  }
}
