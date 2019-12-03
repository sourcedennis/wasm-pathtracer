// Note by Dennis:
// I've been using this file since forever for TypeScript projects
// It's a very minimal clone/copy/imitation of RxJS's concepts for
// defining declarative dataflows in JavaScript; particularly for events.
// RxJS: https://github.com/ReactiveX/rxjs

type OperatorFunction< A, B > = ( input: Observable< A > ) => Observable< B >;

type Maybe< T > = T | undefined;

export class Observable< T > {
  private readonly _fSubscriber: SubscriberFunction< T >;

  public constructor( subscriber : SubscriberFunction< T > ) {
    this._fSubscriber = subscriber;
  }

  // Subscribes to the sequence with an observer
  public subscribe( observer : Observer< T > ): Subscription;

  // Subscribes to the sequence with callbacks
  public subscribe( onNext: ( value: T ) => void,
                    onError?: ( errorValue: any ) => void,
                    onComplete?: ( ) => void ): Subscription;

  // Subscribes to the sequence with callbacks
  public subscribe( onNextOrObserver: ( ( value: T ) => void ) | Observer< T >,
                    onError?: ( errorValue: any ) => void,
                    onComplete?: ( ) => void ): Subscription {
    let observer: Observer< T >;

    if ( typeof onNextOrObserver === 'object' ) {
      observer = < Observer< T > > onNextOrObserver;
    } else if ( typeof onNextOrObserver === 'function' ) {
      let onNext = < ( value: T ) => void > onNextOrObserver;
      observer = { next: onNext, error: onError, complete: onComplete };
    } else {
      observer = { };
    }

    return new SubscriptionImpl( observer, this._fSubscriber );
  }

  public pipe< A >( func: OperatorFunction< T, A > ): Observable< A >;
  public pipe< T2, T3 >(
      func1: OperatorFunction< T, T2 >,
      func2: OperatorFunction< T2, T3 > ): Observable< T3 >;
  public pipe< T2, T3, T4 >(
      func1: OperatorFunction< T, T2 >,
      func2: OperatorFunction< T2, T3 >,
      func3: OperatorFunction< T3, T4 > ): Observable< T4 >;
  public pipe< T2, T3, T4, T5 >(
      func1: OperatorFunction< T, T2 >,
      func2: OperatorFunction< T2, T3 >,
      func3: OperatorFunction< T3, T4 >,
      func4: OperatorFunction< T4, T5 > ): Observable< T5 >;
  public pipe< T2, T3, T4, T5, T6 >(
      func1: OperatorFunction< T, T2 >,
      func2: OperatorFunction< T2, T3 >,
      func3: OperatorFunction< T3, T4 >,
      func4: OperatorFunction< T4, T5 >,
      func5: OperatorFunction< T5, T6 > ): Observable< T6 >;
  public pipe< T2, T3, T4, T5, T6, T7 >(
      func1: OperatorFunction< T, T2 >,
      func2: OperatorFunction< T2, T3 >,
      func3: OperatorFunction< T3, T4 >,
      func4: OperatorFunction< T4, T5 >,
      func5: OperatorFunction< T5, T6 >,
      func6: OperatorFunction< T6, T7 > ): Observable< T7 >;

  public pipe( ...funcs: ( ( source: Observable< any > ) => Observable< any > )[] ): Observable< any > {
    return funcs.reduce( ( prev, fn ) => fn( prev ), this );
  }

  // Converts items to an Observable
  public static of< T >(...items) : Observable< T > {
    return new Observable( observer => {
      for ( let i = 0; i < items.length; i++ ) {
        observer.next( items[ i ] );

        if ( observer.isClosed )
          return;
      }
      observer.complete( );
    } );
  }

  // Converts an observable or iterable to an Observable
  public static from< T >(observable) : Observable< T > {
    if ( observable instanceof Observable ) {
      return new Observable( observer => observable.subscribe( observer ) );
    } else if ( observable[Symbol.iterator] ) {
      let it = observable[Symbol.iterator];

      return new Observable( observer => {
        for ( let item of it( ) ) {
          observer.next( item );
  
          if ( observer.isClosed )
            return;
        }
        observer.complete( );
      } );
    } else {
      throw 'NotObservableException';
    }
  }
}

export interface Subscription {
  // Cancels the subscription
  unsubscribe() : void;

  // A boolean value indicating whether the subscription is closed
  isClosed: boolean;
}

export type SubscriberFunction< T > = ( ( observer: SubscriptionObserver< T > ) => ( ( ) => void ) | Subscription | void );

export interface SubscriptionObserver< T > {
  // Sends the next value in the sequence
  next( value?: T ): void;

  // Sends the sequence error
  error( errorValue: any ): void;

  // Sends the completion notification
  complete( ): void;

  // A boolean value indicating whether the subscription is closed
  isClosed: boolean;
}

export interface Observer< T > {
  // Receives the subscription object when `subscribe` is called
  start?( subscription : Subscription );

  // Receives the next value in the sequence
  next?( value: T ): void

  // Receives the sequence error
  error?( errorValue: any ): void;

  // Receives a completion notification
  complete?( ): void;
}

class SubscriptionImpl< T > implements Subscription {
  public _observer: Maybe< Observer< T > >;
  private _fCleanup: Maybe< ( ) => void >;

  public constructor( observer: Observer< T >, subscriber: SubscriberFunction< T > ) {
    this._observer = observer;

    if ( observer.start ) {
      observer.start( this );
    }

    // It was closed by the start function
    if ( this.isClosed )
      return;

    let sObserver = new SubscriptionObserverImpl( this );

    try {
      let fCleanup = subscriber( sObserver );

      if ( fCleanup ) {
        if ( typeof fCleanup === 'object' ) {
          let subscription = < Subscription > fCleanup;
          fCleanup = ( ( ) => subscription.unsubscribe( ) );
        }
        this._fCleanup = fCleanup;
      }
    } catch ( ex ) {
      sObserver.error( ex );
      return;
    }

    if ( this.isClosed ) {
      this.cleanup( );
    }
  }

  // Cancels the subscription
  public unsubscribe() : void {
    this.close( );
  }

  // A boolean value indicating whether the subscription is closed
  public get isClosed( ): boolean {
    return !Boolean( this._observer );
  }

  public cleanup( ): void {
    if ( !this._fCleanup )
      return;
    let fCleanup = this._fCleanup;
    this._fCleanup = undefined;
    fCleanup( );
  }

  public close( ): void {
    if ( this.isClosed )
      return;
    
    this._observer = undefined;
    this.cleanup( );
  }
}

class SubscriptionObserverImpl< T > implements SubscriptionObserver< T > {
  private readonly _subscription: SubscriptionImpl< T >;

  public constructor( subscription: SubscriptionImpl< T > ) {
    this._subscription = subscription;
  }

  // Sends the next value in the sequence
  public next( value: T ): void {
    if ( this._subscription.isClosed )
      return;
    
    let observer = < Observer< T > > this._subscription._observer;
    
    if ( !observer.next )
      return;
    
    observer.next( value );
  }

  // Sends the sequence error
  public error( errorValue: any ): void {
    if ( this._subscription.isClosed )
      return;
    
    let observer = < Observer< T > > this._subscription._observer;
    
    if ( !observer.error )
      return;
    
    observer.error( errorValue );

    this._subscription.cleanup( );
  }

  // Sends the completion notification
  public complete( ) {
    if ( this._subscription.isClosed )
      return;
    
    let observer = < Observer< T > > this._subscription._observer;
    
    if ( !observer.complete )
      return;
    
    observer.complete( );
    
    this._subscription.cleanup( );
  }

  // A boolean value indicating whether the subscription is closed
  public get isClosed( ): boolean {
    return this._subscription.isClosed;
  }
}

export class XObservable< T > {
  public readonly observable: Observable< T >;
  private readonly _subscribers: Set< SubscriptionObserver< T > >;

  public constructor( ) {
    this._subscribers = new Set( );

    this.observable = new Observable( subscriber => {
      this._subscribers.add( subscriber );

      return ( ) => {
        this._subscribers.delete( subscriber );
      };
    } );
  }

  public hasSubscribers( ): boolean {
    return this._subscribers.size > 0;
  }

  public next( val: T ): void {
    for ( let s of this._subscribers ) {
      s.next( val );
    }
  }

  public error( err: any ): void {
    for ( let s of this._subscribers ) {
      s.error( err );
    }
  }

  public complete( ): void {
    for ( let s of this._subscribers ) {
      s.complete( );
    }
  }
}
