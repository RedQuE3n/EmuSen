import QtQuick

// A board's measure in canvas units: the canvas's design height (kit-core's, through NormalizedCanvas) over the boards' height (EmuSen_Qt.md §3.4.8).
QtObject {
    // The canvas's design height, NormalizedCanvas's designHeight.
    required property real designHeight
    // The height of every Eclipse board, canvas.json's h: a constant of the design.
    readonly property real height: 1200
    readonly property real unit: designHeight / height
}
