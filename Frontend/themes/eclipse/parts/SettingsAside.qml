pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// The About panel beside the rows, as the boards draw it: the preview and its caption, the focused row's name, the program's sentence and its further and muted lines, 22 apart (G:138-151; EmuSen_Qt.md §3.4.11).
Column {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    // The focused row's name, the panel's heading (G:162).
    property string heading
    // Scene pixels to a canvas unit, so the picture's glows blur at the pixels drawn.
    property real pixelScale: 1
    // Settings.preview's numbers, the theme API's (EmuSen_Qt.md §3.2): 1 the picture, 2 the meter, 3 the variants, 4 the text sizes, 6 the face buttons.
    readonly property bool pictured: Settings.preview === 1
    readonly property bool sampled: Settings.preview === 3 || Settings.preview === 4
    objectName: "settingsAside"
    width: 424 * root.unit
    spacing: 22 * root.unit
    Ink {
        id: ink
    }
    SettingsInk {
        id: own
    }
    // The picture, 318 high at a corner of 22 inside its 2 px rim and its 60 px glow (G:146): the paused game's own frame, none while no game is paused (R-20261116-25).
    Item {
        id: picture
        objectName: "asidePicture"
        visible: root.pictured && Session.frame !== null
        width: root.width
        height: 318 * root.unit
        readonly property real radius: 22 * root.unit
        Glow {
            objectName: "asidePictureGlow"
            anchors.fill: parent
            radius: picture.radius
            color: own.pictureGlow
            blur: 60 * root.unit
            spread: 10 * root.unit
            pixelScale: root.pixelScale
        }
        Glow {
            objectName: "asidePictureRim"
            anchors.fill: parent
            radius: picture.radius
            color: ink.resumeRim
            spread: 2 * root.unit
            pixelScale: root.pixelScale
        }
        ShaderEffectSource {
            id: frame
            readonly property Item shown: Session.frame
            readonly property real cutWidth: frame.shown ? Math.min(frame.shown.width, frame.shown.height * picture.width / picture.height) : 0
            readonly property real cutHeight: frame.cutWidth * picture.height / picture.width
            sourceItem: picture.visible ? frame.shown : null
            hideSource: false
            visible: false
            sourceRect: frame.shown ? Qt.rect((frame.shown.width - frame.cutWidth) / 2, (frame.shown.height - frame.cutHeight) / 2, frame.cutWidth, frame.cutHeight) : Qt.rect(0, 0, 0, 0)
            textureSize: Qt.size(Math.ceil(picture.width * root.pixelScale), Math.ceil(picture.height * root.pixelScale))
        }
        Shape {
            objectName: "asidePictureFrame"
            anchors.fill: parent
            visible: Session.frame !== null
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                strokeColor: "transparent"
                fillColor: "transparent"
                fillItem: frame
                fillTransform: PlanarTransform.fromScale(picture.width / Math.max(1, frame.textureSize.width), picture.height / Math.max(1, frame.textureSize.height))
                PathRectangle {
                    width: picture.width
                    height: picture.height
                    radius: picture.radius
                }
            }
        }
    }
    // The focused row's value words, which the ring shows inside it.
    Item {
        visible: false
        Repeater {
            model: Settings.rows
            delegate: Item {
                id: shown
                required property int index
                required property string value
                Binding {
                    target: meter
                    property: "value"
                    value: shown.value
                    when: shown.index === Settings.focused
                    restoreMode: Binding.RestoreNone
                }
            }
        }
    }
    SettingsMeter {
        id: meter
        unit: root.unit
        pixelScale: root.pixelScale
        visible: Settings.preview === 2
        share: Settings.meter
        caption: Settings.caption
    }
    SettingsFaces {
        unit: root.unit
        visible: Settings.preview === 6
    }
    Line {
        objectName: "asideCaption"
        unit: root.unit
        visible: picture.visible && text.length > 0
        width: root.width
        text: Settings.caption
        color: Tokens.muted
        wrapMode: Text.WordWrap
        lineHeightMode: Text.FixedHeight
        lineHeight: 1.5 * 18
        size: 18
    }
    // The program's samples, as the text sizes and the variants show them, each a line in the panel's own form (R-20261123-21).
    Repeater {
        model: root.sampled ? Settings.samples : []
        delegate: Line {
            required property int index
            required property string modelData
            objectName: "asideSample." + index
            unit: root.unit
            width: root.width
            text: modelData
            color: ink.emptyAbout
            wrapMode: Text.WordWrap
            lineHeightMode: Text.FixedHeight
            lineHeight: 1.5 * size
            size: 20 * (Settings.preview === 4 ? (Settings.sampleScales[index] ?? 1) : 1)
            weight: Tokens.weightSentence
        }
    }
    Line {
        objectName: "asideHeading"
        unit: root.unit
        visible: text.length > 0
        width: root.width
        text: root.heading
        color: Tokens.text
        wrapMode: Text.WordWrap
        size: 26
        weight: Tokens.weightTitle
    }
    Line {
        objectName: "asideSentence"
        unit: root.unit
        visible: text.length > 0
        width: root.width
        text: Settings.sentence
        color: ink.emptyAbout
        wrapMode: Text.WordWrap
        lineHeightMode: Text.FixedHeight
        lineHeight: 1.5 * 20
        size: 20
        weight: Tokens.weightSentence
    }
    Repeater {
        model: Settings.lines
        delegate: Line {
            required property int index
            required property string modelData
            objectName: "asideLine." + index
            unit: root.unit
            width: root.width
            text: modelData
            color: ink.emptyAbout
            wrapMode: Text.WordWrap
            lineHeightMode: Text.FixedHeight
            lineHeight: 1.5 * 20
            size: 20
            weight: Tokens.weightSentence
        }
    }
    Repeater {
        model: Settings.muted
        delegate: Line {
            required property int index
            required property string modelData
            objectName: "asideMuted." + index
            unit: root.unit
            width: root.width
            text: modelData
            color: Tokens.muted
            wrapMode: Text.WordWrap
            lineHeightMode: Text.FixedHeight
            lineHeight: 1.5 * 18
            size: 18
        }
    }
}
