pragma Singleton
import QtQuick
import EmuSen.Kit as Kit

// The kit's tokens as a theme reads them, each read-only, in the current form and variant; the names are the kit's registry's (EmuSen_Qt.md §3.2).
QtObject {
    enum Variant { Dark, Light }
    enum Form { Desktop, BigScreen, Card }
    readonly property int form: Kit.Tokens.form
    readonly property int variant: Kit.Tokens.variant
    readonly property real textScale: Kit.Tokens.textScale
    readonly property string textSize: Kit.Tokens.textSize
    readonly property bool reducedMotion: Kit.Tokens.reducedMotion
    readonly property color surface: Kit.Tokens.surface
    readonly property color surfaceRaised: Kit.Tokens.surfaceRaised
    readonly property color inputSurface: Kit.Tokens.inputSurface
    readonly property color voidFill: Kit.Tokens.voidFill
    readonly property color scrim: Kit.Tokens.scrim
    readonly property color captionSurface: Kit.Tokens.captionSurface
    readonly property color border: Kit.Tokens.border
    readonly property color rule: Kit.Tokens.rule
    readonly property color text: Kit.Tokens.text
    readonly property color textStrong: Kit.Tokens.textStrong
    readonly property color muted: Kit.Tokens.muted
    readonly property color accent: Kit.Tokens.accent
    readonly property color overAccent: Kit.Tokens.overAccent
    readonly property color success: Kit.Tokens.success
    readonly property color attention: Kit.Tokens.attention
    readonly property color error: Kit.Tokens.error
    readonly property color overArt: Kit.Tokens.overArt
    readonly property color overArtScrim: Kit.Tokens.overArtScrim
    readonly property real fontDisplay: Kit.Tokens.fontDisplay
    readonly property real fontTitle: Kit.Tokens.fontTitle
    readonly property real fontStrong: Kit.Tokens.fontStrong
    readonly property real fontRow: Kit.Tokens.fontRow
    readonly property real fontBody: Kit.Tokens.fontBody
    readonly property real fontCaption: Kit.Tokens.fontCaption
    readonly property real fontGlyph: Kit.Tokens.fontGlyph
    readonly property real headerSize: Kit.Tokens.headerSize
    readonly property real base: Kit.Tokens.base
    readonly property real rowHeight: Kit.Tokens.rowHeight
    readonly property real controlHeight: Kit.Tokens.controlHeight
    readonly property real paneGutter: Kit.Tokens.paneGutter
    readonly property real paneGap: Kit.Tokens.paneGap
    readonly property real cardWidth: Kit.Tokens.cardWidth
    readonly property real titleRow: Kit.Tokens.titleRow
    readonly property real helpBar: Kit.Tokens.helpBar
    readonly property real tileWidth: Kit.Tokens.tileWidth
    readonly property real tileHeight: Kit.Tokens.tileHeight
    readonly property real tileGap: Kit.Tokens.tileGap
    readonly property real rowRadius: Kit.Tokens.rowRadius
    readonly property real tileRadius: Kit.Tokens.tileRadius
    readonly property real safeAreaX: Kit.Tokens.safeAreaX
    readonly property real safeAreaY: Kit.Tokens.safeAreaY
    readonly property real ringThickness: Kit.Tokens.ringThickness
    readonly property real ringOffset: Kit.Tokens.ringOffset
    readonly property real barWidth: Kit.Tokens.barWidth
    readonly property real tileScale: Kit.Tokens.tileScale
    readonly property real hoverBorder: Kit.Tokens.hoverBorder
    readonly property int instant: Kit.Tokens.instant
    readonly property int quick: Kit.Tokens.quick
    readonly property int enter: Kit.Tokens.enter
    readonly property int exit: Kit.Tokens.exit
    readonly property int pan: Kit.Tokens.pan
    readonly property int reel: Kit.Tokens.reel
    readonly property int fade: Kit.Tokens.fade
    readonly property int poster: Kit.Tokens.poster
    readonly property int quickFade: Kit.Tokens.quickFade
    readonly property int enterFade: Kit.Tokens.enterFade
    readonly property int exitFade: Kit.Tokens.exitFade
    readonly property int panFade: Kit.Tokens.panFade
    readonly property int reelFade: Kit.Tokens.reelFade
    readonly property int posterFade: Kit.Tokens.posterFade
    readonly property int instantCurve: Kit.Tokens.instantCurve
    readonly property int quickCurve: Kit.Tokens.quickCurve
    readonly property int enterCurve: Kit.Tokens.enterCurve
    readonly property int exitCurve: Kit.Tokens.exitCurve
    readonly property int panCurve: Kit.Tokens.panCurve
    readonly property int reelCurve: Kit.Tokens.reelCurve
    readonly property int fadeCurve: Kit.Tokens.fadeCurve
    readonly property int posterCurve: Kit.Tokens.posterCurve
    readonly property int breathDelay: Kit.Tokens.breathDelay
    readonly property int breathPeriod: Kit.Tokens.breathPeriod
    readonly property real breathLow: Kit.Tokens.breathLow
    readonly property bool breathes: Kit.Tokens.breathes
}
