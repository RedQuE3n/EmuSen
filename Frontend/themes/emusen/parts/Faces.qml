import QtQuick
import EmuSen.Theme

// The screens' two faces from the theme API's face tokens: the words', and the labels' for names, the wordmark, the worlds' counts and Home's places' lines, each text's weight its own (R-20261112-1; EmuSen_Qt.md §7.1, §3.4.1; EmuSen_Kit.md §7.117).
Item {
    readonly property string words: Tokens.faceText
    readonly property string label: Tokens.faceLabel
}
