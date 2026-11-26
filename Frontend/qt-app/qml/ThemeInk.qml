pragma Singleton
import QtQuick
import EmuSen.Kit as Kit

// Program-wide chrome colour roles derived from Kit.Tokens so dark, light, and custom theme.json packages re-skin every screen.
QtObject {
    // Direct pass-throughs from Kit.Tokens.
    readonly property color surface: Kit.Tokens.surface
    readonly property color surfaceRaised: Kit.Tokens.surfaceRaised
    readonly property color inputSurface: Kit.Tokens.inputSurface
    readonly property color voidFill: Kit.Tokens.voidFill
    readonly property color text: Kit.Tokens.text
    readonly property color textStrong: Kit.Tokens.textStrong
    readonly property color muted: Kit.Tokens.muted
    readonly property color accent: Kit.Tokens.accent
    readonly property color overAccent: Kit.Tokens.overAccent
    readonly property color corona: Kit.Tokens.coronaColor
    readonly property color border: Kit.Tokens.border
    readonly property color rule: Kit.Tokens.rule
    readonly property color scrim: Kit.Tokens.scrim
    readonly property color glow: Kit.Tokens.glowColor
    readonly property color coronaGlow: Kit.Tokens.coronaGlowColor
    readonly property color coronaHalo: Kit.Tokens.coronaHaloColor
    readonly property color success: Kit.Tokens.success
    readonly property color attention: Kit.Tokens.attention
    readonly property color error: Kit.Tokens.error
    readonly property color shadow: Kit.Tokens.shadowColor
    readonly property color overArt: Kit.Tokens.overArt
    readonly property color overArtScrim: Kit.Tokens.overArtScrim
    readonly property color ringGap: Kit.Tokens.ringGapColor

    // Derived text and divider roles.
    readonly property color body: Qt.rgba((Kit.Tokens.text.r + Kit.Tokens.muted.r) * 0.5, (Kit.Tokens.text.g + Kit.Tokens.muted.g) * 0.5, (Kit.Tokens.text.b + Kit.Tokens.muted.b) * 0.5, 1)
    readonly property color dim: Qt.alpha(Kit.Tokens.muted, 0.78)
    readonly property color hairline: Qt.alpha(Kit.Tokens.border, 0.38)
    readonly property color pillBorder: Qt.alpha(Kit.Tokens.border, 0.85)
    readonly property color track: Qt.alpha(Kit.Tokens.border, 0.32)
    readonly property color overAccentMuted: Qt.alpha(Kit.Tokens.overAccent, 0.78)

    // Derived corona, accent, and glow shades.
    readonly property color coronaSoft: Qt.alpha(Kit.Tokens.coronaColor, 0.55)
    readonly property color accentSoft: Qt.alpha(Kit.Tokens.accent, 0.50)
    readonly property color glowStrong: Qt.alpha(Kit.Tokens.glowColor, 0.50)
    readonly property color glowMid: Qt.alpha(Kit.Tokens.glowColor, 0.45)
    readonly property color glowRing: Qt.alpha(Kit.Tokens.glowColor, 0.35)
    readonly property color glowSoft: Qt.alpha(Kit.Tokens.glowColor, 0.30)
    readonly property color glowWeak: Qt.alpha(Kit.Tokens.glowColor, 0.22)
    readonly property color rose: Qt.alpha(Kit.Tokens.glowColor, 1.0)
    readonly property color coronaGlowMid: Qt.alpha(Kit.Tokens.coronaGlowColor, 0.70)
    readonly property color coronaGlowSoft: Qt.alpha(Kit.Tokens.coronaGlowColor, 0.40)

    // Derived translucent surfaces, scrims, shadows, and semantic fills.
    readonly property color glass: Qt.alpha(Kit.Tokens.surface, 0.78)
    readonly property color glassPill: Qt.alpha(Kit.Tokens.surface, 0.82)
    readonly property color glassStrong: Qt.alpha(Kit.Tokens.surface, 0.85)
    readonly property color scrimDialog: Qt.alpha(Kit.Tokens.scrim, 0.72)
    readonly property color scrimPower: Qt.alpha(Kit.Tokens.scrim, 0.74)
    readonly property color successFill: Qt.alpha(Kit.Tokens.success, 0.22)
    readonly property color attentionFill: Qt.alpha(Kit.Tokens.attention, 0.22)
    readonly property color errorFill: Qt.alpha(Kit.Tokens.error, 0.22)
    readonly property color shadowHeavy: Qt.alpha(Kit.Tokens.shadowColor, 0.60)
    readonly property color shadowStrong: Qt.alpha(Kit.Tokens.shadowColor, 0.55)
    readonly property color shadowHalf: Qt.alpha(Kit.Tokens.shadowColor, 0.50)
    readonly property color shadowMid: Qt.alpha(Kit.Tokens.shadowColor, 0.45)
    readonly property color shadowSoft: Qt.alpha(Kit.Tokens.shadowColor, 0.30)

    // Derived chrome fills and backdrop nebula tints.
    readonly property color rowSurface: Qt.rgba(Kit.Tokens.surface.r * 0.2 + Kit.Tokens.inputSurface.r * 0.8, Kit.Tokens.surface.g * 0.2 + Kit.Tokens.inputSurface.g * 0.8, Kit.Tokens.surface.b * 0.2 + Kit.Tokens.inputSurface.b * 0.8, 1)
    readonly property color checkerDark: Qt.rgba(Kit.Tokens.surfaceRaised.r * 0.75 + Kit.Tokens.border.r * 0.25, Kit.Tokens.surfaceRaised.g * 0.75 + Kit.Tokens.border.g * 0.25, Kit.Tokens.surfaceRaised.b * 0.75 + Kit.Tokens.border.b * 0.25, 1)
    readonly property color backdropGlow: Qt.alpha(Kit.Tokens.coronaGlowColor, 1.0)
    readonly property color backdropHalo: Qt.alpha(Kit.Tokens.coronaHaloColor, 1.0)
}
