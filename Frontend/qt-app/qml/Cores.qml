pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Dialogs
import EmuSen.Kit as Kit
import EmuSen.Models

// A tool on the kit's tool screen, Cores and the tools of jobs alike (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7): every word, row and column from the ToolModel; A the row's action, X its page and Select the List card as cards over it, Start the plan, and the tool's questions in the question form.
Kit.NormalizedCanvas {
    id: screen
    required property ToolModel tool
    property int padFamily
    property Clock clock: null
    property string clockText: ""
    property int battery: -1
    readonly property bool eclipseMode: Kit.Tokens.coronaRim > 0
    readonly property int eclipseScreen: screen.eclipseMode ? screen.tool.eclipseScreen : 0
    readonly property bool eclipseCoresActive: screen.eclipseScreen === 1
    readonly property bool eclipseAnyActive: screen.eclipseScreen > 0
    readonly property real boardUnit: Kit.CanvasUnits.designHeight / 1200
    readonly property string shownClock: screen.clock ? screen.clock.shortTime : (screen.clockText.length > 0 ? screen.clockText : fallbackClock.shortTime)
    readonly property int shownBattery: screen.battery >= 0 ? screen.battery : fallbackBattery.level
    readonly property Item activeView: screen.eclipseScreen === 1 ? eclipseView : screen.eclipseScreen === 2 ? scraperView : screen.eclipseScreen === 3 ? scrapeStatusView : screen.eclipseScreen === 4 ? libraryView : screen.eclipseScreen === 5 ? packagesLoader : screen.eclipseScreen === 6 ? packageLayersLoader : screen.eclipseScreen === 7 ? packagePageLoader : frame
    readonly property var activeController: screen.eclipseScreen === 1 ? eclipseView.controller : screen.eclipseScreen === 2 ? scraperView.controller : screen.eclipseScreen === 3 ? scrapeStatusView.controller : screen.eclipseScreen === 4 ? libraryView.controller : screen.eclipseScreen === 5 && packagesLoader.item ? (packagesLoader.item as PackagesEclipse).controller : screen.eclipseScreen === 6 && packageLayersLoader.item ? (packageLayersLoader.item as PackageLayersEclipse).controller : screen.eclipseScreen === 7 && packagePageLoader.item ? (packagePageLoader.item as PackagePageEclipse).controller : frame.controller
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on the screen.
    signal backed()
    Clock {
        id: fallbackClock
    }
    Battery {
        id: fallbackBattery
        platformLibrary: screen.visible && screen.eclipseScreen > 1 && screen.battery < 0 ? screen.tool.platformLibrary : ""
    }
    Timer {
        interval: 60000
        repeat: true
        triggeredOnStart: true
        running: screen.visible && screen.eclipseScreen > 1 && !screen.clock && screen.clockText.length === 0
        onTriggered: fallbackClock.tick()
    }
    Timer {
        interval: fallbackBattery.every
        repeat: true
        triggeredOnStart: true
        running: screen.visible && screen.eclipseScreen > 1 && screen.battery < 0
        onTriggered: fallbackBattery.read()
    }
    Kit.ToolScreen {
        id: frame
        objectName: `${screen.objectName}Screen`
        // Under a card or a question the screen is drawn and nothing on it is reached: a backdrop to the gate, as ThemeHost's screens are.
        property bool backdrop: screen.tool.cardOpen || screen.tool.asking
        anchors.fill: parent
        visible: !screen.eclipseAnyActive
        form: Kit.Forms.BigScreen
        // While searching, the rows hold the keys once a shoulder has given them the focus (R-20261111-31).
        focus: !screen.eclipseAnyActive && screen.inputs && !screen.tool.cardOpen && !screen.tool.searchKeys
        padFamily: screen.padFamily
        // The question draws its own help bar over the screen's.
        helpShown: !screen.eclipseAnyActive && !screen.tool.asking && !screen.tool.searching && !screen.tool.cardOpen
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        cellFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
        titleFont: Qt.font({ family: Kit.Tokens.faceLabel, pixelSize: Kit.Tokens.fontTitle, weight: Kit.Tokens.weightTitle })
        captionFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontCaption })
        bodyFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
        controller.sections: screen.tool.sections
        controller.selectedSection: screen.tool.section
        controller.actionLines: screen.tool.actionLines
        title: screen.tool.title
        nameRole: "name"
        searchHint: screen.tool.searchHint
        searchText: screen.tool.searchText
        keysTop: screen.tool.searching ? keyboard.y : -1
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
            function onListFocused() { screen.activeView.forceActiveFocus() }
            function onSearchOpened() { searchFrame.forceActiveFocus() }
            function onEclipseScreenChanged() { screen.activeView.forceActiveFocus() }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { screen.tool.act(key) }
            function onSelectedSectionChanged() { screen.tool.chooseSection(frame.controller.selectedSection) }
            function onBackDeclined() { screen.backed() }
        }
    }
    CoresEclipse {
        id: eclipseView
        anchors.fill: parent
        visible: screen.eclipseCoresActive && !screen.tool.cardOpen
        focus: screen.eclipseCoresActive && screen.inputs && !screen.tool.cardOpen
        tool: screen.tool
        padFamily: screen.padFamily
        pixelScale: screen.unit
        battery: screen.shownBattery
        clockText: screen.shownClock
        onBacked: screen.backed()
    }
    ScraperEclipse {
        id: scraperView
        anchors.fill: parent
        visible: screen.eclipseScreen === 2 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 2 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking && !screen.tool.searchKeys
        tool: screen.tool
        padFamily: screen.padFamily
        pixelScale: screen.unit
        battery: screen.shownBattery
        clockText: screen.shownClock
        onBacked: screen.backed()
    }
    ScrapeStatusEclipse {
        id: scrapeStatusView
        anchors.fill: parent
        visible: screen.eclipseScreen === 3 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 3 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking
        tool: screen.tool
        padFamily: screen.padFamily
        pixelScale: screen.unit
        battery: screen.shownBattery
        clockText: screen.shownClock
    }
    LibraryManagerEclipse {
        id: libraryView
        anchors.fill: parent
        visible: screen.eclipseScreen === 4 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 4 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking && !screen.tool.searchKeys
        tool: screen.tool
        padFamily: screen.padFamily
        pixelScale: screen.unit
        battery: screen.shownBattery
        clockText: screen.shownClock
        onBacked: screen.backed()
    }
    Loader {
        id: packagesLoader
        anchors.fill: parent
        active: screen.eclipseMode && screen.eclipseScreen >= 5
        visible: screen.eclipseScreen === 5 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 5 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking && !screen.tool.searchKeys
        onLoaded: if (focus) forceActiveFocus()
        sourceComponent: PackagesEclipse {
            focus: true
            tool: screen.tool
            padFamily: screen.padFamily
            pixelScale: screen.unit
            battery: screen.shownBattery
            clockText: screen.shownClock
            onBacked: screen.backed()
        }
    }
    Loader {
        id: packageLayersLoader
        anchors.fill: parent
        active: screen.eclipseMode && screen.eclipseScreen >= 5
        visible: screen.eclipseScreen === 6 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 6 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking && !screen.tool.searchKeys
        onLoaded: if (focus) forceActiveFocus()
        sourceComponent: PackageLayersEclipse {
            focus: true
            tool: screen.tool
            padFamily: screen.padFamily
            pixelScale: screen.unit
            battery: screen.shownBattery
            clockText: screen.shownClock
        }
    }
    Loader {
        id: packagePageLoader
        anchors.fill: parent
        active: screen.eclipseMode && screen.eclipseScreen >= 5
        visible: screen.eclipseScreen === 7 && !screen.tool.cardOpen
        focus: screen.eclipseScreen === 7 && screen.inputs && !screen.tool.cardOpen && !screen.tool.asking && !screen.tool.searchKeys
        onLoaded: if (focus) forceActiveFocus()
        sourceComponent: PackagePageEclipse {
            focus: true
            tool: screen.tool
            padFamily: screen.padFamily
            pixelScale: screen.unit
            battery: screen.shownBattery
            clockText: screen.shownClock
        }
    }
    FileDialog {
        id: patchFileDialog
        objectName: "patchFileDialog"
        title: screen.tool.fileDialogTitle
        currentFolder: screen.tool.fileDialogFolder
        nameFilters: screen.tool.fileDialogFilters
        onAccepted: screen.tool.addPatchUrl(patchFileDialog.selectedFile)
        onRejected: screen.tool.cancelPatchFile()
    }
    Connections {
        target: screen.tool
        function onChoosePatchFile() { patchFileDialog.open() }
    }
    // The search's words typed on the docked keyboard, from the List card's Search (EmuSen_Interface.md §15.2), as the grid's search types them.
    Kit.KeyboardModel {
        id: typist
        onTextChanged: screen.tool.search(typist.text)
        onClosed: screen.tool.endSearch()
    }
    Connections {
        target: screen.tool
        function onSearchingChanged() { typist.setText(screen.tool.searchText, screen.tool.searchText.length) }
    }
    Kit.Frame {
        id: searchFrame
        objectName: `${screen.objectName}Search`
        anchors.fill: parent
        visible: screen.tool.searching
        surfaceDrawn: false
        focus: screen.inputs && screen.tool.searchKeys
        // Its help stands while the keys hold the focus, as the grid's search's does (EmuSen_Qt.md §7.6).
        helpShown: screen.tool.searchKeys
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.ToolSearch
        }
        // The keyboard's own actions, and the screen's: the shoulders to the rows.
        Connections {
            target: searchFrame.controller
            function onActionInvoked(key) { typist.act(key) }
        }
        Connections {
            target: searchFrame.controller
            function onActionInvoked(key) { screen.tool.act(key) }
        }
        Item {
            x: -searchFrame.safeX
            y: -searchFrame.safeY
            width: screen.width
            height: screen.height
            Kit.Keyboard {
                id: keyboard
                objectName: `${screen.objectName}Keyboard`
                model: typist
                y: 398
                width: screen.width
                focus: true
                onActiveFocusChanged: if (screen.tool.searching && keyboard.activeFocus !== screen.tool.searchKeys) screen.tool.keysFocused(keyboard.activeFocus)
                labelFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
            }
        }
    }
    Kit.Backdrop {
        objectName: "coresCardBackdrop"
        anchors.fill: parent
        visible: screen.eclipseAnyActive && screen.tool.cardOpen
        ground: "#0A1024"
        pixelScale: screen.unit
        fields: [
            Kit.BackdropField { x: 1060 * screen.boardUnit; y: -140 * screen.boardUnit; width: 900 * screen.boardUnit; height: 760 * screen.boardUnit; color: "#1FC9A8"; blur: 90 * screen.boardUnit; opacity: 0.85 },
            Kit.BackdropField { x: 1380 * screen.boardUnit; y: 420 * screen.boardUnit; width: 760 * screen.boardUnit; height: 760 * screen.boardUnit; color: "#6A4CF0"; blur: 110 * screen.boardUnit; opacity: 0.9 },
            Kit.BackdropField { x: 900 * screen.boardUnit; y: -260 * screen.boardUnit; width: 700 * screen.boardUnit; height: 520 * screen.boardUnit; color: "#1D6BF2"; blur: 100 * screen.boardUnit; opacity: 0.8 }
        ]
    }
    // A row's page under X and the List card under Select, on the Card form over the screen (EmuSen_Interface.md §15.5).
    MenuCard {
        id: card
        objectName: `${screen.objectName}Card`
        anchors.fill: parent
        visible: card.open || card.leaving
        open: screen.tool.cardOpen
        eclipse: screen.eclipseMode
        pixelScale: screen.unit
        battery: screen.shownBattery
        clockText: screen.shownClock
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
        paneStatus: screen.tool.cardPaneStatus
        paneStatusOk: screen.tool.cardPaneStatusOk
        positionLine: screen.tool.cardPositionLine
        paneDrawn: screen.eclipseAnyActive && screen.tool.cardDetailValue.length > 0
        paneHeight: detailAside.implicitHeight
        help: HelpModel {
            screen: HelpModel.Card
        }
        onChosen: (index) => screen.tool.cardChoose(index)
        onStepped: (index, by, held) => screen.tool.cardStep(index, by)
        onFocused: (index) => screen.tool.cardFocus(index)
        onBacked: screen.tool.cardBack()
        onActed: (key) => screen.tool.cardBack()
        Column {
            id: detailAside
            visible: card.paneDrawn
            x: 36 * screen.boardUnit
            y: 36 * screen.boardUnit
            width: parent.width - 72 * screen.boardUnit
            spacing: 22 * screen.boardUnit
            CardLine {
                objectName: "detailPaneLabel"
                unit: screen.boardUnit
                boardLine: true
                boardLineHeight: 22
                width: parent.width
                text: screen.tool.cardDetailLabel
                color: "#F3C9D2"
                size: 18
                weight: Font.DemiBold
                spacing: 18 * 0.18
                capitalization: Font.AllUppercase
            }
            CardLine {
                objectName: "detailPaneValue"
                unit: screen.boardUnit
                boardLine: true
                boardLineHeight: 34.8
                width: parent.width
                text: screen.tool.cardDetailValue
                color: Kit.Tokens.textStrong
                size: 24
                weight: Font.Normal
                wrapMode: Text.Wrap
            }
        }
    }
    QuestionHost {
        id: question
        objectName: `${screen.objectName}Question`
        controller: screen.activeController
        title: screen.tool.questionTitle
        sentences: screen.tool.questionSentences
        caption: screen.tool.questionCaption
        choices: screen.tool.questionChoices
        backIndex: screen.tool.questionBack
        padFamily: screen.padFamily
        titleFont: Qt.font({ family: Kit.Tokens.faceLabel, pixelSize: Kit.Tokens.fontTitle, weight: Kit.Tokens.weightTitle })
        bodyFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
        captionFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontCaption })
        rowFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        help: HelpModel {
            screen: HelpModel.Question
        }
        showing: screen.tool.asking
        onChosen: (index) => screen.tool.answer(index)
        onOpenedChanged: {
            if (!question.opened && screen.eclipseAnyActive && !screen.tool.cardOpen && !screen.tool.asking)
                screen.activeView.forceActiveFocus()
        }
    }
    // A job's events pulled while it runs, never pushed (EmuSen_Qt.md §1.1).
    Timer {
        interval: 100
        repeat: true
        running: screen.visible && screen.tool.busy
        onTriggered: screen.tool.poll()
    }
}
