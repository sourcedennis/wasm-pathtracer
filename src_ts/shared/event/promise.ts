
// A Promise encapsulated with an external means to resolve it
//   This external method (through calling `fulfil(..)`) is the
//   only way of resolving it.
export class EmptyPromise< T > {
  public  readonly promise : Promise< T >;
  private          _fResolve: ( v: T | PromiseLike< T > ) => void;

  public constructor( ) {
    this.promise = new Promise( ( fResolve, fReject ) => {
      this._fResolve = fResolve;
    } );
  }

  public fulfil( v: T ): void {
    this._fResolve( v );
  }
}
