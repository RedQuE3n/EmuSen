import QtQuick
import EmuSen.Theme

// Home arriving as the splash's board times it, ease-out (EclipseSplashPlay:24-27, ec-h1 to ec-h4): each part's opacity and offset in board pixels while the splash shows, then the hand-over over `Splash.ease`, whole after it (EmuSen_Qt.md §3.4.8, the splash).
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
    // Home whole in ms: ec-h4's end, 88% of the 11 s loop, a constant of the design; the hand-over runs from it for `Splash.ease`.
    readonly property real whole: 88 * 110
    readonly property bool easing: !encore.splashing && Splash.ease > 0 && Splash.time >= encore.whole && Splash.time < encore.whole + Splash.ease
    // The hand-over, ease-out as Home's arrival: 0 while the splash shows, 1 once Home is whole (R-20261123-9).
    readonly property Ease hand: Ease { x2: 0.58; y2: 1; progress: encore.splashing ? 0 : encore.easing ? (Splash.time - encore.whole) / Splash.ease : 1 }
    readonly property real handed: encore.hand.value
}
