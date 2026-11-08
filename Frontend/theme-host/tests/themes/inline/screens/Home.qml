import QtQuick

// A stub screen of the host's tests: plain QtQuick, no EmuSen.Theme yet.
Item {
    objectName: "inlineHome"
    property var made: Qt.createQmlObject("import EmuSen.Models; LibraryModel {}", this, "inline")
}
