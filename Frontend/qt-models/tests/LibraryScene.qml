import QtQuick
import EmuSen.Models

// The harness's scene for LibraryModel: a column whose rows show the model's roles, read back through the root (EmuSen_Qt.md §1.2.1).
Window {
    objectName: "scene"
    width: 640
    height: 480
    property alias selectedKey: library.selectedKey
    property alias selectedIndex: library.selectedIndex
    property alias rows: rows.count
    property alias platformLibrary: library.platformLibrary
    property alias loading: library.loading
    property string fourthKey: rows.count > 3 ? rows.itemAt(3).key : ""
    property string firstName: rows.count > 2 ? rows.itemAt(0).name : ""
    property string secondKey: rows.count > 2 ? rows.itemAt(1).key : ""
    property string thirdKind: rows.count > 2 ? rows.itemAt(2).kind : ""
    property int thirdCount: rows.count > 2 ? rows.itemAt(2).count : -1
    property string thirdShown: rows.count > 2 ? rows.itemAt(2).text : ""
    // The Home card's facts as the window binds them: the consoles that start empty and the tools' lines are arguments.
    property string startableGiven
    property string toolsGiven
    readonly property string homeFacts: library.homeCardFacts("", "", "", false, startableGiven, toolsGiven)
    LibraryModel {
        id: library
    }
    Column {
        anchors.fill: parent
        Repeater {
            id: rows
            model: library
            delegate: Text {
                required property int index
                required property string kind
                required property string key
                required property string name
                required property int count
                text: name
            }
        }
    }
}
