// Keeps track of the computation times of the frames in the last second
// Observations older than one second are discarded
export class FpsTracker {
  // This list is maintained as a circular list
  //   Because list allocations 20+ times a second are expensive
  private readonly _observations    : FpsObservation[];
  // The current index in the circular list
  private          _index           : number;
  // The number of actual observations in the circular list
  //
  private          _numMeasurements : number;

  public constructor( ) {
    this._observations    = [];
    this._index           = 0;
    this._numMeasurements = 0;
  }

  // Registers a new measurement of the render-time (in ms) of a single frame
  // `time` is the timestamp (in ms) at which the measurement occurs
  public add( time : number, measurement : number ) {
    // Delete old measurements
    while ( this._numMeasurements > 0 && this._observations[ this._index ].time + 1000 < time ) {
      this._index = ( this._index + 1 ) % this._observations.length;
      this._numMeasurements--;
    }

    // Register a new measurement. Extend the circular list if necessary
    if ( this._numMeasurements < this._observations.length ) {
      let nextIndex = ( this._index + this._numMeasurements ) % this._observations.length;
      this._observations[ nextIndex ].time = time;
      this._observations[ nextIndex ].measurement = measurement;
      this._numMeasurements++;
    } else {
      this._index = 0;
      this._numMeasurements++;
      this._observations.push( new FpsObservation( time, measurement ) );
    }
  }

  // Removes all measurements
  public clear( ) {
    this._numMeasurements = 0;
  }

  // The lowest value of all render-times within the last second
  public low( ): number {
    let l = Number.POSITIVE_INFINITY;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      l = Math.min( l, this._observations[ ( this._index + i ) % this._observations.length ].measurement );
    }
    return l;
  }

  // The highest value of all render-times within the last second
  public high( ): number {
    let h = 0;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      h = Math.max( h, this._observations[ ( this._index + i ) % this._observations.length ].measurement );
    }
    return h;
  }

  // The average value of all render-times within the last second
  public avg( ): number {
    let sum = 0;
    for ( let i = 0; i < this._numMeasurements; i++ ) {
      sum += this._observations[ ( this._index + i ) % this._observations.length ].measurement;
    }
    return Math.round( sum / this._numMeasurements );
  }
}

// A single observation for the render-time of a frame
class FpsObservation {
  // The time at which it was observed (at the end of rendering the frame)
  public time        : number;
  // The time it took for the frame to render
  public measurement : number;

  public constructor( time : number, measurement : number ) {
    this.time = time;
    this.measurement = measurement;
  }
}
