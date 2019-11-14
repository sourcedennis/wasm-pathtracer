import { RaytraceTarget } from './offscreen_target';
import { Rect4 } from './rect4';
import { Observable, XObservable } from './observable';

export interface BlockRenderer {
  setScene( viewportWidth : number, viewportHeight : number ): Promise< void >;
  renderBlock( x : number, y : number, width : number, height : number, antiAlias : number ): Promise< Uint8Array >;
  terminate( ): void;
}

interface Event {
  readonly type : string;
}

export interface EventProgress {
  readonly type     : 'progress';
  readonly rect     : Rect4;
  readonly numDone  : number;
  readonly numTotal : number;
}

export interface EventQueued {
  readonly type : 'queued';
  readonly rect : Rect4;
}

export interface EventUnqueued {
  readonly type : 'unqueued';
  readonly rect : Rect4;
}

export interface EventDone {
  readonly type     : 'done';
  readonly duration : number;
}

class BlockRendererInstance {
  public readonly renderer    : BlockRenderer;
  public          initPromise : Promise< void >;
  // Keep track of the blocks in progress, in case the renderer is terminated
  // these need to be assigned to other renderers
  public inprogress : Rect4 | undefined;

  public constructor( renderer : BlockRenderer ) {
    this.renderer    = renderer;
    this.initPromise = Promise.resolve( );
    this.inprogress  = undefined;
  }
}

export class RenderConfig {
  public readonly blockSize : number;
  public readonly width     : number;
  public readonly height    : number;
  public readonly antiAlias : number;
  public readonly isBandless : boolean;

  public constructor( blockSize : number
                    , width : number
                    , height : number
                    , antiAlias : number
                    , isBandless : boolean ) {
    this.blockSize = blockSize;
    this.width     = width;
    this.height    = height;
    this.antiAlias = antiAlias;
    this.isBandless = isBandless;
  }
}

export class RenderManager {
  private          _target       : RaytraceTarget | undefined;
  private readonly _fNewRenderer : ( ) => BlockRenderer;
  private          _renderers    : BlockRendererInstance[];
  private          _todos        : Rect4[];
  private readonly _obsProgress  : XObservable< EventProgress >;
  private readonly _obsQueued    : XObservable< EventQueued >;
  private readonly _obsUnqueued  : XObservable< EventUnqueued >;
  private readonly _obsDone      : XObservable< EventDone >;
  private          _numTotalJobs : number;
  private          _numDoneJobs  : number;
  private          _antiAlias    : number;
  private          _startTime    : number;

  public constructor( fNewRenderer: ( ) => BlockRenderer, numCores : number ) {
    this._target       = undefined;
    this._fNewRenderer = fNewRenderer;
    this._renderers    = [];
    this._todos        = [];
    this._obsProgress  = new XObservable( );
    this._obsQueued    = new XObservable( );
    this._obsUnqueued  = new XObservable( );
    this._obsDone      = new XObservable( );
    this._numTotalJobs = 0;
    this._numDoneJobs  = 0;
    this._antiAlias    = 1;
    this._startTime    = 0;

    for ( let i = 0; i < numCores; i++ ) {
      this._renderers.push( new BlockRendererInstance( fNewRenderer( ) ) );
    }
  }

  public setNumCores( c : number ): void {
    if ( c < this._renderers.length ) {
      // Remove existing renderers
      while ( this._renderers.length > c ) {
        let r = < BlockRendererInstance > this._renderers.shift( );
        if ( r.inprogress ) {
          this._todos.push( r.inprogress );
          this._obsUnqueued.next( { type: 'unqueued', rect: r.inprogress } );
        }
        r.renderer.terminate( );
      }
    } else if ( c > this._renderers.length ) {
      let target = < RaytraceTarget > this._target;
      // Add new renderers
      while ( this._renderers.length < c ) {
        let r = new BlockRendererInstance( this._fNewRenderer( ) );
        this._renderers.push( r );
        r.initPromise = r.renderer.setScene( target.width, target.height ); // TODO
      }
      this._enqueueAll( );
    }
  }

  public on( ev : 'progress' ): Observable< EventProgress >;
  public on( ev : 'queued' ):   Observable< EventQueued >;
  // Unqueueing happens when a renderer is disposed of before finishing the job
  // The job is then unqueued
  public on( ev : 'unqueued' ): Observable< EventQueued >;
  public on( ev : 'done' ): Observable< EventDone >;

  public on( ev : string ): Observable< Event > {
    if ( ev === 'progress' ) {
      return this._obsProgress.observable;
    } else if ( ev === 'queued' ) {
      return this._obsQueued.observable;
    } else if ( ev === 'unqueued' ) {
      return this._obsUnqueued.observable;
    } else if ( ev === 'done' ) {
      return this._obsDone.observable;
    } else {
      return new XObservable< Event >( ).observable;
    }
  }

  public get target( ): RaytraceTarget | undefined {
    return this._target;
  }

  // Terminates another running render job
  public start( config : RenderConfig ): void {
    this._todos = [];

    let numX = Math.ceil( config.width / config.blockSize );
    let numY = Math.ceil( config.height / config.blockSize );

    if ( this._numDoneJobs < this._numTotalJobs ) {
      let numRenderers = this._renderers.length;
      for ( let i = 0; i < numRenderers; i++ ) {
        let r = < BlockRendererInstance > this._renderers.shift( );
        r.renderer.terminate( );
      }
      for ( let i = 0; i < numRenderers; i++ ) {
        let r = new BlockRendererInstance( this._fNewRenderer( ) );
        this._renderers.push( r );
        r.initPromise = r.renderer.setScene( config.width, config.height ); // TODO
      }
    } else {
      for ( let r of this._renderers ) {
        r.inprogress = undefined;
        r.initPromise = r.renderer.setScene( config.width, config.height ); // TODO
      }
    }

    for ( let y = 0; y < numY; y++ ) {
      for ( let x = 0; x < numX; x++ ) {
        let xSize = Math.min( config.width  - x * config.blockSize, config.blockSize );
        let ySize = Math.min( config.height - y * config.blockSize, config.blockSize );
        this._todos.push( new Rect4( x * config.blockSize, y * config.blockSize, xSize, ySize ) );
      }
    }

    shuffle( this._todos );

    this._numTotalJobs = numX * numY;
    this._numDoneJobs  = 0;
    this._target       = new RaytraceTarget( config.width, config.height, config.isBandless );
    this._antiAlias    = config.antiAlias;

    this._startTime    = Date.now( );
    this._enqueueAll( );
  }

  private _enqueueAll( ): void {
    // Specifically cache the target here
    // If the target changes while computing the result, it will not write to the new target
    let target = <RaytraceTarget> this._target;

    for ( let r of this._renderers ) {
      if ( !r.inprogress && this._todos.length > 0 ) {
        let job = < Rect4 > this._todos.shift( );
        r.inprogress = job;
        this._obsQueued.next( { type: 'queued', rect: job } );
        let pResult = r.initPromise.then( ( ) => {
          return r.renderer.renderBlock( job.x, job.y, job.width, job.height, this._antiAlias );
        } );
        pResult.then( res => {
          if ( r.inprogress === job ) {
            r.inprogress = undefined;
            this._numDoneJobs++;
            target.addRect( job.x, job.y, job.width, job.height, res );
            this._obsProgress.next( { type: 'progress', rect: job, numDone: this._numDoneJobs, numTotal: this._numTotalJobs } );

            if ( this._numDoneJobs === this._numTotalJobs ) {
              this._obsDone.next( { type: 'done', duration: Date.now( ) - this._startTime } );
            }
          }
          // Possible this renderer has been disposed of
          // Call `_enqueueAll()` to make sure this isn't the case
          this._enqueueAll( );
        } );
      }
    }
  }
}

function shuffle< T >( xs : T[] ): void {
  for ( let i = 0; i < xs.length; i++ ) {
    let newI = Math.floor( Math.random( ) * xs.length );
    let tmp = xs[ i ];
    xs[ i ] = xs[ newI ];
    xs[ newI ] = tmp;
  }
}
