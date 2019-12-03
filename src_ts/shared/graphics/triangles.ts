
export class Triangles {
  // 3 * n ; one vertex for each triangle corner
  public readonly vertices : Float32Array;
  // 1 * n ; one normal for each triangle
  public readonly normals  : Float32Array;

  public constructor( vertices : Float32Array, normals : Float32Array ) {
    this.vertices = vertices;
    this.normals  = normals;
  }
}
