pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// Centered bottom help bar for Eclipse circular dialogs (EclipseDialog:44-47, EclipsePower:41-44).
Item {
    id: footer
    required property Kit.FrameController controller
    required property var model
    required property int family
    property real unit: 1
    implicitWidth: bar.implicitWidth
    implicitHeight: bar.implicitHeight
    width: implicitWidth
    height: implicitHeight
    Kit.HelpBar {
        id: bar
        objectName: footer.objectName
        controller: footer.controller
        model: footer.model
        family: footer.family
        fitted: true
        ruled: false
        color: "transparent"
        labelColor: "#D8DBE6"
        iconColor: Kit.Tokens.muted
        leadStyle: Kit.GlyphStyles.Filled
        leadColor: Kit.Tokens.accent
        entrySpacing: 34 * footer.unit
        glyphGap: 10 * footer.unit
        glyphSize: 34 * footer.unit
        snapped: false
        label: Component {
            CardLine {
                objectName: "helpWords-" + text
                unit: footer.unit
                boardLine: true
                size: 21
            }
        }
        glyph: Component {
            CardGlyph {
                unit: footer.unit
                side: 34
                ink: "#D8DBE6"
            }
        }
    }
}
