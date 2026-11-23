import QtQuick

// The splash's corona in board pixels before the disc opens, and how far it opens: the splash draws it and Home's hand-over eases from it (EclipseSplashPlay:54; EmuSen_Qt.md §3.4.8, the splash).
QtObject {
    // The rim and the three glows' blur and spread, constants of the design (ec-corona).
    readonly property real rim: 2
    readonly property real nearBlur: 26
    readonly property real nearSpread: 6
    readonly property real middleBlur: 90
    readonly property real middleSpread: 22
    readonly property real farBlur: 240
    readonly property real farSpread: 80
    // ec-group's scale once open, a constant of the design.
    readonly property real opened: 5
}
