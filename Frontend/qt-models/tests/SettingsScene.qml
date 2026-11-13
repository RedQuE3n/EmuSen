import QtQuick
import EmuSen.Models

// The harness's scene for SettingsModel: the model, and a row turned when the test moves `turn`, so the turn needs no turn
// of the event loop of its own (EmuSen_Qt.md §7.7.H7).
Window {
    objectName: "scene"
    width: 640
    height: 480
    property alias platformLibrary: settings.platformLibrary
    property alias open: settings.open
    property alias section: settings.section
    property alias ready: settings.ready
    property alias light: settings.light
    property alias problem: settings.problem
    property int row: -1
    property int turn: 0
    onTurnChanged: settings.step(row, 1)
    SettingsModel {
        id: settings
    }
}
