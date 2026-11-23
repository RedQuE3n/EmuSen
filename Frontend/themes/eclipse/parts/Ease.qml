import QtQuick

// A CSS cubic-Bézier timing function at a progress (CSS Easing 1 §3.2): the curve's parameter found by eight Newton steps from the progress, its y the eased value (EmuSen_Qt.md §3.4.8, the splash).
QtObject {
    // The control points, as `cubic-bezier(x1, y1, x2, y2)` writes them.
    property real x1
    property real y1
    property real x2
    property real y2
    // How far through the keyframes' span the moment is, held to 0 to 1.
    property real progress
    readonly property real x: Math.max(0, Math.min(1, progress))
    // The curve's x and its slope at a parameter s: 3(1-s)²s·x1 + 3(1-s)s²·x2 + s³, and its derivative.
    readonly property real s0: x
    readonly property real s1: Math.max(0, Math.min(1, s0 - (3 * (1 - s0) * (1 - s0) * s0 * x1 + 3 * (1 - s0) * s0 * s0 * x2 + s0 * s0 * s0 - x) / Math.max(1e-9, 3 * (1 - s0) * (1 - s0) * x1 + 6 * (1 - s0) * s0 * (x2 - x1) + 3 * s0 * s0 * (1 - x2))))
    readonly property real s2: Math.max(0, Math.min(1, s1 - (3 * (1 - s1) * (1 - s1) * s1 * x1 + 3 * (1 - s1) * s1 * s1 * x2 + s1 * s1 * s1 - x) / Math.max(1e-9, 3 * (1 - s1) * (1 - s1) * x1 + 6 * (1 - s1) * s1 * (x2 - x1) + 3 * s1 * s1 * (1 - x2))))
    readonly property real s3: Math.max(0, Math.min(1, s2 - (3 * (1 - s2) * (1 - s2) * s2 * x1 + 3 * (1 - s2) * s2 * s2 * x2 + s2 * s2 * s2 - x) / Math.max(1e-9, 3 * (1 - s2) * (1 - s2) * x1 + 6 * (1 - s2) * s2 * (x2 - x1) + 3 * s2 * s2 * (1 - x2))))
    readonly property real s4: Math.max(0, Math.min(1, s3 - (3 * (1 - s3) * (1 - s3) * s3 * x1 + 3 * (1 - s3) * s3 * s3 * x2 + s3 * s3 * s3 - x) / Math.max(1e-9, 3 * (1 - s3) * (1 - s3) * x1 + 6 * (1 - s3) * s3 * (x2 - x1) + 3 * s3 * s3 * (1 - x2))))
    readonly property real s5: Math.max(0, Math.min(1, s4 - (3 * (1 - s4) * (1 - s4) * s4 * x1 + 3 * (1 - s4) * s4 * s4 * x2 + s4 * s4 * s4 - x) / Math.max(1e-9, 3 * (1 - s4) * (1 - s4) * x1 + 6 * (1 - s4) * s4 * (x2 - x1) + 3 * s4 * s4 * (1 - x2))))
    readonly property real s6: Math.max(0, Math.min(1, s5 - (3 * (1 - s5) * (1 - s5) * s5 * x1 + 3 * (1 - s5) * s5 * s5 * x2 + s5 * s5 * s5 - x) / Math.max(1e-9, 3 * (1 - s5) * (1 - s5) * x1 + 6 * (1 - s5) * s5 * (x2 - x1) + 3 * s5 * s5 * (1 - x2))))
    readonly property real s7: Math.max(0, Math.min(1, s6 - (3 * (1 - s6) * (1 - s6) * s6 * x1 + 3 * (1 - s6) * s6 * s6 * x2 + s6 * s6 * s6 - x) / Math.max(1e-9, 3 * (1 - s6) * (1 - s6) * x1 + 6 * (1 - s6) * s6 * (x2 - x1) + 3 * s6 * s6 * (1 - x2))))
    readonly property real s8: Math.max(0, Math.min(1, s7 - (3 * (1 - s7) * (1 - s7) * s7 * x1 + 3 * (1 - s7) * s7 * s7 * x2 + s7 * s7 * s7 - x) / Math.max(1e-9, 3 * (1 - s7) * (1 - s7) * x1 + 6 * (1 - s7) * s7 * (x2 - x1) + 3 * s7 * s7 * (1 - x2))))
    readonly property real value: 3 * (1 - s8) * (1 - s8) * s8 * y1 + 3 * (1 - s8) * s8 * s8 * y2 + s8 * s8 * s8
}
