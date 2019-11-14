port module Main exposing (main)

import Browser
import Html exposing (Html, Attribute, h2, hr, br, div, text, span, button)
import Html.Attributes exposing (class, id, style)
import Html.Events exposing (onClick)

port setAntiAlias : Int -> Cmd msg
port setBlockSize : Int -> Cmd msg
port setBanding   : Bool -> Cmd msg
port setNumCores  : Int -> Cmd msg

type AntiAlias
  = AA1 | AA2 | AA4 | AA8

type BlockSize = BS64 | BS128 | BS256 | BS512

type CoreCount = CC1 | CC2 | CC3 | CC4

type alias Model =
  { antiAlias        : AntiAlias
  , blockSize        : BlockSize
  , isBandingRemoved : Bool
  , numCores         : CoreCount
  }

type Msg
  = SelectAntiAlias AntiAlias
  | SelectBlockSize BlockSize
  | SelectBandingRem Bool
  | SelectCoreCount CoreCount

main =
  Browser.element
    { init = \() -> ( init, Cmd.none )
    , update = update
    , view = view
    , subscriptions = \_ -> Sub.none
    }

init : Model
init =
  { antiAlias        = AA1
  , blockSize        = BS128
  , isBandingRemoved = False
  , numCores         = CC1
  }

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
  case msg of
    SelectAntiAlias aa ->
      let aaInt =
            case aa of
              AA1 -> 1
              AA2 -> 2
              AA4 -> 4
              AA8 -> 8
      in
      ( { model | antiAlias = aa }, setAntiAlias aaInt )
    SelectBlockSize bs ->
      let bsInt =
            case bs of
              BS64  -> 64
              BS128 -> 128
              BS256 -> 256
              BS512 -> 512
      in
      ( { model | blockSize = bs }, setBlockSize bsInt )
    SelectBandingRem br ->
      ( { model | isBandingRemoved = br }, setBanding br )
    SelectCoreCount cc ->
      let ccInt =
            case cc of
              CC1 -> 1
              CC2 -> 2
              CC3 -> 3
              CC4 -> 4
      in
      ( { model | numCores = cc }, setNumCores ccInt )

view : Model -> Html Msg
view m =
  div [ id "sidepanel" ]
    [ h2 [] [ text "Settings" ]
    , hr [] []
    , div []
        [ span [] [ text "Anti-Aliassing (restarts render)" ]
        , buttonC ( m.antiAlias == AA1 ) (SelectAntiAlias AA1)
            [ class "choice", class "top", class "left" ]
            [ text "None" ]
        , buttonC ( m.antiAlias == AA2 ) (SelectAntiAlias AA2)
            [ class "choice", class "top", class "right" ]
            [ text "2x2" ]
        , br [] []
        , buttonC ( m.antiAlias == AA4 ) (SelectAntiAlias AA4)
            [ class "choice", class "bottom", class "left" ]
            [ text "4x4" ]
        , buttonC ( m.antiAlias == AA8 ) (SelectAntiAlias AA8)
            [ class "choice", class "bottom", class "right" ]
            [ text "8x8" ]
        ]
    , div []
        [ span [] [ text "Block size (restarts render)" ]
        , buttonC ( m.blockSize == BS64 ) (SelectBlockSize BS64)
            [ class "choice", class "top", class "left" ]
            [ text "64" ]
        , buttonC ( m.blockSize == BS128 ) (SelectBlockSize BS128)
            [ class "choice", class "top", class "right" ]
            [ text "128" ]
        , br [] []
        , buttonC ( m.blockSize == BS256 ) (SelectBlockSize BS256)
            [ class "choice", class "bottom", class "left" ]
            [ text "256" ]
        , buttonC ( m.blockSize == BS512 ) (SelectBlockSize BS512)
            [ class "choice", class "bottom", class "right" ]
            [ text "512" ]
        ]
    , div []
        [ span [] [ text "Banding Removal" ]
        , buttonC m.isBandingRemoved (SelectBandingRem True)
            [ class "choice", class "left", style "width" "80pt" ]
            [ text "Enabled" ]
        , buttonC (not m.isBandingRemoved) (SelectBandingRem False)
            [ class "choice", class "right", style "width" "80pt" ]
            [ text "Disabled" ]
        ]
    , div []
        [ span [] [ text "Processing Cores" ]
        , buttonC ( m.numCores == CC1 ) (SelectCoreCount CC1)
            [ class "choice", class "left", style "width" "50pt" ]
            [ text "1" ]
        , buttonC ( m.numCores == CC2 ) (SelectCoreCount CC2)
            [ class "choice", class "middle", style "width" "50pt" ]
            [ text "2" ]
        , buttonC ( m.numCores == CC3 ) (SelectCoreCount CC3)
            [ class "choice", class "middle", style "width" "50pt" ]
            [ text "3" ]
        , buttonC ( m.numCores == CC4 ) (SelectCoreCount CC4)
            [ class "choice", class "right", style "width" "50pt" ]
            [ text "4" ]
        ]
    ]

buttonC : Bool -> msg -> List (Attribute msg) -> List (Html msg) -> Html msg
buttonC b m attrs cs =
  if b then
    button ( class "selected" :: attrs ) cs
  else
    button ( onClick m :: style "cursor" "pointer" :: attrs ) cs
