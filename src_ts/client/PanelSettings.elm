port module PanelSettings exposing (main)

import Browser
import Html            exposing
  (Html, Attribute, h2, hr, br, div, text, span, button, table, tr, td, th, input)
import Html.Attributes as Attr exposing (class, id, style, type_)
import Html.Events     exposing (onClick, onInput, onBlur, on, keyCode)
import String          exposing (fromInt, toInt)
import Maybe           exposing (withDefault, andThen)
import Json.Decode     as D
import String          exposing (length)

-- This is the GUI sidepanel with which runtime parameters of the raytracer
-- can be set. The TypeScript instance listens to the ports provided by this
-- module.

-- The ports. Note that only primitive types can be passed across
-- So, some "magic number" (for RenderType) are passed to TypeScript
-- Outgoing ports
port updateRenderType      : Int -> Cmd msg
port updateHasBVH          : Bool -> Cmd msg
port updateReflectionDepth : Int -> Cmd msg
port updateRunning         : Bool -> Cmd msg
port updateMulticore       : Bool -> Cmd msg
port updateViewport        : (Int, Int) -> Cmd msg
-- Incoming ports
port updatePerformance     : ( (Int, Int, Int) -> msg ) -> Sub msg
-- Number of milliseconds it took to build the BVH
port updateBVHTime         : ( Int -> msg ) -> Sub msg
-- Number of nodes in the BVH
port updateBVHCount        : ( Int -> msg ) -> Sub msg
-- Number of ray hits with BVH nodes in the current frame
port updateBVHHits         : ( Int -> msg ) -> Sub msg

-- The state of the side panel
type alias Model =
  { renderType      : RenderType
  , reflectionDepth : Int
    -- Render time over the last second
  , performanceAvg  : Int
  , performanceMin  : Int
  , performanceMax  : Int

  , bvh             : BVHModel

  , width           : Int
  , height          : Int
    -- The viewport size the application is aware of
    -- Only sent updates if they actually changed,
    -- as viewport resizes are expensive
  , sentWidth       : Int
  , sentHeight      : Int

  , isMulticore     : Bool

  , isRunning       : Bool
  }

type BVHModel
  = BVHModel { time     : Maybe Int
             , numNodes : Maybe Int
             , numHits  : Maybe Int
             }
  | BVHNoModel

type RenderType = RenderColor | RenderDepth | RenderBvh

type Msg
  = UpdatePerformance Int Int Int -- render times: avg min max
  | UpdateBVHTime Int -- construction time in ms
  | UpdateBVHCount Int -- #nodes in the BVH
  | UpdateBVHHits Int  -- #hits with rays to BVHs
  | SelectType RenderType
  | SelectReflectionDepth Int
  | SelectMulticore Bool
  | SelectBVH Bool
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
  Sub.batch
    [ updatePerformance <| \(x,y,z) -> UpdatePerformance x y z
    , updateBVHTime <| UpdateBVHTime
    , updateBVHCount <| UpdateBVHCount
    , updateBVHHits <| UpdateBVHHits
    ]

init : Model
init =
  { renderType      = RenderColor
  , reflectionDepth = 1
  , performanceAvg  = 0
  , performanceMin  = 0
  , performanceMax  = 0
  , bvh             = BVHModel { time = Nothing, numNodes = Nothing, numHits = Nothing }
  , width           = 512
  , height          = 512
  , sentWidth       = 512
  , sentHeight      = 512
  , isMulticore     = True
  , isRunning       = True
  }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    SelectBVH b ->
      let bvh =
            if b then
              BVHModel { time = Nothing, numNodes = Nothing, numHits = Nothing }
            else
              BVHNoModel
      in
      ( { model | bvh = bvh }, updateHasBVH b )
    UpdateBVHTime t ->
      ( { model | bvh = updateBVH msg model.bvh }, Cmd.none )
    UpdateBVHCount h ->
      ( { model | bvh = updateBVH msg model.bvh }, Cmd.none )
    UpdateBVHHits h ->
      ( { model | bvh = updateBVH msg model.bvh }, Cmd.none )

    UpdatePerformance avg low high ->
      ( { model | performanceAvg = avg, performanceMin = low, performanceMax = high }, Cmd.none )
    SelectType t ->
      let rtInt =
            case t of
              RenderColor -> 0
              RenderDepth -> 1
              RenderBvh   -> 2
      in
      ( { model | renderType = t }, updateRenderType rtInt )
    SelectReflectionDepth t ->
      ( { model | reflectionDepth = t }, updateReflectionDepth t )
    SelectMulticore b ->
      ( { model | isMulticore = b }, updateMulticore b )
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


updateBVH : Msg -> BVHModel -> BVHModel
updateBVH msg bvh =
  case bvh of
    BVHModel bm ->
      case msg of
        UpdateBVHTime t ->
          BVHModel { bm | time = Just t }
        UpdateBVHCount c ->
          BVHModel { bm | numNodes = Just c }
        UpdateBVHHits h ->
          BVHModel { bm | numHits = Just h }
        _ ->
          BVHModel bm
    BVHNoModel ->
      BVHNoModel

view : Model -> Html Msg
view m =
  div [ class "sidepanel", id "settingspanel" ]
    [ h2 [] [ text "Settings" ]
    , hr [] []
    , div []
        [ span [] [ text "Render type" ]
        , buttonC (m.renderType == RenderColor) (SelectType RenderColor)
            [ class "choice", class "left", style "width" "70pt" ]
            [ text "Color" ]
        , buttonC (m.renderType == RenderDepth) (SelectType RenderDepth)
            [ class "choice", class "middle", style "width" "70pt" ]
            [ text "Depth" ]
        , buttonC (m.renderType == RenderBvh) (SelectType RenderBvh)
            [ class "choice", class "right", style "width" "70pt" ]
            [ text "BVH" ]
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
        ( [ span [] [ text "BVH" ]
          , buttonC (m.bvh /= BVHNoModel) (SelectBVH True)
              [ class "choice", class "left", style "width" "90pt" ]
              [ text "Enabled" ]
          , buttonC (m.bvh == BVHNoModel) (SelectBVH False)
              [ class "choice", class "right", style "width" "90pt" ]
              [ text "Disabled" ]
          ]
          ++
          case m.bvh of
            BVHNoModel -> []
            BVHModel bm ->
              [ div []
                  [ span []
                      [ text "Build time: "
                      , text (withDefault "- ms" (bm.time |> andThen (\t -> Just (fromInt t ++ " ms")) ))
                      ]
                  ]
              , div []
                  [ span []
                      [ text "Node count: "
                      , text (withDefault "-" (bm.numNodes |> andThen (Just << niceInt)))
                      ]
                  ]
              , div []
                  [ span []
                      [ text "Hit count: "
                      , text (withDefault "-" (bm.numHits |> andThen (Just << niceInt)))
                      ]
                  ]
              ]
        )
    , div []
        [ span [] [ text "Viewport" ]
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

niceInt : Int -> String
niceInt n =
  if n < 1000 then
    fromInt n
  else
    niceInt (n // 1000) ++ "," ++ pad3z (fromInt (modBy 1000 n))

-- Pad zeroes *before* the string, such that it has 3 characters
pad3z : String -> String
pad3z s =
  case length s of
    1 -> "00" ++ s
    2 -> "0" ++ s
    _ -> s

-- A checkbox button. It's checked if the provided boolean is true
-- Only unchecked button have the event assigned
buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
