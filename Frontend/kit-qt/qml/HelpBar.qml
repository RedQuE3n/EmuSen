pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The help bar: every entry a button that raises its press, its glyph the pad's own in the pad's family and the style's set, so a pointer has every action the bar names (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.12, §7.28).
Item {
    id: root
    // A frame's controller offered each press; optional, since a bar outside a frame takes its presses from `pressed`.
    property Kit.FrameController controller: null
    property alias model: entries.model
    property real leftInset: 0
    property int family: Kit.Families.Generic
    property int glyphStyle: Kit.GlyphStyles.Outlined
    // The labels' face, the gap between entries and the gap from a glyph to its label; a theme may set its own.
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    property real entrySpacing: Kit.Tokens.paneGap
    property real glyphGap: Kit.Tokens.base * 2
    property color labelColor: Kit.Tokens.text
    property color iconColor: Kit.Tokens.text
    // Each entry's glyph, gap and label at this share of their own size.
    property real entryScale: 1
    // A glyph's side in pixels before entryScale; 0 is the glyph's own size.
    property real glyphSize: 0
    // The bar's own colour and radius, drawn by its backing; the paddings are its box's around the entries.
    property color color: root.glass ? Qt.alpha(Kit.Tokens.inputSurface, root.glassOpacity) : Kit.Tokens.surface
    property alias radius: backing.radius
    property real horizontalPadding: 0
    property real verticalPadding: 0
    // A box that hugs its entries, as a theme's help element; off, the band on the foot.
    property bool fitted: false
    // The rule along its top; a host that draws the band itself turns it off.
    property bool ruled: true
    // At night the band is glass: the input surface and its Border rule at 0.72 over the sky, as LunaP's HelpBand draws it (EmuSen_Kit.md §7.25).
    property bool glass: false
    readonly property real glassOpacity: 0.72
    signal pressed(int button)
    implicitWidth: root.leftInset + 2 * root.horizontalPadding + row.implicitWidth
    implicitHeight: root.fitted ? row.implicitHeight + 2 * root.verticalPadding : Kit.Tokens.helpBar
    // A clear colour draws no backing at all: on Vulkan a clear Rectangle cleared its strip to alpha 0 (EmuSen_Kit.md §7.28).
    Rectangle {
        id: backing
        anchors.fill: parent
        visible: root.color.a > 0
        color: root.color
    }
    Rectangle {
        objectName: "helpGlassRule"
        visible: root.glass && !root.fitted
        width: parent.width
        height: Kit.Tokens.hoverBorder
        color: Qt.alpha(Kit.Tokens.border, root.glassOpacity)
    }
    Rectangle {
        visible: root.ruled && !root.glass && !root.fitted
        anchors.top: parent.top
        width: parent.width
        height: Kit.Tokens.hoverBorder
        color: Kit.Tokens.rule
    }
    Row {
        id: row
        x: root.leftInset + root.horizontalPadding
        anchors.verticalCenter: parent.verticalCenter
        spacing: root.entrySpacing
        Repeater {
            id: entries
            Button {
                id: entry
                required property int button
                required property string label
                required property var model
                readonly property int pair: entry.model.pair ?? -1
                // A theme's own picture for the button, drawn in the glyph's place.
                readonly property string picture: entry.model.icon ?? ""
                // An entry whose action the host would decline: drawn at 38% and not pressed (EmuSen_Kit.md §7.28).
                readonly property bool live: entry.model.enabled ?? true
                objectName: "help-" + entry.label
                flat: true
                leftPadding: 0
                rightPadding: 0
                implicitWidth: entry.implicitContentWidth
                focusPolicy: Qt.NoFocus
                enabled: entry.live
                opacity: entry.live ? 1 : 0.38
                text: entry.label
                // In a fitted box an entry is as high as its content, as it is as wide; on the band the style's height stands.
                Binding on implicitHeight {
                    when: root.fitted
                    value: entry.implicitContentHeight
                }
                onClicked: {
                    root.pressed(entry.button)
                    if (root.controller)
                        root.controller.offer(entry.button)
                }
                contentItem: Row {
                    spacing: root.glyphGap * root.entryScale
                    Image {
                        objectName: "icon-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        visible: entry.picture.length > 0
                        width: glyph.width
                        height: glyph.height
                        sourceSize: Qt.size(glyph.width, glyph.height)
                        fillMode: Image.PreserveAspectFit
                        source: entry.picture
                    }
                    Kit.PadGlyph {
                        id: glyph
                        objectName: "glyph-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        visible: entry.picture.length === 0
                        width: (root.glyphSize > 0 ? root.glyphSize : implicitWidth) * root.entryScale
                        height: (root.glyphSize > 0 ? root.glyphSize : implicitHeight) * root.entryScale
                        family: root.family
                        style: root.glyphStyle
                        ink: root.iconColor
                        button: entry.button
                        swap: Kit.PadRouter.swapAcceptBack
                    }
                    Kit.PadGlyph {
                        objectName: "glyph2-" + entry.label
                        anchors.verticalCenter: parent.verticalCenter
                        visible: entry.pair >= 0
                        width: (root.glyphSize > 0 ? root.glyphSize : implicitWidth) * root.entryScale
                        height: (root.glyphSize > 0 ? root.glyphSize : implicitHeight) * root.entryScale
                        family: root.family
                        style: root.glyphStyle
                        ink: root.iconColor
                        button: Math.max(0, entry.pair)
                        swap: Kit.PadRouter.swapAcceptBack
                    }
                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: entry.label
                        color: root.labelColor
                        font.family: root.labelFont.family
                        font.weight: root.labelFont.weight
                        font.italic: root.labelFont.italic
                        font.capitalization: root.labelFont.capitalization
                        font.letterSpacing: root.labelFont.letterSpacing
                        font.pixelSize: root.labelFont.pixelSize * root.entryScale
                    }
                }
            }
        }
    }
}
