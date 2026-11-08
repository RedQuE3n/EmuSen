import QtQuick
import EmuSen.Kit as Kit

// A game shown without its cover: a card in its console's hue with its title, as Mistress draws a game it has no art for (EmuSen_Interface.md §21).
Rectangle {
    id: root
    required property string title
    required property real hue
    radius: Kit.Tokens.rowRadius
    gradient: Gradient {
        GradientStop { position: 0; color: Qt.hsla(root.hue / 360, 0.45, 0.32, 1) }
        GradientStop { position: 1; color: Qt.hsla(root.hue / 360, 0.5, 0.16, 1) }
    }
    border.color: Kit.Tokens.rule
    Text {
        anchors.fill: parent
        anchors.margins: 6
        text: root.title
        wrapMode: Text.Wrap
        elide: Text.ElideRight
        verticalAlignment: Text.AlignBottom
        color: Kit.Tokens.textStrong
        font.pixelSize: Kit.Tokens.fontBody
    }
}
