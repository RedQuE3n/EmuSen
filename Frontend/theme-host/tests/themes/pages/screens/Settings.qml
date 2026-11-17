import QtQuick
import EmuSen.Theme

// The pages test theme's Settings: every member of Settings as plain text, and each call of Actions a test asks by `command`.
ThemeScreen {
    id: root
    objectName: "pagesSettings"
    role: ThemeScreen.Settings
    focus: true
    // A call and its arguments, split by spaces: "settingsStep 3 1"; `took` is what Actions answered.
    property string command: ""
    property string took: ""
    onCommandChanged: {
        if (root.command.length === 0)
            return
        const parts = root.command.split(" ")
        const args = parts.slice(1).map(a => isNaN(Number(a)) ? a : Number(a))
        root.took = String(Actions[parts[0]](...args))
    }
    Text { objectName: "setSections"; text: Settings.sections.join("|") }
    Text { objectName: "setSection"; y: 20; text: Settings.section }
    Text { objectName: "setTrail"; y: 40; text: Settings.trail.join("|") }
    Text { objectName: "setPage"; y: 60; text: Settings.page }
    Text { objectName: "setFocused"; y: 80; text: Settings.focused + " of " + Settings.count }
    Text { objectName: "setHelp"; y: 100; text: Settings.help }
    Text {
        objectName: "setPane"
        y: 120
        text: [Settings.sentence, Settings.lines.join("/"), Settings.muted.join("/"), Settings.tones.join("/"), Settings.preview, Settings.caption,
            Settings.meter, Settings.samples.join("/"), Settings.sampleScales.join("/")].join("|")
    }
    Row {
        y: 140
        Repeater {
            model: Help.entries(Settings.help)
            delegate: Text {
                required property string label
                required property int index
                objectName: "setHelpEntry" + index
                text: label
            }
        }
    }
    Column {
        y: 160
        Repeater {
            objectName: "setRows"
            model: Settings.rows
            delegate: Text {
                required property int index
                required property string key
                required property string label
                required property string about
                required property string value
                required property int kind
                required property bool isSwitch
                required property bool switchOn
                required property bool held
                required property bool marked
                required property bool code
                required property real minimum
                required property real maximum
                required property real step
                required property real amount
                required property string consoleName
                required property string system
                required property int games
                required property string engine
                objectName: "setRow" + index
                text: [key, label, about, value, kind, isSwitch, switchOn, held, marked, code, minimum, maximum, step, amount, consoleName, system, games, engine].join("|")
            }
        }
    }
}
