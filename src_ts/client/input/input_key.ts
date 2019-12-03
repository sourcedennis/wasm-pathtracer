import { Observable } from '@s/event/observable';

// A keytick event
// If more than one tick (10ms) should have occurred since the previous event,
//   then `count` reflects this. It is the number of ticks since the previous
//   event.
export class KeyTick {
  public readonly keyCode : number;
  public readonly count   : number;

  public constructor( keyCode : number, count : number ) {
    this.keyCode = keyCode;
    this.count   = count;
  }
}

type Interval = any;

// The default key-hold-down tick frequency in browsers is too slow
// This ticker ticks every 10ms while a key is pressed
// If (through OS-level thread sleeping) multiple ticks are missed, the returned
// `KeyTick` will counteract this by setting a `count` value equal to the number
//   of passed ticks since the previous event
// It *only* listens for key-hold-down events on the events with their keycode
//   in the provided input set `keys`
// Note that these events are registered *globally*; that is, on the `window`
//   object
export function keyTicker( keys : Set< number > ): Observable< KeyTick > {
  return new Observable< KeyTick >( observer => {
    let downKeys = new Map< number, Interval >( );

    // Start ticking when a key comes down
    window.addEventListener( 'keydown', ev => {
      if ( keys.has( ev.keyCode ) ) {
        if ( !downKeys.has( ev.keyCode ) ) {
          let ival = setInterval( ( ) => tick( ), 10 );
          downKeys.set( ev.keyCode, ival );
          observer.next( new KeyTick( ev.keyCode, 1 ) );
          // The last tick time for the current keycode
          let lastTickTime = Date.now( );

          function tick( ) {
            let currTime = Date.now( );
            let numTicks = Math.floor( ( currTime - lastTickTime ) / 10 );
            lastTickTime += numTicks * 10;
            observer.next( new KeyTick( ev.keyCode, numTicks ) );
          }
        }

        ev.preventDefault( );
      }
    } );

    // Stop the ticking when the key comes back up
    window.addEventListener( 'keyup', ev => {
      if ( keys.has( ev.keyCode ) ) {
        if ( downKeys.has( ev.keyCode ) ) {
          let ival = downKeys.get( ev.keyCode );
          clearInterval( ival );
          downKeys.delete( ev.keyCode );
        }
        ev.preventDefault( );
      }
    } );
  } );
}
