pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Settings on the kit's settings frame: the six sections, the rows of the section or page shown, and the focused row's pane with its preview; every word and value is SettingsModel's (EmuSen_Qt.md §7.7).
Kit.NormalizedCanvas {
    id: screen
    required property SettingsModel settings
    property int padFamily
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on a section, and the declared actions the screens answer (Start, Home).
    signal backed()
    signal acted(string key)
    Connections {
        target: screen.settings
        function onLeft() { screen.backed() }
    }
    Kit.SettingsFrame {
        id: frame
        objectName: "settingsFrame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: screen.inputs
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        model: screen.settings
        sections: screen.settings.sections
        section: screen.settings.section
        trail: screen.settings.trail
        help: HelpModel {
            screen: screen.settings.help
        }
        onSectionChosen: (section) => screen.settings.section = section
        onStepped: (row, by) => screen.settings.step(row, by)
        onChosen: (row) => screen.settings.choose(row)
        preview: Item {
            objectName: "settingsPreview"
            width: parent ? parent.width : 0
            // Read from the model, not from the items' visible, which is false while the slot is empty and so would keep it so.
            height: screen.settings.preview === 1 ? picture.height : screen.settings.preview === 2 ? meter.height : screen.settings.preview === 3 ? variants.height : screen.settings.preview === 4 ? sizes.height : 0
            // The game's picture, named by its caption; the frame itself is drawn from piece J on (EmuSen_Qt.md §7.7).
            Rectangle {
                id: picture
                objectName: "SettingsPicture"
                visible: screen.settings.preview === 1
                width: parent.width
                height: parent.width * 3 / 4
                color: Kit.Tokens.voidFill
                Text {
                    objectName: "SettingsCaption"
                    anchors.centerIn: parent
                    width: Math.min(implicitWidth, parent.width - Kit.Tokens.base * 8)
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    text: screen.settings.caption
                    color: Kit.Tokens.overArt
                    font.family: faces.condensed
                    font.pixelSize: Kit.Tokens.fontTitle
                }
            }
            Rectangle {
                id: meter
                objectName: "SettingsMeter"
                visible: screen.settings.preview === 2
                width: parent.width
                height: Kit.Tokens.base * 4
                radius: meter.height / 2
                color: Kit.Tokens.inputSurface
                Rectangle {
                    width: meter.width * screen.settings.meter
                    height: meter.height
                    radius: meter.radius
                    color: Kit.Tokens.accent
                }
            }
            Row {
                id: variants
                objectName: "SettingsVariants"
                visible: screen.settings.preview === 3
                spacing: Kit.Tokens.base * 4
                Repeater {
                    model: screen.settings.samples
                    delegate: Rectangle {
                        id: tile
                        required property string modelData
                        required property int index
                        width: (variants.parent.width - variants.spacing) / 2
                        height: tile.width * 10 / 11
                        radius: Kit.Tokens.tileRadius
                        color: tile.index === 0 ? Kit.Tokens.surface : Kit.Tokens.inputSurface
                        border.width: 1
                        border.color: Kit.Tokens.border
                        Text {
                            x: Kit.Tokens.base * 1.5
                            y: Kit.Tokens.base * 1.5
                            text: tile.modelData
                            color: Kit.Tokens.text
                            font.family: faces.regular
                            font.pixelSize: Kit.Tokens.fontCaption
                        }
                    }
                }
            }
            Column {
                id: sizes
                objectName: "SettingsSizes"
                visible: screen.settings.preview === 4
                width: parent.width
                spacing: frame.layout.lineGap
                Repeater {
                    model: screen.settings.samples
                    delegate: Text {
                        id: sample
                        objectName: "SettingsLine"
                        required property string modelData
                        required property int index
                        width: sizes.width
                        text: sample.modelData
                        elide: Text.ElideRight
                        color: Kit.Tokens.text
                        font.family: faces.regular
                        font.pixelSize: Kit.Tokens.fontBody / Kit.Tokens.textScale * screen.settings.sampleScales[sample.index]
                    }
                }
            }
        }
        lines: Column {
            objectName: "settingsPaneLines"
            width: parent ? parent.width : 0
            spacing: frame.layout.lineGap
            Repeater {
                model: screen.settings.paneLines
                delegate: Text {
                    id: line
                    objectName: "SettingsLine"
                    required property string modelData
                    width: parent.width
                    text: line.modelData
                    wrapMode: Text.Wrap
                    color: Kit.Tokens.text
                    font.family: faces.regular
                    font.pixelSize: Kit.Tokens.fontBody
                }
            }
            Repeater {
                model: screen.settings.paneMuted
                delegate: Text {
                    id: muted
                    objectName: "SettingsLine"
                    required property string modelData
                    width: parent.width
                    text: muted.modelData
                    wrapMode: Text.Wrap
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: Kit.Tokens.fontBody
                }
            }
        }
        Binding {
            target: screen.settings
            property: "focused"
            value: frame.currentRow
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { screen.acted(key) }
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { screen.settings.back() }
        }
    }
    Faces {
        id: faces
    }
}
