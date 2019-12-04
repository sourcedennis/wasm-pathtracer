import { Camera }                  from '@s/graphics/camera';
import { XObservable, Observable } from '@s/event/observable';
import { Vec3 }                    from '@s/math/vec3';
import { keyTicker }               from './input_key';

const KeyCode = {
  PAGE_UP:   33,
  PAGE_DOWN: 34,
  LEFT:      37,
  UP:        38,
  RIGHT:     39,
  DOWN:      40,
  W:         87,
  D:         68,
  S:         83,
  A:         65
};

// Provides controls for controlling the camera within the scene globally.
// The following keys are listened to:
// * WASD            - Moving the camera across the plane
// * PageUp/PageDown - Moving the camera up and down
// * Arrow Keys      - Rotating the camera
export class CameraController {
  // Gets notified whenever the Camera changes
  private readonly _onUpdate : XObservable< Camera >;
  // The current camera
  private          _camera : Camera;

  public constructor( camera : Camera ) {
    this._onUpdate = new XObservable( );
    this._camera   = camera;

    let keys = new Set(
      [ KeyCode.PAGE_UP
      , KeyCode.PAGE_DOWN
      , KeyCode.LEFT
      , KeyCode.UP
      , KeyCode.RIGHT
      , KeyCode.DOWN
      , KeyCode.W
      , KeyCode.D
      , KeyCode.S
      , KeyCode.A
      ] );

    keyTicker( keys ).subscribe( ev => {
      let translation: Vec3 | null = null;
      switch ( ev.keyCode ) {
      case KeyCode.W: // Move forward
        translation = new Vec3( 0, 0, 0.03 * ev.count );
        break;
      case KeyCode.D: // Move right
        translation = new Vec3( 0.03 * ev.count, 0, 0 );
        break;
      case KeyCode.S: // Move backward
        translation = new Vec3( 0, 0, -0.03 * ev.count );
        break;
      case KeyCode.A: // Move left
        translation = new Vec3( -0.03 * ev.count, 0, 0 );
        break;
      case KeyCode.LEFT: // Rotate left
        this.rotY -= 0.001 * ev.count * Math.PI;
        break;
      case KeyCode.UP: // Rotate down
        this.rotX -= 0.001 * ev.count * Math.PI;
        break;
      case KeyCode.RIGHT: // Rotate right
        this.rotY += 0.001 * ev.count * Math.PI;
        break;
      case KeyCode.DOWN: // Rotate up
        this.rotX += 0.001 * ev.count * Math.PI;
        break;
      case KeyCode.PAGE_UP: // Move the camera up
        translation = new Vec3( 0, 0.03 * ev.count, 0 );
        break;
      case KeyCode.PAGE_DOWN: // Move the camera down
        translation = new Vec3( 0, -0.03 * ev.count, 0 );
        break;
      }

      if ( translation != null ) {
        translation = translation.rotX( this.rotX ).rotY( this.rotY );
        let c = this._camera;
        this._camera = new Camera( c.location.add( translation ), c.rotX, c.rotY );
        this._onUpdate.next( this._camera );
      }
    } );
  }

  // Obtains the current camera
  public get( ): Camera {
    return this._camera;
  }

  // Gets notified whenever the camera changes
  public onUpdate( ): Observable< Camera > { return this._onUpdate.observable; }

  public get x( ): number { return this._camera.location.x; }
  public get y( ): number { return this._camera.location.y; }
  public get z( ): number { return this._camera.location.z; }
  public get rotX( ): number { return this._camera.rotX; }
  public get rotY( ): number { return this._camera.rotY; }

  // Updates the current camera
  public set( c : Camera ) {
    this._camera = c;
    this._onUpdate.next( this._camera );
  }

  public set x( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setX( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set y( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setY( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set z( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location.setZ( v ), c.rotX, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set rotX( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location, v, c.rotY );
    this._onUpdate.next( this._camera );
  }

  public set rotY( v : number ) {
    let c = this._camera;
    this._camera = new Camera( c.location, c.rotX, v );
    this._onUpdate.next( this._camera );
  }
}
