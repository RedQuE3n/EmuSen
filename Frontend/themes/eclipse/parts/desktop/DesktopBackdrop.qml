import QtQuick
import EmuSen.Theme
import ".."

// Eclipse Desktop atmospheric sky backdrop with corner corona nebulae and subtle orbital limb.
Item {
    id: root
    required property real px
    required property real pixelScale
    property bool showLimb: false

    DesktopInk {
        id: ink
    }

    Backdrop {
        anchors.fill: parent
        ground: Tokens.surface
        pixelScale: root.pixelScale
        fields: [
            BackdropField {
                x: -140 * root.px
                y: -160 * root.px
                width: 680 * root.px
                height: 680 * root.px
                color: ink.backdropGlow
                blur: 140 * root.px
                opacity: 0.12
            },
            BackdropField {
                x: root.width - 520 * root.px
                y: -140 * root.px
                width: 700 * root.px
                height: 700 * root.px
                color: ink.backdropHalo
                blur: 150 * root.px
                opacity: 0.14
            },
            BackdropField {
                x: root.width * 0.25
                y: root.height - 320 * root.px
                width: 780 * root.px
                height: 620 * root.px
                color: ink.backdropGlow
                blur: 160 * root.px
                opacity: 0.08
            }
        ]
    }

    Limb {
        visible: root.showLimb
        x: -1200 * root.px
        y: -480 * root.px
        width: 1700 * root.px
        height: 2200 * root.px
        fill: "transparent"
        opacity: 0.55
    }
}
