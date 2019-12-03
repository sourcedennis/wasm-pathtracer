// Continuously call an asynchronous function
// That is, it is called again upon its previous completion.
// However, it does so after a delay of 0ms, which means that control is briefly
//   returned to the browser's event-loop to avoid completely hanging the
//   browser. This also makes sure `terminate()` requests are registered.
// After calling `terminate()` the call-loop stops executing.
export class Runner {
  private _isTerminated : boolean;

  // Constructs a new asynchronous call-loop. It starts immediately upon
  //   construction.
  public constructor( f: ( ) => Promise< void > ) {
    this._isTerminated = false;
    go( this );

    function go( self : Runner ) {
      f( ).then( ( ) => {
        if ( !self._isTerminated ) {
          setTimeout( ( ) => go( self ), 0 );
        }
      } );
    }
  }

  // Terminates the currently running call-loop
  public terminate( ): void {
    this._isTerminated = true;
  }
}
