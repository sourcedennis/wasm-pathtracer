// An immutable 3-dimensional number vector
export class Vec3 {
  public readonly x : number;
  public readonly y : number;
  public readonly z : number;

  public constructor( x : number, y : number, z : number ) {
    this.x = x;
    this.y = y;
    this.z = z;
  }

  public setX( v : number ): Vec3 {
    return new Vec3( v, this.y, this.z );
  }

  public setY( v : number ): Vec3 {
    return new Vec3( this.x, v, this.z );
  }

  public setZ( v : number ): Vec3 {
    return new Vec3( this.x, this.y, v );
  }

  public add( v : Vec3 ): Vec3 {
    return new Vec3( this.x + v.x, this.y + v.y, this.z + v.z );
  }

  public rotY( angle : number ): Vec3 {
    // [  c 0 s ] [x]
    // [  0 1 0 ] [y]
    // [ -s 0 c ] [z]
    let x = this.x;
    let y = this.y;
    let z = this.z;

    let c = Math.cos( angle );
    let s = Math.sin( angle );
    return new Vec3( c * x + s * z, y, -s * x + c * z )
  }

  public rotX( angle : number ): Vec3 {
    // [ 1 0  0 ] [x]
    // [ 0 c -s ] [y]
    // [ 0 s  c ] [z]
    let x = this.x;
    let y = this.y;
    let z = this.z;

    let c = Math.cos( angle );
    let s = Math.sin( angle )
    return new Vec3( x, c * y - s * z, s * y + c * z )
  }
}
