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

    ( <any> WebAssembly ).instantiate( mod, importObject ).then( ins => {
      let iStartTime = Date.now( );
      ins.exports.init( msg.width, msg.height, msg.isDepth, msg.rayDepth );

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
  }
};
