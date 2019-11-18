port module Main exposing (main)

import Browser
import Html exposing (Html, Attribute, h2, hr, br, div, text, span, button)
import Html.Attributes exposing (class, id, style)
import Html.Events exposing (onClick)
import String exposing (fromInt, fromFloat)

port callRestart : () -> Cmd msg
port updateRenderType : Int -> Cmd msg
port updateReflectionDepth : Int -> Cmd msg
port updateProgress : (Int -> msg) -> Sub msg
port doneProgress : (Float -> msg) -> Sub msg

type Progress = ProgressPct Int | ProgressDone Float

type alias Model =
  { renderType      : RenderType
  , progress        : Progress
  , reflectionDepth : Int
  }

type RenderType = RenderColor | RenderDepth

type Msg
  = UpdateProgress Int
  | DoneProgress Float
  | SelectType RenderType
  | SelectReflectionDepth Int
  | ClickRestart

main =
  Browser.element
    { init = \() -> ( init, Cmd.none )
    , update = update
    , view = view
    , subscriptions = subscriptions
    }

subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.batch [
      updateProgress UpdateProgress
    , doneProgress DoneProgress
    ]

init : Model
init =
  { renderType      = RenderColor
  , progress        = ProgressPct 0
  , reflectionDepth = 1
  }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    UpdateProgress p ->
      ( { model | progress = ProgressPct p }, Cmd.none )
    DoneProgress time ->
      ( { model | progress = ProgressDone time }, Cmd.none )
    SelectType t ->
      let rtInt =
            case t of
              RenderColor -> 0
              RenderDepth -> 1
      in
      ( { model | renderType = t }, updateRenderType rtInt )
    SelectReflectionDepth t ->
      ( { model | reflectionDepth = t }, updateReflectionDepth t )
    ClickRestart ->
      ( model, callRestart () )


view : Model -> Html Msg
view m =
  div [ id "sidepanel" ]
    [ h2 [] [ text "Settings" ]
    , hr [] []
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
        [ span [] [ text "Reflection depth" ]
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
            [ class "choice", class "bottom left", style "width" "40pt", style "border-left" "none", style "border-top" "1px solid white" ]
            [ text "3" ]
        , buttonC (m.reflectionDepth == 4) (SelectReflectionDepth 4)
            [ class "choice", class "middle", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "4" ]
        , buttonC (m.reflectionDepth == 5) (SelectReflectionDepth 5)
            [ class "choice", class "bottom right", style "width" "40pt", style "border-top" "1px solid white" ]
            [ text "5" ]
        -- , br [] []
        -- , buttonC (m.reflectionDepth == 6) (SelectReflectionDepth 6)
        --     [ class "choice", class "bottom left", style "width" "40pt" ]
        --     [ text "6" ]
        -- , buttonC (m.reflectionDepth == 7) (SelectReflectionDepth 7)
        --     [ class "choice", class "middle", style "width" "40pt" ]
        --     [ text "7" ]
        -- , buttonC (m.reflectionDepth == 8) (SelectReflectionDepth 8)
        --     [ class "choice", class "bottom right", style "width" "40pt" ]
        --     [ text "8" ]
        ]
    , div []
        ( span [] [ text "Progress" ] ::
          ( case m.progress of
            ProgressPct p ->
              span [] [ text ( fromInt p ++ "%" ) ]
            ProgressDone t ->
              span [] [ text ( "Time: " ++ fromFloat t ++ " seconds" ) ] )
          :: [ button [ onClick ClickRestart ] [ text "Restart" ] ]
        )
    ]

buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
