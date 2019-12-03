import { Camera } from './graphics/camera';
import { Vec2 } from './math/vec2';

export interface Msg {
  type : string;
}

// ### Client-to-Worker Messages ###
// C2W stands for Client-to-Worker

export interface MsgC2WInit extends Msg {
  type     : 'init',
  mod      : WebAssembly.Module,
  sceneId  : number,
  pixels   : Vec2[],
  buffer   : SharedArrayBuffer,
  width    : number,
  height   : number,
  isDepth  : boolean,
  rayDepth : number,
  camera   : Camera
}

export interface MsgC2WUpdateCamera extends Msg {
  type   : 'update_camera',
  camera : Camera
}

export interface MsgC2WUpdateParams extends Msg {
  type        : 'update_params',
  isDepth     : boolean,
  maxRayDepth : number
}

export interface MsgC2WUpdateScene extends Msg {
  type    : 'update_scene',
  sceneId : number
}

export interface MsgC2WCompute extends Msg {
  type : 'compute'
}

// ### Worker-to-Client Messages ###
// W2C stands for Worker-to-Client

export interface MsgW2CInitDone extends Msg {
  type : 'init_done'
}

export interface MsgW2CComputeDone extends Msg {
  type : 'compute_done'
}
