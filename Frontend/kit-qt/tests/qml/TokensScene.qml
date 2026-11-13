import QtQuick
import EmuSen.Kit as Kit

// Reads any token of the kit's Tokens by its name: `answer` is `asked`'s value as text (EmuSen_Kit.md §7.2).
Window {
    objectName: "scene"
    visible: true
    width: 320
    height: 240
    property string asked: "base"
    readonly property string answer: String(Kit.Tokens[asked])
}
