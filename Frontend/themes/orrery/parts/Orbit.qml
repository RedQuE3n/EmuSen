import QtQuick
import EmuSen.Theme

// The list an orbit draws: a row the view keeps made about its current item, and `turning`, the current index followed over Reel, which places each cover on the arc (orrery.md §1.5).
ListView {
    id: orbit
    property real turning: Math.max(0, orbit.currentIndex)
    Behavior on turning {
        NumberAnimation {
            duration: Tokens.reel
            easing.type: Tokens.reelCurve
        }
    }
    orientation: ListView.Horizontal
    // The wheel turns it, a step a notch, as the pad does.
    interactive: true
    keyNavigationEnabled: true
    keyNavigationWraps: false
    highlightRangeMode: ListView.StrictlyEnforceRange
    preferredHighlightBegin: orbit.width / 2 - 100
    preferredHighlightEnd: orbit.width / 2 + 100
    highlightMoveDuration: 0
    cacheBuffer: 400
    // Rows that arrive after the view is made start it at the first, so a cover holds the focus the walk gives the view.
    onCountChanged: if (orbit.currentIndex < 0 && orbit.count > 0) orbit.currentIndex = 0
}
