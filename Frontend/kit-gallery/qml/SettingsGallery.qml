pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The gallery's settings states: the frame with a picture, a meter or the two variants in its pane, a page of choices and a page of sliders, each from a model the scene holds, as a host would hand them (EmuSen_Kit.md §7.40–§7.42).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: Kit.Tokens.voidFill
    // 0 the frame with a picture, 1 a meter, 2 the two variants, 3 a page of choices, 4 a page of sliders.
    property int shown: 0
    property int chosenRow: -1
    property int steppedRow: -1
    property int steppedBy: 0
    property int slidRow: -1
    property real slidAmount: -1
    property int backs: 0
    readonly property var stateModels: [display, sound, display, engine, crop]
    readonly property var stateTrails: [[], [], [], ["Consoles", "Super Nintendo", "Engine"], ["Consoles", "Super Nintendo", "Custom crop"]]
    readonly property var sectionModels: [controllers, display, sound, controllers, controllers, controllers]
    ListModel {
        id: display
        ListElement { label: "Picture shape"; value: "TV shape"; sentence: "The shape the picture is drawn in."; kind: Kit.RowKinds.Steps }
        ListElement { label: "CRT shader"; value: "Off"; sentence: "Off: the console's pixels as they are, with no television."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Shader levers"; value: ""; sentence: "Every lever of the television, for the console running."; kind: Kit.RowKinds.Opens }
        ListElement { label: "Shaders"; value: ""; sentence: "The shader packs on this machine."; kind: Kit.RowKinds.Opens }
        ListElement { label: "Crop overscan"; value: "Whole picture"; sentence: "What of the picture's edges is hidden."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Text size"; value: "Default"; sentence: "Text size scales every word on the big screen."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Background art on Home"; value: "On"; sentence: "A game's art behind Home."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Reduced motion"; value: "System"; sentence: "Glides jump and fades shorten."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Start-up splash"; value: "On"; sentence: "The splash as the program starts."; kind: Kit.RowKinds.Steps; held: true }
        ListElement { label: "Light or dark"; value: "Dark"; sentence: "Dark is the big screen's default."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Play videos"; value: "On"; sentence: "A game's clip plays where a theme draws one."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Reset the display"; value: ""; sentence: "Every row above back to its default, after a question."; kind: Kit.RowKinds.Asks }
    }
    ListModel {
        id: sound
        ListElement { label: "Volume"; value: "100%"; sentence: "The game's volume."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Navigation sounds"; value: "On"; sentence: "The interface's cues."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Sound volume"; value: "60%"; sentence: "The cues' volume."; kind: Kit.RowKinds.Steps }
        ListElement { label: "Output"; value: "System default"; sentence: "Where the sound plays."; kind: Kit.RowKinds.Opens }
    }
    ListModel {
        id: controllers
        ListElement { label: "Players"; value: ""; sentence: "Which pad plays as whom."; kind: Kit.RowKinds.Opens }
        ListElement { label: "Swap A and B"; value: "Off"; sentence: "A and B trade places."; kind: Kit.RowKinds.Steps }
        ListElement { label: "About"; value: "1.0"; sentence: "The program's version."; kind: Kit.RowKinds.Action }
    }
    ListModel {
        id: engine
        ListElement { label: "VenusRT (Rust)"; value: "EmuSen"; sentence: "VenusRT runs every Super Nintendo game."; kind: Kit.RowKinds.Choice; marked: false }
        ListElement { label: "Venus (C#)"; value: "EmuSen, default"; sentence: "Venus, the reference."; kind: Kit.RowKinds.Choice; marked: true }
        ListElement { label: "Snes9x"; value: "libretro"; sentence: "A hosted core."; kind: Kit.RowKinds.Choice; marked: false }
    }
    ListModel {
        id: crop
        ListElement { label: "Left"; value: "0.0%"; sentence: "The part of the picture hidden at its left edge, 0 to 25%."; kind: Kit.RowKinds.Slider; minimum: 0; maximum: 25; step: 0.5; amount: 0 }
        ListElement { label: "Right"; value: "2.0%"; sentence: "The part hidden at its right edge."; kind: Kit.RowKinds.Slider; minimum: 0; maximum: 25; step: 0.5; amount: 2 }
        ListElement { label: "Top"; value: "8.0%"; sentence: "The part hidden at its top."; kind: Kit.RowKinds.Slider; minimum: 0; maximum: 25; step: 0.5; amount: 8 }
        ListElement { label: "Bottom"; value: "25.0%"; sentence: "The part hidden at its foot, held by the console."; kind: Kit.RowKinds.Slider; minimum: 0; maximum: 25; step: 0.5; amount: 25; held: true }
        ListElement { label: "Curvature"; value: "1"; sentence: "A lever stepped as a value."; kind: Kit.RowKinds.Steps }
    }
    Kit.NormalizedCanvas {
        Kit.SettingsFrame {
            id: frame
            objectName: "settings"
            anchors.fill: parent
            form: Kit.Forms.BigScreen
            focus: true
            sections: ["Controllers", "Display", "Sound", "Consoles", "Library", "EmuSen"]
            section: 1
            trail: scene.stateTrails[scene.shown]
            model: scene.shown === 0 ? scene.sectionModels[frame.section] : scene.stateModels[scene.shown]
            help: ListModel {
                ListElement { button: 4; label: "Choose" }
                ListElement { button: 5; label: "Back" }
                ListElement { button: 8; label: "Section" }
            }
            onChosen: (row) => {
                scene.chosenRow = row
                if (scene.shown === 3) {
                    for (let i = 0; i < engine.count; ++i)
                        engine.setProperty(i, "marked", i === row)
                    frame.layout.read()
                }
            }
            onStepped: (row, by) => {
                scene.steppedRow = row
                scene.steppedBy = by
            }
            onSlid: (row, amount) => {
                scene.slidRow = row
                scene.slidAmount = amount
                crop.setProperty(row, "amount", amount)
                crop.setProperty(row, "value", amount.toFixed(1) + "%")
            }
            controller.onBackDeclined: scene.backs += 1
            preview: Item {
                width: parent ? parent.width : 0
                height: [276, 16, 160, 0, 0][scene.shown]
                Rectangle {
                    objectName: "picture"
                    visible: scene.shown === 0
                    anchors.fill: parent
                    color: Kit.Tokens.voidFill
                    Kit.Body { anchors.centerIn: parent; width: 120; height: 120; hue: 212; ring: Kit.Rings.Subtle }
                }
                Rectangle {
                    objectName: "meter"
                    visible: scene.shown === 1
                    anchors.fill: parent
                    radius: height / 2
                    color: Kit.Tokens.inputSurface
                    Rectangle { width: parent.width * 0.6; height: parent.height; radius: parent.radius; color: Kit.Tokens.accent }
                }
                Row {
                    objectName: "variants"
                    visible: scene.shown === 2
                    spacing: 16
                    Rectangle { width: 176; height: 160; radius: Kit.Tokens.tileRadius; color: Kit.Tokens.surface; border.width: 1; border.color: Kit.Tokens.border
                        Text { x: 12; y: 12; text: "Dark"; color: Kit.Tokens.text; font.pixelSize: Kit.Tokens.fontCaption } }
                    Rectangle { width: 176; height: 160; radius: Kit.Tokens.tileRadius; color: Kit.Tokens.inputSurface; border.width: 1; border.color: Kit.Tokens.border
                        Text { x: 12; y: 12; text: "Light"; color: Kit.Tokens.text; font.pixelSize: Kit.Tokens.fontCaption } }
                }
            }
            lines: Text {
                visible: scene.shown === 3
                width: parent ? parent.width : 0
                text: "A chooses; B keeps what was chosen."
                wrapMode: Text.Wrap
                color: Kit.Tokens.muted
                font.pixelSize: Kit.Tokens.fontBody
            }
        }
    }
}
