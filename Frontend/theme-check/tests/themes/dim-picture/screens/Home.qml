import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// Home's places as rows, and a caption in the text token over a pale picture: legible in the light variant, near 1:1 in the dark.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
    NormalizedCanvas {
        Image {
            objectName: "sand"
            x: 640
            y: 300
            width: 480
            height: 240
            source: "../pictures/sand.png"
        }
        // Words not yet given draw nothing, so nothing is judged behind them.
        Text {
            objectName: "unwritten"
            x: 672
            y: 460
            width: 200
            height: 40
            text: ""
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 26
        }
        Text {
            objectName: "caption"
            x: 672
            y: 400
            text: Words.title
            color: Tokens.text
            font.family: "Barlow"
            font.pixelSize: 26
        }
    }
}
