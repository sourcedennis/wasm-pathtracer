port module Main exposing (main)

import Browser
import Html exposing (Html, Attribute, h2, hr, br, div, text, span, button, table, tr, td, th)
import Html.Attributes exposing (class, id, style)
import Html.Events exposing (onClick)
import String exposing (fromInt, fromFloat)

-- port callRestart : () -> Cmd msg
port updateRenderType : Int -> Cmd msg
port updateReflectionDepth : Int -> Cmd msg
port updateRunning : Bool -> Cmd msg
port updateMulticore : Bool -> Cmd msg
port updatePerformance : ( (Int, Int, Int) -> msg ) -> Sub msg

type alias Model =
  { renderType      : RenderType
  , reflectionDepth : Int
    -- Render time over the last second
  , performanceAvg  : Int
  , performanceMin  : Int
  , performanceMax  : Int

  , isMulticore     : Bool

  , isRunning       : Bool
  }

type RenderType = RenderColor | RenderDepth

type Msg
  = UpdatePerformance Int Int Int -- render times: avg min max
  | SelectType RenderType
  | SelectReflectionDepth Int
  | SelectMulticore Bool
  | SelectRunning Bool -- Play/Pause (Play=True)

main =
  Browser.element
    { init = \() -> ( init, Cmd.none )
    , update = update
    , view = view
    , subscriptions = subscriptions
    }

subscriptions : Model -> Sub Msg
subscriptions model =
    updatePerformance <| \(x,y,z) -> UpdatePerformance x y z

init : Model
init =
  { renderType      = RenderColor
  , reflectionDepth = 1
  , performanceAvg  = 0
  , performanceMin  = 0
  , performanceMax  = 0
  , isMulticore     = False
  , isRunning       = True
  }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    UpdatePerformance avg low high ->
      ( { model | performanceAvg = avg, performanceMin = low, performanceMax = high }, Cmd.none )
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

buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
