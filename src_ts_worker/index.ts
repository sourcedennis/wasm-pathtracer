
class EmptyPromise< T > {
  public readonly promise : Promise< T >;
  private _fResolve: ( v : T ) => void;

  public constructor( ) {
    this.promise = new Promise( ( fResolve, fReject ) => {
      this._fResolve = fResolve;
    } );
  }

  public fulfil( v : T ): void {
    this._fResolve( v );
  }
}

class WasmRaytraceModule {
  private readonly _instance : any;
  private _cachegetInt32Memory: Int32Array | null;
  private _cachegetUint8Memory: Uint8Array | null;

  public constructor( instance : any ) {
    this._instance = instance;
    this._cachegetInt32Memory = null;
    this._cachegetUint8Memory = null;
  }

  public init( width : number, height : number ) {
    this._instance.init( width, height );
  }

  public compute( vp_x : number, vp_y : number, width : number, height : number, antiAlias : number ) {
    const retptr = 8;
    const ret = this._instance.compute(retptr, vp_x, vp_y, width, height, antiAlias);
    const memi32 = this._getInt32Memory();
    const v0 = this._getArrayU8FromWasm(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1]).slice();
    this._instance.__wbindgen_free(memi32[retptr / 4 + 0], memi32[retptr / 4 + 1] * 1);
    return v0;
  }

  private _getInt32Memory( ): Int32Array {
    if ( this._cachegetInt32Memory === null || this._cachegetInt32Memory.buffer !== this._instance.memory.buffer) {
        this._cachegetInt32Memory = new Int32Array(this._instance.memory.buffer);
    }
    return <Int32Array> this._cachegetInt32Memory;
  }

  private _getUint8Memory( ): Uint8Array {
    if (this._cachegetUint8Memory === null || this._cachegetUint8Memory.buffer !== this._instance.memory.buffer) {
      this._cachegetUint8Memory = new Uint8Array(this._instance.memory.buffer);
    }
    return <Uint8Array> this._cachegetUint8Memory;
  }

  private _getArrayU8FromWasm( ptr : number, len : number ): Uint8Array {
    return this._getUint8Memory( ).subarray(ptr / 1, ptr / 1 + len);
  }
}

interface Window {
  postMessage(message: any, transferable?: any[]): void;
}

const pWasm = new EmptyPromise< WasmRaytraceModule >( );

onmessage = function( ev ) {
  let msg = ev.data;
  
  if ( msg.type === 'init' ) {
    //console.log( msg.module );
    WebAssembly.instantiate( msg.module, {} ).then( instance => {
      pWasm.fulfil( new WasmRaytraceModule( ( <any> instance ).exports ) );
    } );
    //console.log( 'initialised' );
  } else if ( msg.type === 'call_setup_scene' ) {
    //console.log( 'call setup scene' );
    pWasm.promise.then( mod => {
      mod.init( msg.width, msg.height );
      self.postMessage( { jobId: msg.jobId } );
    } );
  } else if ( msg.type === 'call_compute' ) {
    //console.log( 'call compute' );
    pWasm.promise.then( mod => {
      let data = mod.compute( msg.x, msg.y, msg.width, msg.height, msg.antiAlias );
      self.postMessage( { jobId: msg.jobId, data }, [ data.buffer ] );
    } );
  } else {
    console.log( 'Other', msg.type );
  }
}

function _assertNum(n) {
  if (typeof(n) !== 'number') throw new Error('expected a number argument');
}
