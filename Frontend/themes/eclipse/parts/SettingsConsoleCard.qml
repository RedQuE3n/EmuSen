pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A console page's Core planet card in the About panel: 92 px planet or checkerboard beside the core's name and subtitle (EclipseSettingsConsole:67-70).
Rectangle {
    id: root
    required property real unit
    property real pixelScale: 1
    objectName: "asideConsoleCard"
    width: parent ? parent.width : 0
    height: 148 * root.unit
    radius: 22 * root.unit
    color: own.facesGround
    border.width: root.unit
    border.color: own.facesLine
    SettingsInk {
        id: own
    }
    PlanetSphere {
        id: planet
        objectName: "asideCorePlanet"
        unit: root.unit
        planet: Settings.cardPlanet
        side: 92
        corner: 18
        glowBlur: 34
        glowSpread: 6
        glowAlpha: 0x66 / 255
        pixelScale: root.pixelScale
        x: 28 * root.unit
        y: 28 * root.unit
    }
    Column {
        x: (28 + 92 + 22) * root.unit
        y: (root.height - height) / 2
        width: root.width - x - 28 * root.unit
        spacing: 6 * root.unit
        Line {
            objectName: "asideCoreName"
            unit: root.unit
            boardLine: true
            width: parent.width
            text: Settings.cardTitle
            color: Tokens.text
            elide: Text.ElideRight
            size: 30
            weight: Tokens.weightTitle
        }
        Line {
            objectName: "asideCoreSub"
            unit: root.unit
            boardLine: true
            width: parent.width
            text: Settings.cardSubtitle
            color: Tokens.muted
            elide: Text.ElideRight
            size: 18
        }
    }
}
