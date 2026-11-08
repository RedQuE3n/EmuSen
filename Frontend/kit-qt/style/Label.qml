import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A label in the tokens: body text in Text, links in the accent.
T.Label {
    id: control
    color: control.enabled ? Kit.Tokens.text : Kit.Tokens.muted
    linkColor: Kit.Tokens.accent
    font.pixelSize: Kit.Tokens.fontBody
}
