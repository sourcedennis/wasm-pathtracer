import { Msg } from '@s/worker_messages';

export class MsgHandler {
  private readonly _handlers : Map< string, ( v : any ) => void >;

  public constructor( ) {
    this._handlers = new Map( );
  }

  public register< T extends Msg >( id : string, handler : ( v : T ) => void ) {
    this._handlers.set( id, handler );
  }

  public handle( msg : Msg ) {
    if ( this._handlers.has( msg.type ) ) {
      let h = < ( v : any ) => void > this._handlers.get( msg.type );
      h( msg );
    } else {
      console.error( `Handler not found for message: ${msg.type}` );
    }
  }
}
