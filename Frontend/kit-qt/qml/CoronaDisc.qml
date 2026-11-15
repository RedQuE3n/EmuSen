pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The eclipse's dark disc in its box, its rim and its glows round it, drawn past the box as a box-shadow is; every measure a token or a property (EmuSen_Kit.md §7.116).
Item {
    id: root
    // The disc's fill.
    property color fill: Kit.Tokens.surface
    // A game's corona; transparent, the default, where none is given (EmuSen_Kit.md §7.116).
    property color corona: "transparent"
    readonly property bool gameCorona: root.corona.a > 0
    // The rim's colour: the game's corona where one is given, the look's otherwise.
    readonly property color rimColor: root.gameCorona ? root.corona : Kit.Tokens.coronaColor
    // The default glows' colours: the game's corona at each glow token's own strength (R-20261114-38), the tokens' otherwise.
    readonly property color glowColor: root.gameCorona ? Qt.rgba(root.corona.r, root.corona.g, root.corona.b, Kit.Tokens.coronaGlowColor.a) : Kit.Tokens.coronaGlowColor
    readonly property color haloColor: root.gameCorona ? Qt.rgba(root.corona.r, root.corona.g, root.corona.b, Kit.Tokens.coronaHaloColor.a) : Kit.Tokens.coronaHaloColor
    // The rim's width outside the disc.
    property real rim: Kit.Tokens.coronaRim
    // The glows under the rim, the first on top, as a box-shadow lists them.
    property list<Kit.CoronaGlow> glows: [
        Kit.CoronaGlow {
            blur: Kit.Tokens.coronaGlowBlur
            spread: Kit.Tokens.coronaGlowSpread
            color: root.glowColor
        },
        Kit.CoronaGlow {
            blur: Kit.Tokens.coronaHaloBlur
            spread: Kit.Tokens.coronaHaloSpread
            color: root.haloColor
        }
    ]
    readonly property real diameter: Math.min(root.width, root.height)
    Repeater {
        model: root.glows
        Kit.Glow {
            id: glow
            required property int index
            required property Kit.CoronaGlow modelData
            objectName: "coronaGlow." + glow.index
            anchors.centerIn: parent
            width: root.diameter
            height: root.diameter
            // Under the rim and the disc, each glow under those before it.
            z: -1 - glow.index
            radius: root.diameter / 2
            blur: glow.modelData.blur
            spread: glow.modelData.spread
            color: glow.modelData.color
        }
    }
    Rectangle {
        objectName: "coronaRim"
        anchors.centerIn: parent
        width: root.diameter + 2 * root.rim
        height: root.diameter + 2 * root.rim
        radius: width / 2
        color: root.rimColor
        visible: root.rim > 0
    }
    Rectangle {
        objectName: "coronaDisc"
        anchors.centerIn: parent
        width: root.diameter
        height: root.diameter
        radius: width / 2
        color: root.fill
    }
}
