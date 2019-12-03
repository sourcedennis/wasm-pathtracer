import { Camera }    from '@s/graphics/camera';
import { Triangles } from '@s/graphics/triangles';
import { Texture }   from '@s/graphics/texture';

// The interface that all raytraces implement
// It is configured with parameters (scene, camera, ray depth, etc.)
//   which are then used to (asynchronously) render the scene
// As these computations can take place externally (e.g. in other threads)
//   no guarantees exist on when this result is produced, but it is always
//   eventually produced once requested. Hence the Promise.
export interface Raytracer {
  // With the current configuration of the raytracer, trace the rays within the
  // scene. The result is a RGBA pixel buffer.
  // (Alpha is always 255, but this is convienient when pushing Canvas ImageData)
  render( ): Promise< Uint8Array >;

  // Destroys the entire instance. Should always be called when disposing it.
  //   (Otherwise WebWorkers may remain lingering zombies)
  destroy( ): void;

  // Updates the scene that is rendered.
  // Affects *following* render calls (so not any currently active calls)
  updateScene( sceneId : number ): void;

  // Updates the render parameters
  // Affects *following* render calls (so not any currently active calls)
  // `isDepth` is true if a depth-buffer is rendered. Otherwise a diffuse-buffer
  // `maxRayDepth` is the maximum number of ray-bounces in the raytracer
  updateParams( isDepth : boolean, maxRayDepth : number ): void;

  // Updates the camera
  // Affects *following* render calls (so not any currently active calls)
  // It *first* rotates around the x-axis, and then the y-axis. And then translation is applied
  updateCamera( camera : Camera ): void;

  // Updates the render viewport
  updateViewport( width : number, height : number ): void;

  // Meshes are obtained (e.g. read from a file) externally, and provided to
  // the raytracer through this method.
  storeMesh( id : number, mesh : Triangles ): void;

  // Textures are obtained externally, and sent to the raytracer through this
  // method.
  storeTexture( id : number, texture : Texture ): void;
}
