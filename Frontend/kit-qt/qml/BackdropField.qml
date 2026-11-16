import QtQuick

// One field of a Backdrop: an ellipse or a rounded box of one colour, its CSS filter blur and opacity, turned about its centre, in the backdrop's units (EmuSen_Kit.md §7.118).
QtObject {
    property real x
    property real y
    property real width
    property real height
    property color color: "transparent"
    // The CSS filter blur, the Gaussian's standard deviation (Filter Effects 1, blur()).
    property real blur
    property real opacity: 1
    // Degrees, clockwise, about the field's centre.
    property real rotation
    // The corners' radius; below 0 the field is the ellipse its box holds (border-radius 50%).
    property real corner: -1
}
