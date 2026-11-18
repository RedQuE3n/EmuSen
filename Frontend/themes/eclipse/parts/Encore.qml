import QtQuick
import EmuSen.Theme

// Home arriving as the splash's board times it, ease-out (EclipseSplashPlay:24-27, ec-h1 to ec-h4): each part's opacity and offset in board pixels while the splash shows, whole once it has gone (EmuSen_Qt.md §3.4.8, the splash).
QtObject {
    id: encore
    readonly property bool splashing: Splash.shown
    // The moment as the board's per cent of its 11 s loop, a constant of the design; past its end once the splash has gone.
    readonly property real p: encore.splashing ? Splash.time / 110 : 100
    // The header from 16 above, 72% to 80%; the words from 30 to the left, 74% to 83%; Recent from 36 below, 77% to 86%; the help line 80% to 88%.
    readonly property Ease h1: Ease { x2: 0.58; y2: 1; progress: (encore.p - 72) / 8 }
    readonly property Ease h2: Ease { x2: 0.58; y2: 1; progress: (encore.p - 74) / 9 }
    readonly property Ease h3: Ease { x2: 0.58; y2: 1; progress: (encore.p - 77) / 9 }
    readonly property Ease h4: Ease { x2: 0.58; y2: 1; progress: (encore.p - 80) / 8 }
    readonly property real headerOpacity: encore.h1.value
    readonly property real headerY: -16 * (1 - encore.h1.value)
    readonly property real mainOpacity: encore.h2.value
    readonly property real mainX: -30 * (1 - encore.h2.value)
    readonly property real recentOpacity: encore.h3.value
    readonly property real recentY: 36 * (1 - encore.h3.value)
    readonly property real footerOpacity: encore.h4.value
}
