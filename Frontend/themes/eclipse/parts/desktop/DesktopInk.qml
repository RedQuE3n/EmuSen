import QtQuick
import EmuSen.Theme

// Derived desktop chrome inks and translucent fills anchored to Tokens.
QtObject {
    readonly property color surface: Tokens.surface
    readonly property color surfaceRaised: Tokens.surfaceRaised
    readonly property color inputSurface: Tokens.inputSurface
    readonly property color text: Tokens.text
    readonly property color textStrong: Tokens.textStrong
    readonly property color muted: Tokens.muted
    readonly property color accent: Tokens.accent
    readonly property color overAccent: Tokens.overAccent
    readonly property color corona: Tokens.coronaColor
    readonly property color border: Tokens.border
    readonly property color glow: Tokens.glowColor
    readonly property color success: Tokens.success
    readonly property color attention: Tokens.attention
    readonly property color error: Tokens.error
    readonly property color overArt: Tokens.overArt
    readonly property color overArtScrim: Tokens.overArtScrim

    readonly property color dim: Qt.rgba(Tokens.muted.r * 0.88 + Tokens.surface.r * 0.12, Tokens.muted.g * 0.88 + Tokens.surface.g * 0.12, Tokens.muted.b * 0.88 + Tokens.surface.b * 0.12, 1)
    readonly property color hairline: Qt.alpha(Tokens.border, 0.34)
    readonly property color pillBorder: Qt.alpha(Tokens.border, 0.72)
    readonly property color track: Qt.alpha(Tokens.border, 0.32)
    readonly property color coronaSoft: Qt.alpha(Tokens.coronaColor, 0.55)
    readonly property color accentSoft: Qt.alpha(Tokens.accent, 0.45)
    readonly property color glowWeak: Qt.alpha(Tokens.glowColor, 0.16)
    readonly property color glowMid: Qt.alpha(Tokens.glowColor, 0.30)
    readonly property color rose: Qt.alpha(Tokens.glowColor, 1.0)
    readonly property color glass: Qt.alpha(Tokens.surface, 0.80)
    readonly property color glassStrong: Qt.alpha(Tokens.surface, 0.90)
    readonly property color railSurface: Qt.alpha(Tokens.inputSurface, 0.84)
    readonly property color cardHover: Qt.alpha(Tokens.surfaceRaised, 0.94)
    readonly property color successFill: Qt.alpha(Tokens.success, 0.18)
    readonly property color attentionFill: Qt.alpha(Tokens.attention, 0.18)
    readonly property color errorFill: Qt.alpha(Tokens.error, 0.18)
    readonly property color backdropGlow: Qt.alpha(Tokens.coronaGlowColor, 1.0)
    readonly property color backdropHalo: Qt.alpha(Tokens.coronaHaloColor, 1.0)
    readonly property color limbFill: Qt.rgba(Tokens.inputSurface.r * 0.65 + Tokens.surface.r * 0.35, Tokens.inputSurface.g * 0.65 + Tokens.surface.g * 0.35, Tokens.inputSurface.b * 0.65 + Tokens.surface.b * 0.35, 0.72)
    readonly property color limbRim: Qt.alpha(Tokens.coronaColor, 0.22)
}
