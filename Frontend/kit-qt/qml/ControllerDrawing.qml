pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// A controller drawn from a drawing file, a label beside it for each control the host declares with its leader, the focused control ringed and the held ones lit (EmuSen_Kit.md §7.62).
Item {
    id: root
    // The drawing file's text; empty draws the generic controller from the controls alone.
    property alias drawing: layout.drawing
    // The controls the host's controller declares, in the order the walk takes them.
    property alias controls: layout.controls
    // What each control's label says beside its name, in the controls' order.
    property alias texts: layout.texts
    property alias focused: layout.focused
    property alias held: layout.held
    // How far each control is pushed, 0 to 1, in the controls' order: a stick's directions move its knob.
    property list<real> pushes
    readonly property alias layout: layout
    // A label chosen by the pointer, by its control's place among the controls.
    signal chosen(int index)

    Accessible.role: Accessible.Grouping
    Accessible.name: layout.name

    Kit.ControllerLayout {
        id: layout
        objectName: "controllerLayout"
        width: root.width
        height: root.height
        labelHeight: Math.ceil(probe.height) + 2 * root.inset
        accent: Kit.Tokens.accent
    }
    Binding {
        target: layout
        property: "pushes"
        value: root.pushes
    }

    // A label's border and the space inside it.
    readonly property real inset: 2
    readonly property real padding: 6
    FontMetrics {
        id: probe
        font.pixelSize: Kit.Tokens.fontCaption
    }

    Shape {
        objectName: "controllerLeaders"
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: Kit.Tokens.muted
            strokeWidth: 1.5
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            PathSvg { path: layout.leaders }
        }
        ShapePath {
            strokeColor: Kit.Tokens.accent
            strokeWidth: 2.5
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            PathSvg { path: layout.focusLeader }
        }
    }

    // The controller's picture: the drawing as one SVG document, its colours the console's own (EmuSen_Kit.md §7.62).
    Image {
        objectName: "controllerArt"
        visible: layout.scale > 0
        x: layout.place.x
        y: layout.place.y
        width: Math.max(1, layout.place.width)
        height: Math.max(1, layout.place.height)
        sourceSize.width: Math.ceil(width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(height * Screen.devicePixelRatio)
        source: layout.picture
        cache: false
        smooth: true
    }

    Repeater {
        model: root.controls.length
        // A label is measured as it is made and again whenever its words change; the layout places it by its width.
        onItemAdded: (index, item) => layout.setLabelWidth(index, item.implicitWidth)
        delegate: Rectangle {
            id: label
            required property int index
            readonly property bool focusedLabel: label.index === layout.focused
            readonly property list<real> boxes: layout.labelBoxes
            objectName: "controllerLabel"
            x: label.boxes.length > 4 * label.index ? label.boxes[4 * label.index] : 0
            y: label.boxes.length > 4 * label.index ? label.boxes[4 * label.index + 1] : 0
            width: label.boxes.length > 4 * label.index ? label.boxes[4 * label.index + 2] : 0
            height: label.boxes.length > 4 * label.index ? label.boxes[4 * label.index + 3] : 0
            implicitWidth: Math.ceil(words.implicitWidth) + 2 * (root.inset + root.padding)
            onImplicitWidthChanged: layout.setLabelWidth(label.index, label.implicitWidth)
            visible: label.width > 0
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.inputSurface
            border.color: label.focusedLabel ? Kit.Tokens.accent : Kit.Tokens.border
            border.width: label.focusedLabel ? 2 : 1
            Accessible.role: Accessible.Button
            Accessible.name: words.text
            Text {
                id: words
                objectName: "controllerLabelWords"
                x: root.inset + root.padding
                anchors.verticalCenter: parent.verticalCenter
                text: label.index < layout.labelWords.length ? layout.labelWords[label.index] : ""
                color: label.focusedLabel ? Kit.Tokens.textStrong : Kit.Tokens.text
                font.pixelSize: Kit.Tokens.fontCaption
            }
            TapHandler {
                onTapped: root.chosen(label.index)
            }
        }
    }
}
