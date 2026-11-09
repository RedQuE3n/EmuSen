pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A fixture: the help bar outside a frame in the filled set, as a theme's help element draws it, in the four families at ES-DE's measured sizes, and one bar with a theme's own picture for a button (EmuSen_Kit.md §7.28).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: "#000000"
    property int pressedButton: -1
    property int pressedCount: 0
    // ES-DE's help at 1280 x 800 (EmuSen_BigPicture.md §40.1): a 22-pixel disc, 18-pixel capitals, 7 from glyph to label, 12 to the next entry.
    readonly property real glyphScale: 22 / 0.85 / probe.implicitWidth
    Kit.PadGlyph {
        id: probe
        visible: false
    }
    ListModel {
        id: gamelist
        ListElement { button: 0; label: "CHOOSE" }
        ListElement { button: 2; label: "SYSTEM" }
        ListElement { button: 8; label: "JUMP" }
        ListElement { button: 9; label: "JUMP" }
        ListElement { button: 10; label: "FIRST" }
        ListElement { button: 11; label: "LAST" }
        ListElement { button: 4; label: "LAUNCH" }
        ListElement { button: 5; label: "BACK" }
        ListElement { button: 13; label: "FAVORITE" }
        ListElement { button: 7; label: "OPTIONS" }
    }
    Column {
        x: 38
        y: 24
        spacing: 14
        Repeater {
            model: 4
            Kit.HelpBar {
                required property int index
                objectName: "esdeHelp" + index
                model: gamelist
                family: index
                glyphStyle: Kit.GlyphStyles.Filled
                fitted: true
                color: "#202020"
                labelColor: "#ffffff"
                iconColor: "#ffffff"
                entryScale: scene.glyphScale
                labelFont: Qt.font({ pixelSize: 28 / scene.glyphScale })
                glyphGap: 7 / scene.glyphScale
                entrySpacing: 12
                horizontalPadding: 2
                verticalPadding: 4
            }
        }
        Kit.HelpBar {
            objectName: "ownHelp"
            family: Kit.Families.Xbox
            glyphStyle: Kit.GlyphStyles.Filled
            fitted: true
            radius: 8
            color: "#303060"
            labelColor: "#ffffff"
            iconColor: "#ffffff"
            horizontalPadding: 16
            verticalPadding: 8
            onPressed: (button) => {
                scene.pressedButton = button
                scene.pressedCount += 1
            }
            model: ListModel {
                ListElement { button: 4; label: "Play"; icon: ""; enabled: true }
                ListElement { button: 6; label: "Menu"; icon: "qrc:/scenes/button-picture.svg"; enabled: true }
                ListElement { button: 5; label: "Back"; icon: ""; enabled: false }
            }
        }
    }
}
