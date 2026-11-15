pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models

// Settings as the window places it: made by a Loader only while shown (EmuSen_Qt.md §9.3.2), the canvas scaling to this
// item, which the Loader sizes; the Loader focuses the row the model holds, then sets `placed`.
Item {
    id: holder
    required property SettingsModel settings
    property int padFamily
    property bool inputs: true
    property alias placed: settingsScreen.placed
    readonly property alias layout: settingsScreen.layout
    signal backed()
    signal acted(string key)
    Settings {
        id: settingsScreen
        objectName: "settings"
        settings: holder.settings
        padFamily: holder.padFamily
        inputs: holder.inputs
        onBacked: holder.backed()
        onActed: (key) => holder.acted(key)
    }
}
