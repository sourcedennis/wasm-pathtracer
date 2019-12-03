import { Triangles } from '@s/graphics/triangles';

export function parseObj( s : string ): Triangles {
  let lines = s.split( '\n' );

  let vertices : number[] = [];
  let normals : number[] = [];
  let facesVertices : number[] = [];
  let facesNormals  : number[] = [];
  
  for ( let i = 0; i < lines.length; i++ ) {
    let l = lines[ i ];
    let segs = l.split( ' ' );

    if ( segs[ 0 ] === 'v' ) {
      let x = parseFloat( segs[ 1 ] );
      let y = parseFloat( segs[ 2 ] );
      let z = parseFloat( segs[ 3 ] );
      vertices.push( x, y, z );
    } else if ( segs[ 0 ] === 'vn' ) {
      let x = parseFloat( segs[ 1 ] );
      let y = parseFloat( segs[ 2 ] );
      let z = parseFloat( segs[ 3 ] );
      normals.push( x, y, z );
    } else if ( segs[ 0 ] === 'f' ) {
      //console.log( 'face' );
      if ( segs.length !== 4 ) {
        throw new Error( 'Non-triangular face in OBJ file' );
      }
      let v1 = segs[ 1 ].split( '/' );
      let v2 = segs[ 2 ].split( '/' );
      let v3 = segs[ 3 ].split( '/' );
      facesVertices.push( parseInt( v1[0] )-1, parseInt( v2[0] )-1, parseInt( v3[0] )-1 );
      facesNormals.push( parseInt( v1[2] )-1, parseInt( v2[2] )-1, parseInt( v3[2] )-1 );
    } else if ( segs[ 0 ] === '#' ) {
      //console.log( 'comment' );
    } else {
      //console.log( 'OBJ unknown: ', lines[ i ] );
    }
  }

  let outVertices = new Float32Array( facesVertices.length * 3 );
  let outNormals  = new Float32Array( facesNormals.length * 3 );

  for ( let i = 0; i < facesVertices.length; i++ ) {
    outVertices[ i * 3 + 0 ] = vertices[ facesVertices[ i ] * 3 + 0 ];
    outVertices[ i * 3 + 1 ] = vertices[ facesVertices[ i ] * 3 + 1 ];
    outVertices[ i * 3 + 2 ] = vertices[ facesVertices[ i ] * 3 + 2 ];
    outNormals[ i * 3 + 0 ]  = normals[ facesNormals[ i ] * 3 + 0 ];
    outNormals[ i * 3 + 1 ]  = normals[ facesNormals[ i ] * 3 + 1 ];
    outNormals[ i * 3 + 2 ]  = normals[ facesNormals[ i ] * 3 + 2 ];
  }

  return new Triangles( outVertices, outNormals );
}
