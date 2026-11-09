pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Cores on the kit's tool screen (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7): every word, row and column from the ToolModel; A the row's action, X its page and Select the List card as cards over it, Start the plan, and the tool's questions in the question form.
Kit.NormalizedCanvas {
    id: screen
    required property ToolModel tool
    property int padFamily
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on the screen.
    signal backed()
    Kit.ToolScreen {
        id: frame
        objectName: "coresScreen"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: screen.inputs && !screen.tool.cardOpen
        padFamily: screen.padFamily
        // The question draws its own help bar over the screen's.
        helpShown: !screen.tool.asking
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        cellFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
        titleFont: Qt.font({ family: faces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
        captionFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontCaption })
        bodyFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontBody })
        controller.sections: screen.tool.sections
        controller.selectedSection: screen.tool.section
        controller.actionLines: screen.tool.actionLines
        title: screen.tool.title
        nameRole: "name"
        searchHint: screen.tool.searchHint
        facts: RowsModel {
            rows: screen.tool.facts
        }
        columns: RowsModel {
            rows: screen.tool.columns
        }
        rows: RowsModel {
            rows: screen.tool.rows
        }
        paneTitle: screen.tool.paneTitle
        paneCaption: screen.tool.paneCaption
        sentences: RowsModel {
            rows: screen.tool.sentences
        }
        summary: screen.tool.summary
        summaryTone: screen.tool.summaryTone
        progress: screen.tool.progress
        applyLabel: screen.tool.applyLabel
        applyEnabled: screen.tool.applyEnabled
        meter: screen.tool.meter
        meterWords: screen.tool.meterWords
        help: RowsModel {
            rows: screen.tool.help
        }
        onActivated: (index) => screen.tool.activate(index)
        onApplied: screen.tool.apply()
        onSearchTextChanged: screen.tool.search(frame.searchText)
        Connections {
            target: frame.list
            function onCurrentIndexChanged() { screen.tool.focus(frame.list.currentIndex) }
        }
        // The model's focus, moved by a run or an approval, followed by the list; a Binding, since the list's own moves write its index.
        Binding {
            frame.list.currentIndex: screen.tool.current
        }
        // The list takes the keys back from a card as it closes.
        Connections {
            target: screen.tool
            function onCardClosed() { frame.forceActiveFocus() }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { screen.tool.act(key) }
            function onSelectedSectionChanged() { screen.tool.chooseSection(frame.controller.selectedSection) }
            function onBackDeclined() { screen.backed() }
        }
        Kit.Question {
            id: question
            objectName: "coresQuestion"
            controller: frame.controller
            title: screen.tool.questionTitle
            sentences: screen.tool.questionSentences
            caption: screen.tool.questionCaption
            choices: screen.tool.questionChoices
            backIndex: screen.tool.questionBack
            padFamily: screen.padFamily
            titleFont: Qt.font({ family: faces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
            help: HelpModel {
                screen: HelpModel.Question
            }
            onChosen: (index) => screen.tool.answer(index)
        }
        // Opened each time the tool asks; a Binding, since B closing the popup would undo a plain one.
        Binding {
            question.visible: screen.tool.asking
        }
    }
    // A row's page under X and the List card under Select, on the Card form over the screen (EmuSen_Interface.md §15.5).
    MenuCard {
        id: card
        objectName: "coresCard"
        anchors.fill: parent
        visible: card.open || card.leaving
        open: screen.tool.cardOpen
        focus: screen.inputs && screen.tool.cardOpen
        padFamily: screen.padFamily
        rows: RowsModel {
            rows: screen.tool.cardRows
        }
        title: screen.tool.cardTitle
        breadcrumb: screen.tool.cardBreadcrumb
        current: screen.tool.cardCurrent
        paneLines: screen.tool.cardPaneLines
        paneMuted: screen.tool.cardPaneMuted
        help: HelpModel {
            screen: HelpModel.Card
        }
        onChosen: (index) => screen.tool.cardChoose(index)
        onStepped: (index, by, held) => screen.tool.cardStep(index, by)
        onFocused: (index) => screen.tool.cardFocus(index)
        onBacked: screen.tool.cardBack()
        onActed: (key) => screen.tool.cardBack()
    }
    // A job's events pulled while it runs, never pushed (EmuSen_Qt.md §1.1).
    Timer {
        interval: 100
        repeat: true
        running: screen.visible && screen.tool.busy
        onTriggered: screen.tool.poll()
    }
    Faces {
        id: faces
    }
}
