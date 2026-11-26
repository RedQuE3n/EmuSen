pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.App

// Four-row Eclipse keyboard matching board #8 (10x3 keys plus 123 / Space / Erase / Done).
FocusScope {
    id: root
    required property Kit.KeyboardModel typist
    required property real unit
    property real pixelScale: 1
    property bool activeFocusOnKeys: root.activeFocus
    property string eraseIcon: ""
    property string keyPrefix: ""
    property string keyRingPrefix: ""
    property string keyBgPrefix: ""
    property string keyTextPrefix: ""

    width: 1446 * root.unit
    height: 426 * root.unit
    Keys.onPressed: (event) => event.accepted = root.typist.key(event.key, event.modifiers, event.text)

    Repeater {
        model: root.typist
        Item {
            id: keyBox
            required property int index
            required property string label
            readonly property bool isRow3: keyBox.index >= 30
            readonly property int col: keyBox.isRow3 ? keyBox.index - 30 : keyBox.index % 10
            readonly property int row: keyBox.isRow3 ? 3 : Math.floor(keyBox.index / 10)
            readonly property bool focusedKey: root.activeFocusOnKeys && root.typist.current === keyBox.index
            readonly property bool isErase: keyBox.index === 32
            readonly property bool isDone: keyBox.index === 33
            readonly property bool accentFill: keyBox.focusedKey || keyBox.isDone

            objectName: root.keyPrefix + keyBox.index
            x: (!keyBox.isRow3 ? keyBox.col * 146 : (keyBox.col === 0 ? 0 : keyBox.col === 1 ? 219 : keyBox.col === 2 ? 949 : 1168)) * root.unit
            y: keyBox.row * 110 * root.unit
            width: (!keyBox.isRow3 ? 132 : (keyBox.col === 0 || keyBox.col === 2 ? 205 : keyBox.col === 1 ? 716 : 278)) * root.unit
            height: 96 * root.unit

            Kit.FocusRing {
                objectName: root.keyRingPrefix + keyBox.index
                anchors.fill: parent
                visible: keyBox.focusedKey
                radius: 20 * root.unit
                gap: 3 * root.unit
                thickness: 3 * root.unit
                glowColor: ThemeInk.glowMid
                glowBlur: 34 * root.unit
                glowSpread: 8 * root.unit
                pixelScale: root.pixelScale
            }

            Rectangle {
                objectName: root.keyBgPrefix + keyBox.index
                anchors.fill: parent
                radius: 20 * root.unit
                color: keyBox.accentFill ? Kit.Tokens.accent : Kit.Tokens.surfaceRaised
            }

            CardLine {
                objectName: root.keyTextPrefix + keyBox.index
                visible: !keyBox.isErase
                anchors.centerIn: parent
                unit: root.unit
                boardLine: true
                text: keyBox.label
                size: !keyBox.isRow3 ? 34 : (keyBox.col === 1 ? 24 : 26)
                weight: (!keyBox.isRow3 || keyBox.isDone) ? Font.DemiBold : Font.Normal
                color: keyBox.accentFill ? Kit.Tokens.overAccent : (keyBox.col === 1 && keyBox.isRow3 ? ThemeInk.body : Kit.Tokens.text)
            }

            Item {
                visible: keyBox.isErase
                anchors.centerIn: parent
                width: 40 * root.unit
                height: 40 * root.unit
                Shape {
                    width: 24
                    height: 24
                    scale: 40 * root.unit / 24
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: "transparent"
                        strokeColor: keyBox.accentFill ? Kit.Tokens.overAccent : Kit.Tokens.text
                        strokeWidth: 1.8
                        capStyle: ShapePath.RoundCap
                        joinStyle: ShapePath.RoundJoin
                        PathSvg { path: root.eraseIcon }
                    }
                }
            }

            TapHandler {
                onTapped: root.typist.click(keyBox.index)
            }
        }
    }
}
