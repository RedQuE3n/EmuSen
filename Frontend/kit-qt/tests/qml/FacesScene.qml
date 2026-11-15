import QtQuick
import EmuSen.Kit as Kit

// A text in each face token's family, and one in each family the kit bundles, so a test reads the family each is drawn in (EmuSen_Kit.md §7.117).
Window {
    objectName: "scene"
    visible: true
    width: 320
    height: 240
    property string asked
    readonly property string answer: String(Kit.Tokens[asked])
    // The families, comma separated, as a test sets them.
    property string families
    Column {
        Text { objectName: "faceText"; text: "Words"; font.family: Kit.Tokens.faceText }
        Text { objectName: "faceLabel"; text: "Label"; font.family: Kit.Tokens.faceLabel }
        Text { objectName: "faceInterface"; text: "Row"; font.family: Kit.Tokens.faceInterface }
        Text { objectName: "faceCode"; text: "7E00F2:04"; font.family: Kit.Tokens.faceCode }
        Repeater {
            model: families.length > 0 ? families.split(",") : []
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
