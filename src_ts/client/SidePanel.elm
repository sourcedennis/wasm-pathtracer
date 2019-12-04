port module SidePanel exposing (main)

import Browser
import Html            exposing
  (Html, Attribute, h2, hr, br, div, text, span, button, table, tr, td, th)
import Html.Attributes exposing (class, id, style)
import Html.Events     exposing (onClick)
import String          exposing (fromInt, fromFloat)
import Round           as R

-- This is the GUI sidepanel with which runtime parameters of the raytracer
-- can be set. The TypeScript instance listens to the ports provided by this
-- module.

-- The ports. Note that only primitive types can be passed across
-- So, some "magic number" (for RenderType and Scene) are passed to TypeScript
-- Outgoing ports
port updateScene           : Int -> Cmd msg
port updateRenderType      : Int -> Cmd msg
port updateReflectionDepth : Int -> Cmd msg
port updateRunning         : Bool -> Cmd msg
port updateMulticore       : Bool -> Cmd msg
-- Incoming ports
-- render times: (avg, min, max)
port updatePerformance     : ( (Int, Int, Int) -> msg ) -> Sub msg
-- camera properties: (x, y, z, rotX, rotY)
port updateCamera          : ( Camera -> msg ) -> Sub msg

-- The state of the side panel
type alias Model =
  { scene           : Scene
  , renderType      : RenderType
  , reflectionDepth : Int
    -- Render time over the last second
  , performanceAvg  : Int
  , performanceMin  : Int
  , performanceMax  : Int

  , camera          : Camera

  , isMulticore     : Bool

  , isRunning       : Bool
  }

-- Identifiers for the hard-coded scenes
type Scene
  = SceneCubeAndSpheres
  | SceneSimpleBall
  | SceneAirHole
  | SceneMesh
  | SceneTexture

type RenderType = RenderColor | RenderDepth

type Msg
  = UpdatePerformance Int Int Int -- render times: avg min max
  | UpdateCamera Camera
  | SelectScene Scene
  | SelectType RenderType
  | SelectReflectionDepth Int
  | SelectMulticore Bool
  | SelectRunning Bool -- Play/Pause (Play=True)

type alias Camera =
  { x    : Float
  , y    : Float
  , z    : Float
  , rotX : Float
  , rotY : Float
  }

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
    SceneCubeAndSpheres -> 0
    SceneSimpleBall     -> 1
    SceneAirHole        -> 2
    SceneMesh           -> 3
    SceneTexture        -> 4
    
subscriptions : Model -> Sub Msg
subscriptions _ =
  Sub.batch
    [ updatePerformance <| \(x,y,z) -> UpdatePerformance x y z
    , updateCamera <| \x -> UpdateCamera x
    ]

init : Model
init =
  { scene           = SceneCubeAndSpheres
  , renderType      = RenderColor
  , reflectionDepth = 1
  , performanceAvg  = 0
  , performanceMin  = 0
  , performanceMax  = 0
  , camera          = { x = 0, y = 0, z = 0, rotX = 0, rotY = 0 }
  , isMulticore     = False
  , isRunning       = True
  }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    UpdatePerformance avg low high ->
      ( { model | performanceAvg = avg, performanceMin = low, performanceMax = high }, Cmd.none )
    UpdateCamera c ->
      ( { model | camera = c }, Cmd.none )
    SelectScene s ->
      ( { model | scene = s }, updateScene (sceneId s) )
    SelectType t ->
      let rtInt =
            case t of
              RenderColor -> 0
              RenderDepth -> 1
      in
      ( { model | renderType = t }, updateRenderType rtInt )
    SelectReflectionDepth t ->
      ( { model | reflectionDepth = t }, updateReflectionDepth t )
    SelectMulticore b ->
      ( { model | isMulticore = b }, updateMulticore b )
    SelectRunning b ->
      ( { model | isRunning = b }, updateRunning b )


view : Model -> Html Msg
view m =
  div [ id "sidepanel" ]
    [ h2 [] [ text "Settings" ]
    , hr [] []
    , div []
        [ span [] [ text "Scene" ]
        , buttonC (m.scene == SceneCubeAndSpheres) (SelectScene SceneCubeAndSpheres)
            [ class "choice", class "top", style "width" "160pt", style "text-align" "left" ]
            [ text "Cube and spheres" ]
        , buttonC (m.scene == SceneSimpleBall) (SelectScene SceneSimpleBall)
            [ class "choice", class "middle", style "width" "160pt", style "border-left" "none", style "border-top" "1px solid white", style "text-align" "left" ]
            [ text "Simple Ball" ]
        , buttonC (m.scene == SceneAirHole) (SelectScene SceneAirHole)
            [ class "choice", class "middle", style "width" "160pt", style "border-left" "none", style "border-top" "1px solid white", style "text-align" "left" ]
            [ text "Air Hole" ]
        , buttonC (m.scene == SceneMesh) (SelectScene SceneMesh)
            [ class "choice", class "middle", style "width" "160pt", style "border-left" "none", style "border-top" "1px solid white", style "text-align" "left" ]
            [ text ".obj Mesh" ]
        , buttonC (m.scene == SceneTexture) (SelectScene SceneTexture)
            [ class "choice", class "bottom", style "width" "160pt", style "text-align" "left" ]
            [ text "Whitted Turner's Scene" ]
        ]
    , div []
        [ span [] [ text "Render type" ]
        , buttonC (m.renderType == RenderColor) (SelectType RenderColor)
            [ class "choice", class "left", style "width" "80pt" ]
            [ text "Color" ]
        , buttonC (m.renderType == RenderDepth) (SelectType RenderDepth)
            [ class "choice", class "right", style "width" "80pt" ]
            [ text "Depth" ]
        ]
    , div []
        [ span [] [ text "Ray depth" ]
        , buttonC (m.reflectionDepth == 0) (SelectReflectionDepth 0)
            [ class "choice", class "top left", style "width" "40pt" ]
            [ text "0" ]
        , buttonC (m.reflectionDepth == 1) (SelectReflectionDepth 1)
            [ class "choice", class "middle", style "width" "40pt" ]
            [ text "1" ]
        , buttonC (m.reflectionDepth == 2) (SelectReflectionDepth 2)
            [ class "choice", class "top right", style "width" "40pt" ]
            [ text "2" ]
        , br [] []
        , buttonC (m.reflectionDepth == 3) (SelectReflectionDepth 3)
            [ class "choice", class "middle", style "width" "40pt", style "border-left" "none", style "border-top" "1px solid white" ]
            [ text "3" ]
        , buttonC (m.reflectionDepth == 4) (SelectReflectionDepth 4)
            [ class "choice", class "middle", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "4" ]
        , buttonC (m.reflectionDepth == 5) (SelectReflectionDepth 5)
            [ class "choice", class "middle", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "5" ]
        , br [] []
        , buttonC (m.reflectionDepth == 6) (SelectReflectionDepth 6)
            [ class "choice", class "bottom left", style "width" "40pt", style "border-left" "none", style "border-top" "1px solid white" ]
            [ text "6" ]
        , buttonC (m.reflectionDepth == 7) (SelectReflectionDepth 7)
            [ class "choice", class "middle", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "7" ]
        , buttonC (m.reflectionDepth == 8) (SelectReflectionDepth 8)
            [ class "choice", class "bottom right", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "8" ]
        ]
    , div []
        [ span [] [ text "Processor count" ]
        , buttonC (not m.isMulticore) (SelectMulticore False)
            [ class "choice", class "left", style "width" "90pt" ]
            [ text "Single" ]
        , buttonC m.isMulticore (SelectMulticore True)
            [ class "choice", class "right", style "width" "90pt" ]
            [ text "Multi (8)" ]
        ]
    , div []
        [ span [] [ text "Camera" ]
        , table []
            [ tr [] [ th [] [ text "location:" ], td [ style "width" "100pt" ] [ span [] [ text <| showXYZ m.camera.x m.camera.y m.camera.z ] ] ]
            , tr [] [ th [] [ text "rot x:" ], td [ style "text-align" "left", style "padding-left" "8pt" ] [ span [] [ text <| R.round 2 m.camera.rotX ] ] ]
            , tr [] [ th [] [ text "rot y:" ], td [ style "text-align" "left", style "padding-left" "8pt" ] [ span [] [ text <| R.round 2 m.camera.rotY ] ] ]
            ]
        ]
    , div []
        [ span [] [ text "Performance (in last second)" ]
        , table []
            [ tr [] [ th [] [ text "Average:" ], td [] [ span [] [ text <| fromInt m.performanceAvg ++ " ms" ] ] ]
            , tr [] [ th [] [ text "Min:" ], td [] [ span [] [ text <| fromInt m.performanceMin ++ " ms" ] ] ]
            , tr [] [ th [] [ text "Max:" ], td [] [ span [] [ text <| fromInt m.performanceMax ++ " ms" ] ] ]
            ]
        ]
    , if m.isRunning then
        button [ onClick (SelectRunning False) ] [ text "Pause" ]
      else
        button [ onClick (SelectRunning True) ] [ text "Resume" ]
    ]


showXYZ : Float -> Float -> Float -> String
showXYZ x y z = R.round 2 x ++ "; " ++ R.round 2 y ++ "; " ++ R.round 2 z

-- A checkbox button. It's checked if the provided boolean is true
-- Only unchecked button have the event assigned
buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
