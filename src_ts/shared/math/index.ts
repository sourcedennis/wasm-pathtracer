
export function clamp( x : number, minVal : number, maxVal : number ): number {
  return Math.min( maxVal, Math.max( x, minVal ) );
}