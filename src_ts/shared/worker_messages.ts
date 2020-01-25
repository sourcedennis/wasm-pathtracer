import { Camera }    from './graphics/camera';
import { Vec2 }      from './math/vec2';
import { Triangles } from './graphics/triangles';
import { Texture }   from './graphics/texture';

export interface Msg {
  type : string;
}

// ### Client-to-Worker Messages ###
// C2W stands for Client-to-Worker

export interface MsgC2WInit extends Msg {
  type       : 'init',
  mod        : WebAssembly.Module,
  sceneId    : number,
  buffer     : SharedArrayBuffer,
  width      : number,
  height     : number,
  camera     : Camera
}

export interface MsgC2WUpdateCamera extends Msg {
  type   : 'update_camera',
  camera : Camera
}

export interface MsgC2WUpdateViewport extends Msg {
  type   : 'update_viewport',
  width  : number,
  height : number,
  // The new shared pixel buffer
  buffer : SharedArrayBuffer
}

export interface MsgC2WUpdateScene extends Msg {
  type    : 'update_scene',
  sceneId : number
}

export interface MsgC2WStoreMesh extends Msg {
  type : 'store_mesh',
  id   : number,
  mesh : Triangles
}

export interface MsgC2WStoreTexture extends Msg {
  type    : 'store_texture',
  id      : number,
  texture : Texture
}

export interface MsgC2WPause extends Msg {
  type : 'pause'
}

export interface MsgC2WResume extends Msg {
  type : 'resume'
}

// export interface MsgC2WRebuildBVH extends Msg {
//   type    : 'rebuild_bvh',
//   numBins : number,
//   isBVH4  : boolean
// }

// export interface MsgC2WDisableBVH extends Msg {
//   type : 'disable_bvh'
// }

// export interface MsgC2WCompute extends Msg {
//   type : 'compute'
// }

// ### Worker-to-Client Messages ###
// W2C stands for Worker-to-Client

// Sent when the initialisation of the WebAssembly Instance is done
export interface MsgW2CInitDone extends Msg {
  type : 'init_done'
}

// Sent when a part of the computation is done, such that the screen can be
// updated
export interface MsgW2CComputeDone extends Msg {
  type : 'compute_done'
}

// export interface MsgW2CBvhDone extends Msg {
//   type : 'bvh_done'
// }

// Sent when a texture is succesfully pushed to the path tracer
// export interface MsgW2CTextureDone extends Msg {
//   type: 'texture_done'
// }

// // Sent when a mesh is successfully pushed to the path tracer
// export interface MsgW2CMeshDone extends Msg {
//   type: 'mesh_done'
// }

// export interface MsgW2CUpdateSceneDone extends Msg {
//   type: 'update_scene_done'
// }
