port module PanelSettings exposing (main)

import Browser
import Html            exposing
  (Html, Attribute, h2, hr, br, div, text, span, button, table, tr, td, input)
import Html.Attributes as Attr exposing (class, id, style, type_)
import Html.Events     exposing (onClick, onInput, onBlur, on, keyCode)
import String          exposing (fromInt, toInt)
import Maybe           exposing (withDefault)
import Json.Decode     as D

-- This is the GUI sidepanel with which runtime parameters of the raytracer
-- can be set. The TypeScript instance listens to the ports provided by this
-- module.

-- The ports. Note that only primitive types can be passed across
-- So, some "magic number" (for RenderType) are passed to TypeScript
-- Outgoing ports
port updateLeftRenderType  : Int -> Cmd msg
port updateRightRenderType : Int -> Cmd msg
port updateLeftAdaptive    : Bool -> Cmd msg
port updateRightAdaptive   : Bool -> Cmd msg
port updateLightDebug      : Bool -> Cmd msg
-- -- If true, show the sampling strategy. If false, show the diffuse buffer
port updateSamplingDebug   : Bool -> Cmd msg
port updateRunning         : Bool -> Cmd msg
port updateViewport        : (Int, Int) -> Cmd msg

-- The state of the side panel
type alias Model =
  { leftRenderType  : RenderType
  , rightRenderType : RenderType

  , isLeftAdaptive  : Bool
  , isRightAdaptive : Bool

  , isLightDebug    : Bool
  , isSamplingDebug : Bool

  , width           : Int
  , height          : Int
    -- The viewport size the application is aware of
    -- Only sent updates if they actually changed,
    -- as viewport resizes are expensive
  , sentWidth       : Int
  , sentHeight      : Int

  , isRunning       : Bool
  }

type RenderType = RenderNoNEE | RenderNEE | RenderPNEE

type Msg
  = SelectLeftType  RenderType
  | SelectRightType RenderType
  | SelectLeftAdaptive Bool
  | SelectRightAdaptive Bool
  | SelectLightDebug Bool
  | SelectSamplingDebug Bool
  | SelectRunning Bool -- Play/Pause (Play=True)
  | ChangeWidth Int
  | ChangeHeight Int
  | SubmitViewport
  | Skip

main : Program () Model Msg
main =
  Browser.element
    { init = \() -> ( init, Cmd.none )
    , update = update
    , view = view
    , subscriptions = subscriptions
    }

subscriptions : Model -> Sub Msg
subscriptions _ =
  Sub.none

init : Model
init =
  { leftRenderType  = RenderNEE
  , rightRenderType = RenderPNEE
  , isLeftAdaptive  = False
  , isRightAdaptive = True
  , isLightDebug    = False
  , isSamplingDebug = False
  , width           = 512
  , height          = 512
  , sentWidth       = 512
  , sentHeight      = 512
  , isRunning       = True
  }

rtInt : RenderType -> Int
rtInt t =
  case t of
    RenderNoNEE -> 0
    RenderNEE   -> 1
    RenderPNEE  -> 2

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    SelectLeftType t ->
      ( { model | leftRenderType = t }, updateLeftRenderType (rtInt t) )
    SelectRightType t ->
      ( { model | rightRenderType = t }, updateRightRenderType (rtInt t) )
    SelectLeftAdaptive b ->
      ( { model | isLeftAdaptive = b }, updateLeftAdaptive b )
    SelectRightAdaptive b ->
      ( { model | isRightAdaptive = b }, updateRightAdaptive b )
    SelectLightDebug b ->
      ( { model | isLightDebug = b }, updateLightDebug b )
    SelectSamplingDebug b ->
      ( { model | isSamplingDebug = b }, updateSamplingDebug b )
    SelectRunning b ->
      ( { model | isRunning = b }, updateRunning b )
    ChangeWidth w ->
      ( { model | width = w }, Cmd.none )
    ChangeHeight h ->
      ( { model | height = h }, Cmd.none )
    SubmitViewport ->
      let w = min 1024 (max model.width 128)
          h = min 1024 (max model.height 128)
      in
      if model.sentWidth /= w || model.sentHeight /= h then
        ( { model | width = w, height = h, sentWidth = w, sentHeight = h }, updateViewport (w, h) )
      else
        ( { model | width = w, height = h }, Cmd.none )
    Skip -> ( model, Cmd.none )

view : Model -> Html Msg
view m =
  div [ class "sidepanel", id "settingspanel" ]
    [ h2 [] [ text "Settings" ]
    , hr [] []
    , span [ style "font-family" "OpenSansLight, Arial", style "text-decoration" "underline" ] [ text "Render Type" ]
    , div []
        [ table []
          [ tr []
            [ td []
                [ span [] [ text "Left" ]
                , buttonC (m.leftRenderType == RenderNoNEE) (SelectLeftType RenderNoNEE)
                    [ class "choice", class "top", style "width" "80pt" ]
                    [ text "No NEE" ]
                , br [] []
                , buttonC (m.leftRenderType == RenderNEE) (SelectLeftType RenderNEE)
                    [ class "choice", class "middle", style "width" "81pt", style "margin-left" "-1px" ]
                    [ text "NEE" ]
                , br [] []
                , buttonC (m.leftRenderType == RenderPNEE) (SelectLeftType RenderPNEE)
                    [ class "choice", class "bottom", style "width" "80pt" ]
                    [ text "PNEE" ]
                ]
            , td [ style "width" "10pt" ] []
            , td []
                [ span [] [ text "Right" ]
                , buttonC (m.rightRenderType == RenderNoNEE) (SelectRightType RenderNoNEE)
                    [ class "choice", class "top", style "width" "80pt" ]
                    [ text "No NEE" ]
                , br [] []
                , buttonC (m.rightRenderType == RenderNEE) (SelectRightType RenderNEE)
                    [ class "choice", class "middle", style "width" "81pt", style "margin-left" "-1px" ]
                    [ text "NEE" ]
                , br [] []
                , buttonC (m.rightRenderType == RenderPNEE) (SelectRightType RenderPNEE)
                    [ class "choice", class "bottom", style "width" "80pt" ]
                    [ text "PNEE" ]
                ]
            ]
          ]
        ]
        
    , span [ style "font-family" "OpenSansLight, Arial", style "text-decoration" "underline" ] [ text "Sampling" ]
    , div []
        [ table []
          [ tr []
            [ td []
                [ span [] [ text "Left" ]
                , buttonC (not m.isLeftAdaptive) (SelectLeftAdaptive False)
                    [ class "choice", class "top", style "width" "80pt" ]
                    [ text "Random" ]
                , br [] []
                , buttonC m.isLeftAdaptive (SelectLeftAdaptive True)
                    [ class "choice", class "bottom", style "width" "80pt" ]
                    [ text "Adaptive" ]
                ]
            , td [ style "width" "10pt" ] []
            , td []
                [ span [] [ text "Right" ]
                , buttonC (not m.isRightAdaptive) (SelectRightAdaptive False)
                    [ class "choice", class "top", style "width" "80pt" ]
                    [ text "Random" ]
                , br [] []
                , buttonC m.isRightAdaptive (SelectRightAdaptive True)
                    [ class "choice", class "bottom", style "width" "80pt" ]
                    [ text "Adaptive" ]
                ]
            ]
          ]
        ]
        
    , span [ style "font-family" "OpenSansLight, Arial", style "text-decoration" "underline" ] [ text "Photon Debug" ]
    , div []
        [ buttonC (not m.isLightDebug) (SelectLightDebug False)
            [ class "choice", class "left", style "width" "80pt" ]
            [ text "PBR" ]
        , buttonC m.isLightDebug (SelectLightDebug True)
            [ class "choice", class "right", style "width" "100pt" ]
            [ text "Photon" ]
        ]

    , span [ style "font-family" "OpenSansLight, Arial", style "text-decoration" "underline" ] [ text "Sampling Debug" ]
    , span [ style "font-family" "OpenSansLight, Arial", style "margin-left" "4pt" ] [ text "(No restart)" ]
    , div []
        [ buttonC (not m.isSamplingDebug) (SelectSamplingDebug False)
            [ class "choice", class "left", style "width" "80pt" ]
            [ text "Diffuse" ]
        , buttonC m.isSamplingDebug (SelectSamplingDebug True)
            [ class "choice", class "right", style "width" "100pt" ]
            [ text "Sampling" ]
        ]

    , div []
        [ span [ style "text-decoration" "underline" ] [ text "Viewport" ]
        , table []
            [ tr []
                [ td [] [ text "width" ]
                , td [] [ text "height" ]
                ]
            , tr []
                [ td [] [ input [ type_ "number", Attr.min "0", Attr.max "1024", style "width" "50pt"
                                , Attr.value <| fromInt m.width
                                , onInput (ChangeWidth << withDefault m.width << toInt)
                                , onBlur SubmitViewport, onEnterDown SubmitViewport
                                ] [ ]
                        ]
                , td [] [ input [ type_ "number", Attr.min "0", Attr.max "1024", style "width" "50pt"
                                , Attr.value <| fromInt m.height, onInput (ChangeHeight << withDefault m.width << toInt)
                                , onBlur SubmitViewport, onEnterDown SubmitViewport
                                ] [ ]
                                ]
                ]
            ]
        ]
    -- , div []
    --     [ span [] [ text "Performance (in last second)" ]
    --     , table []
    --         [ tr [] [ th [] [ text "Average:" ], td [] [ span [] [ text <| fromInt m.performanceAvg ++ " ms" ] ] ]
    --         , tr [] [ th [] [ text "Min:" ], td [] [ span [] [ text <| fromInt m.performanceMin ++ " ms" ] ] ]
    --         , tr [] [ th [] [ text "Max:" ], td [] [ span [] [ text <| fromInt m.performanceMax ++ " ms" ] ] ]
    --         ]
    --     ]
    , if m.isRunning then
        button [ onClick (SelectRunning False) ] [ text "Pause" ]
      else
        button [ onClick (SelectRunning True) ] [ text "Resume" ]
    ]

onEnterDown : Msg -> Attribute Msg
onEnterDown m =
  let
      enterMsg : Int -> Msg
      enterMsg i =
        case i of -- F1 to F4
          13 -> m
          _  -> Skip
  in
  on "keydown" (D.map enterMsg keyCode)

-- A checkbox button. It's checked if the provided boolean is true
-- Only unchecked button have the event assigned
buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
