import { Raytracer } from './index';
import { Camera }    from '@s/graphics/camera';
import { Triangles } from '@s/graphics/triangles';
import { Texture }   from '@s/graphics/texture';

// A raytracer that executes on the *main thread*, by executing the functions
// implemented in the Rust module. As it runs on the main thread, it may hang
// the browser when tracing bigger scenes.
//
// See the `src` directory in the project root for the actual implementation
// of the raytracing algorithms. This is more of an interface to that module.
export class SinglecoreRaytracer implements Raytracer {
  // An instance of the compiled WASM module
  private readonly _ins    : Promise< WebAssembly.Instance >;
  // Viewport width
  private readonly _width  : number;
  // Viewport height
  private readonly _height : number;

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
                    , camera     : Camera ) {
    this._width  = width;
    this._height = height;

    // This interface is demanded by the Rust compiler (or wasm_bindgen), it seems
    let importObject =
      { env: { abort: ( ) => console.log( 'abort' ) } };
      
    this._ins = WebAssembly.instantiate( mod, importObject ).then( ins => <any> ins ).then( ins => {
      // Pass stuff across WASM boundary (only primitives allowed)
      ins.exports.init( width, height, sceneId, renderType, rayDepth
                      , camera.location.x, camera.location.y, camera.location.z
                      , camera.rotX, camera.rotY );
      
      // Now put the assigned pixels into WASM memory. These are all pixels in the viewport
      //   (as it is only single-threaded)
      let rayPtr = ins.exports.ray_store( width * height );
      let rays = new Uint32Array( ins.exports.memory.buffer, rayPtr, width * height * 2 );

      for ( let y = 0; y < height; y++ ) {
        for ( let x = 0; x < width; x++ ) {
          rays[ 2 * ( y * width + x ) + 0 ] = x;
          rays[ 2 * ( y * width + x ) + 1 ] = y;
        }
      }
      ins.exports.ray_store_done( );
      return ins;
    } );
  }

  // See `Raytracer#render()`
  public render( ): Promise< Uint8Array > {
    return this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.compute( );
      return new Uint8Array( exps.memory.buffer, exps.results( ), this._width * this._height * 4 );
    } );
  }

  // See `Raytracer#updateScene()`
  public updateScene( sceneId : number ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.update_scene( sceneId );
    } );
  }

  // See `Raytracer#destroy()`
  public destroy( ): void {
    // No destruction necessary here, as all owned resources can be
    //   garbage-collected
  }

  // See `Raytracer#updateParams()`
  public updateParams( renderType : number, maxRayDepth : number ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.update_params( renderType, maxRayDepth );
    } );
  }

  // See `Raytracer#updateCamera()`
  public updateCamera( cam : Camera ) {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.update_camera( cam.location.x, cam.location.y, cam.location.z, cam.rotX, cam.rotY );
    } );
  }

  // See `Raytracer#updateViewport()`
  // public updateViewport( width : number, height : number ) {
  //   console.error( 'TODO: Update viewport' ); 
  // }

  // See `Raytracer#rebuildBVH()`
  public rebuildBVH( numBins : number ): Promise< number > {
    return this._ins.then( ins => {
      let exps = <any> ins.exports;
      let time = Date.now( );
      exps.rebuild_bvh( numBins );
      return Date.now( ) - time;
    } );
  }

  // See `Raytracer#storeMesh()`
  public storeMesh( id : number, mesh : Triangles ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      exps.allocate_mesh( id, mesh.vertices.length );
      let ptrVertices = exps.mesh_vertices( id );
      let dst = new Float32Array( exps.memory.buffer, ptrVertices, mesh.vertices.length );
      dst.set( mesh.vertices );
      exps.notify_mesh_loaded( id );
    } );
  }

  // See `Raytracer#storeTexture()`
  public storeTexture( id : number, texture : Texture ): void {
    this._ins.then( ins => {
      let exps = <any> ins.exports;
      let ptrRgb = exps.allocate_texture( id, texture.width, texture.height );
      let dst = new Uint8Array( exps.memory.buffer, ptrRgb, texture.width * texture.height * 3 );
      dst.set( texture.data );
      exps.notify_texture_loaded( id );
    } );
  }
}
