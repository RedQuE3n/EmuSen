import QtQuick
import EmuSen.Kit as Kit

// A text in each face token's family, its words the family's name, and one in each family the kit bundles, so a test reads the family each is drawn in (EmuSen_Kit.md §7.117).
Window {
    id: root
    objectName: "scene"
    visible: true
    width: 320
    height: 240
    property string asked
    readonly property string answer: String(Kit.Tokens[root.asked])
    // The families joined by the separator, as a test sets both.
    property string families
    property string separator
    Column {
        Text { objectName: "faceText"; text: Kit.Tokens.faceText; font.family: Kit.Tokens.faceText }
        Text { objectName: "faceLabel"; text: Kit.Tokens.faceLabel; font.family: Kit.Tokens.faceLabel }
        Text { objectName: "faceInterface"; text: Kit.Tokens.faceInterface; font.family: Kit.Tokens.faceInterface }
        Text { objectName: "faceCode"; text: Kit.Tokens.faceCode; font.family: Kit.Tokens.faceCode }
        Repeater {
            model: root.families.length > 0 ? root.families.split(root.separator) : []
            delegate: Text {
                required property string modelData
                required property int index
                objectName: "bundled." + index
                text: modelData
                font.family: modelData
            }
        }
    }
}
