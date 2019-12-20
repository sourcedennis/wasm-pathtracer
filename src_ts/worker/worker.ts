import { Msg, MsgC2WInit, MsgC2WCompute, MsgC2WUpdateCamera, MsgC2WUpdateParams
       , MsgC2WUpdateScene, MsgW2CInitDone, MsgW2CComputeDone, MsgW2CBvhDone
       , MsgC2WStoreMesh, MsgC2WStoreTexture, MsgC2WRebuildBVH
       } from '@s/worker_messages';
import { MsgHandler } from './msg_handler';

declare function postMessage( msg : any ): void;

// ### Global State (to the worker) ###
let instance : any;
let width    : number;
let height   : number;
let buffer   : Uint8Array;
let pixels   : any[];

const handlers = new MsgHandler( );
handlers.register( 'init',          handleInit );
handlers.register( 'compute',       handleCompute );
handlers.register( 'update_params', handleUpdateParams )
handlers.register( 'update_camera', handleUpdateCamera );
handlers.register( 'update_scene',  handleUpdateScene );
handlers.register( 'store_mesh',    handleStoreMesh );
handlers.register( 'store_texture', handleStoreTexture );
handlers.register( 'rebuild_bvh',   handleRebuildBvh );
handlers.register( 'disable_bvh',   handleDisableBvh );

onmessage = ev => {
  handlers.handle( ev.data );
};

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

  let camera = msg.camera;

  ( <any> WebAssembly ).instantiate( mod, importObject ).then( ins => {
    let iStartTime = Date.now( );
    // Pass all the primitives to initialisation
    ins.exports.init( msg.width, msg.height, msg.sceneId, msg.renderType, msg.rayDepth
      , camera.location.x, camera.location.y, camera.location.z, camera.rotX, camera.rotY );

    let rayPtr = ins.exports.ray_store( msg.pixels.length );
    let raysDst = new Uint32Array( ins.exports.memory.buffer, rayPtr, msg.width * msg.height * 2 );

    for ( let i = 0; i < msg.pixels.length; i++ ) {
      raysDst[ 2 * i + 0 ] = msg.pixels[ i ].x;
      raysDst[ 2 * i + 1 ] = msg.pixels[ i ].y;
    }
    ins.exports.ray_store_done( );

    console.log( 'init time', Date.now( ) - iStartTime );

    pixels = msg.pixels;
    instance = ins;

    postMessage( <MsgW2CInitDone> { type: 'init_done' } );
  } );
}

function handleCompute( msg : MsgC2WCompute ) {
  let numBVHHits = instance.exports.compute( );
  let resPtr = instance.exports.results( );
  let mem8 = new Uint8Array( instance.exports.memory.buffer, resPtr, width * height * 4 );
  for ( let i = 0; i < pixels.length; i++ ) {
    let x = pixels[ i ].x;
    let y = pixels[ i ].y;

    buffer[ 4 * ( y * width + x ) + 0 ] = mem8[ 4 * ( y * width + x ) + 0 ];
    buffer[ 4 * ( y * width + x ) + 1 ] = mem8[ 4 * ( y * width + x ) + 1 ];
    buffer[ 4 * ( y * width + x ) + 2 ] = mem8[ 4 * ( y * width + x ) + 2 ];
    buffer[ 4 * ( y * width + x ) + 3 ] = mem8[ 4 * ( y * width + x ) + 3 ];
  }
  postMessage( <MsgW2CComputeDone> { type: 'compute_done', numBVHHits } );
}

function handleUpdateParams( msg : MsgC2WUpdateParams ) {
  instance.exports.update_params( msg.renderType, msg.maxRayDepth );
}

function handleUpdateCamera( msg : MsgC2WUpdateCamera ) {
  const cam = msg.camera;
  instance.exports.update_camera( cam.location.x, cam.location.y, cam.location.z, cam.rotX, cam.rotY );
}

function handleUpdateScene( msg : MsgC2WUpdateScene ) {
  instance.exports.update_scene( msg.sceneId );
  postMessage( <Msg> { type: 'update_scene_done' } ); // TODO
}

function handleStoreMesh( msg : MsgC2WStoreMesh ) {
  let exps = <any> instance.exports;
  exps.allocate_mesh( msg.id, msg.mesh.vertices.length );
  let ptrVertices = exps.mesh_vertices( msg.id );
  let dst = new Float32Array( exps.memory.buffer, ptrVertices, msg.mesh.vertices.length );
  dst.set( msg.mesh.vertices );
  exps.notify_mesh_loaded( msg.id );
  postMessage( <Msg> { type: 'mesh_done' } ); // TODO
}

function handleStoreTexture( msg : MsgC2WStoreTexture ) {
  let exps = <any> instance.exports;
  let ptrRgb = exps.allocate_texture( msg.id, msg.texture.width, msg.texture.height );
  let dst = new Uint8Array( exps.memory.buffer, ptrRgb, msg.texture.width * msg.texture.height * 3 );
  dst.set( msg.texture.data );
  exps.notify_texture_loaded( msg.id );
  postMessage( <Msg> { type: 'texture_done' } ); // TODO
}

function handleRebuildBvh( msg : MsgC2WRebuildBVH ) {
  let exps = <any> instance.exports;
  exps.rebuild_bvh( msg.numBins );
  postMessage( <MsgW2CBvhDone> { type: 'bvh_done' } );
}

function handleDisableBvh( msg : MsgC2WRebuildBVH ) {
  let exps = <any> instance.exports;
  exps.disable_bvh( );
}
