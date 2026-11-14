import QtQuick

// One glow round a corona disc: the disc's circle grown by its spread and blurred by its blur, in its colour (EmuSen_Kit.md §7.116).
QtObject {
    property real blur
    property real spread
    property color color
}
