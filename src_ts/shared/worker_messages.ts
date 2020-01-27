import { Camera }    from './graphics/camera';
import { Triangles } from './graphics/triangles';
import { Texture }   from './graphics/texture';

export interface Msg {
  type : string;
}

// ### Client-to-Worker Messages ###
// C2W stands for Client-to-Worker

// Initialises the renderer
export interface MsgC2WInit extends Msg {
  type       : 'init',
  // The compiled WebAssembly module that should be instantiated by the
  // render worker.
  mod        : WebAssembly.Module,
  sceneId    : number,
  buffer     : SharedArrayBuffer,
  width      : number,
  height     : number,
  camera     : Camera
}

// Moves the camera
export interface MsgC2WUpdateCamera extends Msg {
  type   : 'update_camera',
  camera : Camera
}

// Resizes the viewport
export interface MsgC2WUpdateViewport extends Msg {
  type   : 'update_viewport',
  width  : number,
  height : number,
  // The new shared pixel buffer
  buffer : SharedArrayBuffer
}

// Selects a new scene to be rendered. This restarts the render
export interface MsgC2WUpdateScene extends Msg {
  type    : 'update_scene',
  sceneId : number
}

// Stores a mesh in the renderer's memory
export interface MsgC2WStoreMesh extends Msg {
  type : 'store_mesh',
  id   : number,
  mesh : Triangles
}

// Stores a texture in the renderer's memory
export interface MsgC2WStoreTexture extends Msg {
  type    : 'store_texture',
  id      : number,
  texture : Texture
}

// Pauses rendering
export interface MsgC2WPause extends Msg {
  type : 'pause'
}

// Resumes rendering
export interface MsgC2WResume extends Msg {
  type : 'resume'
}

// Updates fundamental settings of the renderer. This restarts the render.
export interface MsgC2WUpdateSettings extends Msg {
  type            : 'update_settings',
  leftType        : number, //0=NoNEE, 1=NEE, 2=PNEE
  rightType       : number,
  isLeftAdaptive  : boolean,
  isRightAdaptive : boolean,
  isLightDebug    : boolean
}

// Changes the buffer that is shown. Either the diffuse render buffer or a
// visualisation of the sampled pixels. This does *not* reset the render.
export interface MsgC2WUpdateViewType {
  type                      : 'update_view_type',
  // If true, show the sampling strategy. Otherwise show the diffuse buffer
  isShowingSamplingStrategy : boolean
}

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
