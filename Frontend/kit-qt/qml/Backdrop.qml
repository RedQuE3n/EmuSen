pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit

// A screen's backdrop as the Eclipse boards draw it: a ground, and over it blurred colour fields, or a picture blurred in their place; on the software backend each unblurred at its opacity (EmuSen_Kit.md §7.118).
Item {
    id: root
    property color ground: "transparent"
    // The fields, drawn in order, each in this item's units.
    property list<Kit.BackdropField> fields
    // A picture blurred over the whole box in place of the fields, as a focused game's art; empty draws the fields.
    property url source
    // The picture's CSS filter blur, the Gaussian's standard deviation, and its opacity; a screen sets both from its board.
    property real sourceBlur
    property real sourceOpacity: 1
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
                // The list's entry can go before its delegate when a theme's engine is dropped (§7.116): read as none then.
                readonly property color paint: field.modelData?.color ?? "transparent"
                // The field's CSS blur, the Gaussian's deviation in the backdrop's units (Filter Effects 1, blur()).
                readonly property real deviation: field.modelData?.blur ?? 0
                objectName: "backdropField." + field.index
                x: (field.modelData?.x ?? 0)
                y: (field.modelData?.y ?? 0)
                width: (field.modelData?.width ?? 0)
                height: (field.modelData?.height ?? 0)
                rotation: (field.modelData?.rotation ?? 0)
                opacity: (field.modelData?.opacity ?? 0)
                // On the card each field is drawn blurred by the kit's Gaussian, its deviation the field's CSS blur; on the
                // software backend, which renders no shader effect, the field is drawn unblurred (R-20261115-6).
                Item {
                    id: body
                    anchors.fill: parent
                    // The ellipse its box holds: a circle as wide as the box, scaled to its height, a painted ground as the disc is.
                    Rectangle {
                        objectName: "backdropEllipse"
                        width: field.width
                        height: field.width
                        visible: (field.modelData?.corner ?? -1) < 0
                        radius: width / 2
                        color: field.paint
                        antialiasing: true
                        transform: Scale {
                            yScale: field.height / Math.max(1, field.width)
                        }
                    }
                    Rectangle {
                        objectName: "backdropBox"
                        anchors.fill: parent
                        visible: (field.modelData?.corner ?? -1) >= 0
                        radius: Math.max(0, (field.modelData?.corner ?? -1))
                        color: field.paint
                    }
                }
                Loader {
                    active: !root.software && (field.modelData?.blur ?? 0) > 0
                    sourceComponent: Component {
                        Kit.GaussianBlur {
                            source: body
                            deviation: field.deviation
                            pixelScale: root.pixelScale
                        }
                    }
                }
            }
        }
    }
    Image {
        objectName: "backdropPicture"
        anchors.fill: parent
        // On the software backend no blur is made: the picture is drawn unblurred at its opacity (R-20261115-6).
        visible: root.pictured
        opacity: root.sourceOpacity
        source: root.visible ? root.source : ""
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        layer.enabled: !root.software
        layer.effect: MultiEffect {
            blurEnabled: true
            blur: 1
            blurMax: root.blurMax
            blurMultiplier: Math.max(0, root.sourceBlur * root.radiusShare * root.pixelScale / root.blurMax - 1)
            autoPaddingEnabled: false
        }
    }
}
