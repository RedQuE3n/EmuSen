pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A console as a star of its maker's figure: its body (72, focused 96; compact 56 and 80) with the circle ring and three sparkles when focused, its name above a raised body or below a lowered one, and in the full tier its count with the engine mark (constellation.md §3).
ChartNode {
    id: body
    required property int index
    required property string key
    required property string name
    required property string displayName
    required property string maker
    required property int year
    required property int count
    required property real hue
    required property string body
    required property string engineName
    required property string engineKind
    required property string engineMark
    required property bool libretro
    required property int group
    readonly property string facts: Words.facts(body.maker, body.year, body.count, Words.runsOn(body.engineName, body.engineKind))
    readonly property real rest: body.layout ? body.layout.body : 72
    readonly property real size: body.activeFocus ? body.rest + 24 : body.rest
    readonly property real box: body.rest + 24
    readonly property bool raised: body.layout ? body.layout?.raised[body.index] ?? false : false
    readonly property bool full: body.layout ? !body.layout.compact : true
    nodeIndex: body.index
    figure: body.group
    objectName: "body." + body.index
    Accessible.name: body.name
    Accessible.description: body.facts
    x: body.cx - body.box / 2
    y: body.cy - body.box / 2
    width: body.box
    height: body.box
    z: body.activeFocus ? 1 : 0
    Body {
        id: world
        objectName: "world." + body.index
        visible: body.inView
        x: (body.box - body.size) / 2
        y: (body.box - body.size) / 2
        width: body.size
        height: body.size
        world: body.body
        Behavior on width { NumberAnimation { duration: Tokens.quick; easing.type: Tokens.quickCurve } }
        Behavior on height { NumberAnimation { duration: Tokens.quick; easing.type: Tokens.quickCurve } }
    }
    Rectangle {
        objectName: "bodyRing." + body.index
        visible: body.activeFocus
        anchors.centerIn: parent
        width: body.box + 14
        height: body.box + 14
        radius: width / 2
        color: "transparent"
        border.color: Tokens.accent
        border.width: Tokens.ringThickness
    }
    Repeater {
        model: body.activeFocus ? 3 : 0
        Glyph {
            required property int index
            readonly property real side: index === 1 ? 1 : -1
            readonly property real share: index === 0 ? 0.08 : (index === 1 ? 0.43 : 0.88)
            width: index === 0 ? 13 : (index === 1 ? 10 : 7)
            height: width
            x: side < 0 ? world.x - width - 2 : world.x + world.width + 2
            y: world.y + share * world.height
            path: Marks.sparkle
            filled: true
            ink: Tokens.textStrong
        }
    }
    Column {
        objectName: "bodyLabel." + body.index
        visible: body.inView
        x: (body.box - (body.layout?.labelWidth ?? 140)) / 2
        y: body.raised ? -height : body.box
        width: body.layout?.labelWidth ?? 140
        Text {
            objectName: "bodyName." + body.index
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: body.name
            elide: Text.ElideRight
            color: Tokens.textStrong
            font.family: body.faces?.condensed ?? ""
            font.weight: Font.Medium
            font.pixelSize: 20
            font.letterSpacing: 1.2
            font.capitalization: Font.AllUppercase
            lineHeight: 24
            lineHeightMode: Text.FixedHeight
        }
        Row {
            visible: body.full
            anchors.horizontalCenter: parent.horizontalCenter
            spacing: 6
            Text {
                objectName: "bodyCount." + body.index
                text: Words.games(body.count)
                color: Tokens.muted
                font.family: body.faces?.regular ?? ""
                font.pixelSize: 19
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
            Scene {
                anchors.verticalCenter: parent.verticalCenter
                width: 13
                height: 13
                EngineMark {
                    anchors.fill: parent
                    mark: body.engineMark
                    libretro: body.libretro
                    ink: body.libretro ? Tokens.muted : Qt.hsla(body.hue / 360, Tokens.variant === Tokens.Light ? 0.6 : 0.5, Tokens.variant === Tokens.Light ? 0.3 : 0.72, 1)
                }
            }
        }
    }
}
