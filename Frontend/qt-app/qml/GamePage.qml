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
        Connections {
            target: frame.controller
            function onBackDeclined() { screen.backed() }
        }
    }
    Faces {
        id: faces
    }
}
