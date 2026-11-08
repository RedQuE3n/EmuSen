pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// A card's context pane in canvas units, laid in the card form's pane box, as Mistress's PauseCardPane draws it: the slots or the reel's newest pictures in a row of three, or the reel's focused moment over its strip of six, and a sentence; an empty pane hides, and the card form's box with it (EmuSen_Qt.md §7.2).
Item {
    id: root
    objectName: "PauseCardPaneArea"
    // The pane as plain data, a page model's `pane`.
    property alias pane: paneModel.pane
    readonly property alias model: paneModel
    PaneModel {
        id: paneModel
    }
    Faces {
        id: faces
    }
    Column {
        objectName: "PauseCardPane"
        x: 24
        y: 24
        width: 592
        spacing: paneModel.moment ? 8 : 12
        Rectangle {
            objectName: "PaneMoment"
            visible: paneModel.moment
            width: 592
            height: 300
            radius: Kit.Tokens.tileRadius
            color: Kit.Tokens.voidFill
            clip: true
            Image {
                objectName: "PaneMomentPicture"
                anchors.fill: parent
                source: paneModel.big
                fillMode: Image.PreserveAspectFit
                smooth: true
            }
        }
        Row {
            objectName: "PaneSlots"
            spacing: paneModel.moment ? 0 : 20
            Repeater {
                model: paneModel
                delegate: Column {
                    id: tile
                    required property int index
                    required property string picture
                    required property string name
                    required property string when
                    required property bool ringed
                    width: paneModel.moment ? 592 / 6 : 184
                    spacing: paneModel.moment ? 6 : 4
                    Rectangle {
                        objectName: "PaneSlot" + (tile.index + 1)
                        width: paneModel.moment ? 88 : 184
                        height: paneModel.moment ? 66 : 138
                        radius: paneModel.moment ? 6 : Kit.Tokens.tileRadius
                        color: paneModel.moment ? Kit.Tokens.voidFill : tile.picture.length > 0 ? Kit.Tokens.inputSurface : "transparent"
                        clip: true
                        Image {
                            objectName: "PanePicture"
                            anchors.fill: parent
                            visible: tile.picture.length > 0
                            source: tile.picture
                            fillMode: Image.PreserveAspectCrop
                            smooth: true
                        }
                        // An empty slot: a dashed outline in the Border token with its word in it.
                        Shape {
                            anchors.fill: parent
                            visible: paneModel.slots && tile.picture.length === 0
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                strokeColor: Kit.Tokens.border
                                strokeWidth: 1
                                strokeStyle: ShapePath.DashLine
                                fillColor: "transparent"
                                PathRectangle {
                                    x: 0.5
                                    y: 0.5
                                    width: 183
                                    height: 137
                                    radius: Kit.Tokens.tileRadius
                                }
                            }
                        }
                        Text {
                            anchors.centerIn: parent
                            visible: paneModel.slots && tile.picture.length === 0
                            text: paneModel.blank
                            color: Kit.Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 22
                        }
                        Rectangle {
                            anchors.fill: parent
                            visible: tile.ringed
                            radius: parent.radius
                            color: "transparent"
                            border.width: 3
                            border.color: Kit.Tokens.accent
                        }
                    }
                    Text {
                        width: parent.width
                        text: tile.name
                        color: tile.ringed || !paneModel.moment ? Kit.Tokens.textStrong : Kit.Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        elide: Text.ElideRight
                    }
                    Text {
                        width: parent.width
                        visible: !paneModel.moment && tile.when.length > 0
                        text: tile.when
                        color: Kit.Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        elide: Text.ElideRight
                    }
                }
            }
        }
        Text {
            objectName: "PaneSentence"
            width: 592
            text: paneModel.sentence
            color: Kit.Tokens.text
            font.family: faces.regular
            font.pixelSize: 22
            wrapMode: Text.Wrap
        }
    }
}
