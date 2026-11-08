import QtQuick
import EmuSen.Endymion

// The harness's scene for BellPlayer: a cue played whenever the root's cue is set (EmuSen_Kit.md §7.15).
Window {
    objectName: "scene"
    width: 320
    height: 200
    property alias platformLibrary: bells.platformLibrary
    property alias ready: bells.ready
    property alias problem: bells.problem
    property int cue: -1
    onCueChanged: bells.play(cue)
    BellPlayer {
        id: bells
        volume: 0.5
    }
}
