port module PanelScenes exposing (main)

import Browser
import Html            exposing
  (Html, Attribute, h2, hr, br, div, text, span, button)
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
  = SceneAirHole
  | SceneBunnyLow
  | SceneBunnyHigh
  | SceneCloud100
  | SceneCloud10k
  | SceneCloud100k

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
    SceneAirHole        -> 0
    SceneBunnyLow       -> 1
    SceneBunnyHigh      -> 2
    SceneCloud100       -> 3
    SceneCloud10k       -> 4
    SceneCloud100k      -> 5

subscriptions : Model -> Sub Msg
subscriptions _ = Sub.none

init : Model
init = SceneAirHole

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
    , div [ style "overflow-y" "scroll", style "width" "225pt" ]
        [ sceneC (m == SceneAirHole)   (SelectScene SceneAirHole)   "Air Hole"     "images/banners/air_hole.png"
        , sceneC (m == SceneBunnyLow)  (SelectScene SceneBunnyLow)  "Bunny Low Poly (~5k)" "images/banners/rabbit.png"
        , sceneC (m == SceneBunnyHigh) (SelectScene SceneBunnyHigh) "Bunny High Poly(~144k)" "images/banners/rabbit.png"
        , sceneC (m == SceneCloud100)  (SelectScene SceneCloud100)  "Cloud 100"    "images/banners/cloud100.png"
        , sceneC (m == SceneCloud10k)  (SelectScene SceneCloud10k)  "Cloud 10k"    "images/banners/cloud10k.png"
        , sceneC (m == SceneCloud100k) (SelectScene SceneCloud100k) "Cloud 100k"   "images/banners/cloud100k.png"
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
