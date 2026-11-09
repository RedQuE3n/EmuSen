import QtQuick
import EmuSen.Kit as Kit

// Words in a tone: the token the tone names, so a model hands what its words say and never a colour (EmuSen_Kit.md §7.43).
Text {
    id: root
    property int tone: Kit.Tones.Text
    color: root.tone === Kit.Tones.Muted ? Kit.Tokens.muted
        : root.tone === Kit.Tones.Strong ? Kit.Tokens.textStrong
        : root.tone === Kit.Tones.Accent ? Kit.Tokens.accent
        : root.tone === Kit.Tones.Success ? Kit.Tokens.success
        : root.tone === Kit.Tones.Attention ? Kit.Tokens.attention
        : root.tone === Kit.Tones.Error ? Kit.Tokens.error
        : Kit.Tokens.text
}
