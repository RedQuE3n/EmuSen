pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// One match or action pill inside Scrape This Game's dialog card (EclipseGameScrape:47-50, 68).
Walkable {
    id: root
    required property real unit
    required property int index
    required property string label
    property real pixelScale: 1
    property color ringColor: "transparent"
    property color glowColor: "transparent"
    readonly property bool lit: root.activeFocus
    objectName: "scrapeRow." + root.index
    Accessible.name: root.label
    width: 872 * root.unit
    height: 68 * root.unit
    Glow {
        objectName: "scrapeRowHalo." + root.index
        anchors.fill: parent
        visible: root.lit
        radius: 34 * root.unit
        color: root.ringColor
        spread: 6 * root.unit
        pixelScale: root.pixelScale
    }
    Glow {
        objectName: "scrapeRowGlow." + root.index
        anchors.fill: parent
        visible: root.lit
        radius: 34 * root.unit
        color: root.glowColor
        blur: 46 * root.unit
        spread: 6 * root.unit
        pixelScale: root.pixelScale
    }
    Rectangle {
        objectName: "scrapeRowFill." + root.index
        anchors.fill: parent
        radius: 34 * root.unit
        color: root.lit ? Tokens.accent : "transparent"
    }
    Glyph {
        id: lead
        objectName: "scrapeRowGlyph." + root.index
        visible: root.lit
        unit: root.unit
        x: 12 * root.unit
        y: (root.height - height) / 2
        button: PadGlyph.Accept
        side: 46
        lead: true
        leadLetterSize: 21
        leadWeight: Font.DemiBold
        leadFill: Tokens.overAccent
        leadInk: Tokens.accent
    }
    Line {
        objectName: "scrapeRowName." + root.index
        unit: root.unit
        x: (root.lit ? 74 : 22) * root.unit
        y: (root.height - height) / 2
        width: root.width - x - 22 * root.unit
        boardLine: true
        text: root.label
        color: root.lit ? Tokens.overAccent : Tokens.text
        size: 24
        weight: root.lit ? Font.DemiBold : Font.Normal
        elide: Text.ElideRight
    }
}
