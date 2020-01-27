port module PanelScenes exposing (main)

import Browser
import Html            exposing
  (Html, h2, hr, div, text, span)
import Html.Attributes exposing (class, id, style)
import Html.Events     exposing (onClick)

-- This is the GUI sidepanel with which runtime parameters of the raytracer
-- can be set. The TypeScript instance listens to the ports provided by this
-- module.

-- The ports. Note that only primitive types can be passed across
-- So, some "magic number" (for RenderType and Scene) are passed to TypeScript
-- Outgoing ports
port updateScene           : Int -> Cmd msg

-- The state of the side panel
type alias Model = Scene

-- Identifiers for the hard-coded scenes
type Scene
  = SceneLights
  | SceneBunny

type Msg
  = SelectScene Scene

main : Program () Model Msg
main =
  Browser.element
    { init = \() -> ( init, Cmd.none )
    , update = update
    , view = view
    , subscriptions = subscriptions
    }

-- "Magic numbers" for scene ids
sceneId : Scene -> Int
sceneId s =
  case s of
    SceneLights -> 0
    SceneBunny  -> 2

subscriptions : Model -> Sub Msg
subscriptions _ = Sub.none

init : Model
init = SceneLights

update : Msg -> Model -> (Model, Cmd Msg)
update msg _ =
  case msg of
    SelectScene s ->
      ( s, updateScene (sceneId s) )

view : Model -> Html Msg
view m =
  div [ class "sidepanel", id "scenepanel" ]
    [ h2 [] [ text "Scene" ]
    , hr [ style "width" "100%" ] []
    , div [ style "overflow-y" "auto", style "overflow-x" "hidden", style "width" "225pt" ]
        [ sceneC (m == SceneLights) (SelectScene SceneLights) "Many Lights"  "images/banners/lights.png"
        , sceneC (m == SceneBunny)  (SelectScene SceneBunny)  "Bunny"        "images/banners/bunny_high.png"
        ]
    ]

-- A checkbox button. It's checked if the provided boolean is true
-- Only unchecked button have the event assigned
sceneC : Bool -> msg -> String -> String -> Html msg
sceneC b m txt url =
  if b then
    div
      [ class "scene-option", class "selected", style "background-image" ("url('"++url++"')") ]
      [ span [] [ text txt ] ]
  else
    div
      [ class "scene-option", onClick m, style "cursor" "pointer", style "background-image" ("url('"++url++"')") ]
      [ span [] [ text txt ] ]
