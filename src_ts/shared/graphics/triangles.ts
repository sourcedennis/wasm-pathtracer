
export class Triangles {
  // 3 * 3 * n ; one vertex for each of 3 triangle corners.
  //   Each vertex has 3 components (x,y,z)
  public readonly vertices : Float32Array;

  public constructor( vertices : Float32Array ) {
    this.vertices = vertices;
  }
}
