import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import EmuSen.Kit as Kit

// A row with a slider, for a lever or a crop's edge: its label, the slider and the value in words; the pad steps it by Left and Right through the frame's walk, the pointer drags it (EmuSen_Kit.md §7.38).
Kit.WholeRow {
    id: root
    property string label
    property string value
    property real minimum: 0
    property real maximum: 1
    property real step: 0
    property real amount: 0
    // Something holds the value: drawn, not changed.
    property bool held: false
    // The pointer moved the slider to this amount; the pad's steps come from the frame's walk.
    signal slid(real amount)
    Accessible.name: root.label
    RowLayout {
        anchors.fill: parent
        spacing: Kit.Tokens.base * 4
        Text {
            objectName: "label"
            Layout.preferredWidth: parent.width * 0.4
            text: root.label
            color: root.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
            font.pixelSize: Kit.Tokens.fontRow
            elide: Text.ElideRight
        }
        Slider {
            id: bar
            objectName: "slider"
            Layout.fillWidth: true
            from: root.minimum
            to: root.maximum
            stepSize: root.step
            snapMode: Slider.SnapAlways
            value: root.amount
            enabled: !root.held
            focusPolicy: Qt.NoFocus
            Accessible.name: root.label
            onMoved: root.slid(bar.value)
        }
        Text {
            objectName: "value"
            Layout.preferredWidth: Kit.Tokens.fontRow * 4
            horizontalAlignment: Text.AlignRight
            text: root.value
            color: Kit.Tokens.muted
            font.pixelSize: Kit.Tokens.fontRow
        }
    }
}
