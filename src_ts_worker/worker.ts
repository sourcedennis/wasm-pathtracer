declare function postMessage( msg : any );

let instance : any;
let buffer : Uint8Array;
let pixels : any[];

onmessage = ev => {
  let msg = ev.data;

  if ( msg.type === 'init' ) {
    let mod = msg.mod;
    // mod, buffer, rays

    buffer = new Uint8Array( msg.buffer );
    
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
      ins.exports.init( msg.width, msg.height, msg.sceneId, msg.isDepth, msg.rayDepth
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

      postMessage( { type: 'init_done' } );
    } );
  } else if ( msg.type === 'compute' ) {
    instance.exports.compute( );
    let resPtr = instance.exports.results( );
    let mem8 = new Uint8Array( instance.exports.memory.buffer, resPtr, 512 * 512 * 4 );
    for ( let i = 0; i < pixels.length; i++ ) {
      let x = pixels[ i ].x;
      let y = pixels[ i ].y;

      buffer[ 4 * ( y * 512 + x ) + 0 ] = mem8[ 4 * ( y * 512 + x ) + 0 ];
      buffer[ 4 * ( y * 512 + x ) + 1 ] = mem8[ 4 * ( y * 512 + x ) + 1 ];
      buffer[ 4 * ( y * 512 + x ) + 2 ] = mem8[ 4 * ( y * 512 + x ) + 2 ];
      buffer[ 4 * ( y * 512 + x ) + 3 ] = mem8[ 4 * ( y * 512 + x ) + 3 ];
    }
    postMessage( { type: 'compute_done' } );
  } else if ( msg.type === 'update_params' ) {
    instance.exports.update_params( msg.isDepth ? 1 : 0, msg.maxRayDepth );
  } else if ( msg.type === 'update_camera' ) {
    let cam = msg.camera;
    instance.exports.update_camera( cam.location.x, cam.location.y, cam.location.z, cam.rotX, cam.rotY );
  } else if ( msg.type === 'update_scene' ) {
    instance.exports.update_scene( msg.sceneId );
  }
};
