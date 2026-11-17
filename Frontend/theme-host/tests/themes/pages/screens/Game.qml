import QtQuick
import EmuSen.Theme

// The pages test theme's game page: every member of Page as plain text, and each call of Actions a test asks by `command`.
ThemeScreen {
    id: root
    objectName: "pagesGame"
    role: ThemeScreen.Game
    focus: true
    // A call and its arguments, split by spaces: "pageAct 0"; `took` is what Actions answered.
    property string command: ""
    property string took: ""
    onCommandChanged: {
        if (root.command.length === 0)
            return
        const parts = root.command.split(" ")
        const args = parts.slice(1).map(a => isNaN(Number(a)) ? a : Number(a))
        root.took = String(Actions[parts[0]](...args))
    }
    Text { objectName: "gameReady"; text: Page.ready }
    Text { objectName: "gameKey"; y: 20; text: Page.key }
    Text { objectName: "gameWords"; y: 40; text: [Page.title, Page.facts, Page.cover, Page.hero, Page.clip, Page.words, Page.picture].join("|") }
    Text { objectName: "gameSections"; y: 60; text: Page.sections.join("|") + "|" + Page.section + "|" + Page.shows + "|" + Page.coverArt }
    Text { objectName: "gameFocus"; y: 80; text: Page.focusedAction + " " + Page.focusedContent + " " + Page.help }
    Text {
        objectName: "gameQuestion"
        y: 100
        text: [Page.asking, Page.questionTitle, Page.questionSentences.join("/"), Page.questionChoices.join("/"), Page.questionBack].join("|")
    }
    Row {
        y: 120
        Repeater {
            model: Help.entries(Page.help)
            delegate: Text {
                required property string label
                required property int index
                objectName: "gameHelpEntry" + index
                text: label
            }
        }
    }
    Column {
        y: 140
        Repeater {
            objectName: "gameActions"
            model: Page.actions
            delegate: Text {
                required property int index
                required property string label
                required property string value
                required property int button
                required property bool filled
                required property bool stepped
                required property bool stepLeft
                required property bool stepRight
                objectName: "gameAction" + index
                text: [label, value, button, filled, stepped, stepLeft, stepRight].join("|")
            }
        }
        Repeater {
            objectName: "gameRows"
            model: Page.rows
            delegate: Text {
                required property int index
                required property string key
                required property string label
                required property string value
                required property string line
                required property string picture
                required property bool steps
                required property bool opens
                required property bool marked
                objectName: "gameRow" + index
                text: [key, label, value, line, picture, steps, opens, marked].join("|")
            }
        }
    }
}
