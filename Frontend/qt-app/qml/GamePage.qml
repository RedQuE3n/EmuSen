pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// A game's page on the kit's page frame: the hero and its clip, the cover, the facts, the actions, the five sections and their rows or tiles; every word and value is GamePageModel's (EmuSen_Qt.md §7.8).
Kit.NormalizedCanvas {
    id: screen
    required property GamePageModel page
    property int padFamily
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on the page, and the declared actions the screens answer (Start).
    signal backed()
    signal acted(string key)
    // The focus as the help bar reads it: an action's index, else -1 and the content's index.
    readonly property int actionFocused: frame.actionView.activeFocus ? frame.actionView.currentIndex : -1
    readonly property int contentFocused: frame.showsTiles ? frame.tileView.currentIndex : frame.rowView.currentIndex
    // Told as it moves, inside the model's own reset too, where the model keeps it until the reset ends (EmuSen_Qt.md §7.8.1).
    onActionFocusedChanged: screen.page.focus(screen.actionFocused, screen.contentFocused)
    onContentFocusedChanged: screen.page.focus(screen.actionFocused, screen.contentFocused)
    PageActionModel {
        id: actions
        actions: screen.page.actions
    }
    Kit.PageFrame {
        id: frame
        objectName: "pageFrame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: screen.inputs
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        hero: screen.page.hero
        clipSource: screen.page.clip
        // The clip plays while the page holds the focus, and stops when it loses it.
        clipPlaying: screen.visible && frame.activeFocus
        // Heard at the player's Volume, as the media viewer's clip is (R-20261111-100).
        clipMuted: false
        clipVolume: screen.page.clipVolume
        cover: screen.page.cover
        coverAspect: screen.page.coverAspect
        coverName: screen.page.coverName
        title: screen.page.title
        facts: screen.page.facts
        actionModel: actions
        tileModel: screen.page
        rowModel: screen.page
        showsTiles: screen.page.shows === 1
        showsText: screen.page.shows === 0
        contentWords: screen.page.words
        contentPicture: screen.page.picture
        controller.sections: screen.page.sections
        help: HelpModel {
            objectName: "pageHelp"
            screen: screen.page.help
        }
        onActed: (index) => screen.page.act(index)
        onActionStepped: (index, by) => screen.page.stepAction(index, by)
        onTileChosen: (index) => screen.page.choose(index)
        onRowChosen: (index) => screen.page.choose(index)
        onRowStepped: (index, by) => screen.page.step(index, by)
        Binding {
            frame.controller.selectedSection: screen.page.sections[screen.page.section]
        }
        Connections {
            target: frame.controller
            function onSelectedSectionChanged() { screen.page.section = screen.page.sections.indexOf(frame.controller.selectedSection) }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { screen.page.declared(key) }
        }
        Connections {
            target: screen.page
            function onPassed(key) { screen.acted(key) }
        }
        // A game's page opens with the focus on Play, wherever the last page left it (Mistress's `GamePage`).
        Connections {
            target: screen.page
            function onGameChanged() { frame.focusFirstAction() }
        }
        // The question's popup held the focus; the page's walk takes it back where it was.
        Connections {
            target: screen.page
            function onAnswered() { frame.forceActiveFocus() }
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { screen.backed() }
        }
        // Asked before deleting a state or hiding the game: No first and focused, B answers No (EmuSen_Interface.md §15.6).
        Kit.Question {
            id: question
            objectName: "pageQuestion"
            controller: frame.controller
            title: screen.page.questionTitle
            sentences: screen.page.questionSentences
            choices: screen.page.questionChoices
            backIndex: screen.page.questionBack
            padFamily: screen.padFamily
            titleFont: Qt.font({ family: faces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
            help: HelpModel {
                screen: HelpModel.Question
            }
            onChosen: (index) => screen.page.answer(index)
        }
        // Opened each time it is asked; a Binding, since B closing the popup would undo a plain one.
        Binding {
            question.visible: screen.page.asking
        }
    }
    Faces {
        id: faces
    }
}
