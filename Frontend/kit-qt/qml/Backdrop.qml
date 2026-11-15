pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A screen's backdrop as the Eclipse boards draw it: a ground, and over it blurred colour fields, or a picture blurred in their place; on the software backend the fields unblurred at their opacity and no picture (EmuSen_Kit.md §7.118).
Item {
    id: root
    property color ground: "transparent"
    // The fields, drawn in order, each in this item's units.
    property list<Kit.BackdropField> fields
    // A picture blurred over the whole box in place of the fields, as a focused game's art; empty draws the fields.
    property url source
    // The picture's CSS filter blur, the Gaussian's standard deviation.
    property real sourceBlur
    // Scene pixels to one of this item's units, so a blur in units is a blur in the pixels the card draws.
    property real pixelScale: 1
    // The software backend renders no shader effect (doc.qt.io, "Qt Quick Software Adaptation"): there no blur is made.
    readonly property bool software: GraphicsInfo.api === GraphicsInfo.Software
    readonly property bool pictured: root.source.toString().length > 0
    // MultiEffect's blur radius, read as a box-shadow's blur radius, twice the deviation (CSS Backgrounds 3 §6.1.2): a constant of the reading.
    readonly property real radiusShare: 2
    // The largest radius MultiEffect's page calls meaningful for blurMax; past it the multiplier extends it.
    readonly property int blurMax: 64
    Rectangle {
        objectName: "backdropGround"
        anchors.fill: parent
        visible: root.ground.a > 0
        color: root.ground
    }
    Item {
        objectName: "backdropFields"
        anchors.fill: parent
        visible: !root.pictured
        Repeater {
            model: root.fields
            Item {
                id: field
                required property int index
                required property Kit.BackdropField modelData
                // The field's colour, as a reader asks it.
                readonly property color paint: field.modelData.color
                readonly property real radius: field.modelData.blur * root.radiusShare * root.pixelScale
                objectName: "backdropField." + field.index
                x: field.modelData.x
                y: field.modelData.y
                width: field.modelData.width
                height: field.modelData.height
                rotation: field.modelData.rotation
                opacity: field.modelData.opacity
                layer.enabled: !root.software && field.radius > 0
                layer.effect: MultiEffect {
                    blurEnabled: true
                    blur: 1
                    blurMax: root.blurMax
                    blurMultiplier: Math.max(0, field.radius / root.blurMax - 1)
                    autoPaddingEnabled: true
                }
                Shape {
                    objectName: "backdropEllipse"
                    anchors.fill: parent
                    visible: field.modelData.corner < 0
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        strokeColor: "transparent"
                        fillColor: field.modelData.color
                        PathAngleArc {
                            centerX: field.width / 2
                            centerY: field.height / 2
                            radiusX: field.width / 2
                            radiusY: field.height / 2
                            sweepAngle: 360
                        }
                    }
                }
                Rectangle {
                    objectName: "backdropBox"
                    anchors.fill: parent
                    visible: field.modelData.corner >= 0
                    radius: Math.max(0, field.modelData.corner)
                    color: field.modelData.color
                }
            }
        }
    }
    Image {
        objectName: "backdropPicture"
        anchors.fill: parent
        // On the software backend no blur is made, so no picture is drawn sharp in its place (R-20261115-6).
        visible: root.pictured && !root.software
        source: root.visible ? root.source : ""
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        layer.enabled: true
        layer.effect: MultiEffect {
            blurEnabled: true
            blur: 1
            blurMax: root.blurMax
            blurMultiplier: Math.max(0, root.sourceBlur * root.radiusShare * root.pixelScale / root.blurMax - 1)
            autoPaddingEnabled: false
        }
    }
}
