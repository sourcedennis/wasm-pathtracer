import { Vec3 } from '../math/vec3';

export class Camera {
  public readonly location : Vec3;
  public readonly rotX     : number;
  public readonly rotY     : number;

  public constructor( location : Vec3, rotX : number, rotY : number ) {
    this.location = location;
    this.rotX = rotX;
    this.rotY = rotY;
  }
}
