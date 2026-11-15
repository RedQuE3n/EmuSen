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
    // A card of Cover art: its words, its picture and whether it is in use, the model's roles.
    component CoverCardItem: Kit.Walkable {
        required property string label
        required property string line
        required property url picture
        required property bool marked
        required property int index
    }
    readonly property int contentFocused: screen.page.coverArt ? coverCards.currentIndex : frame.showsTiles ? frame.tileView.currentIndex : frame.rowView.currentIndex
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
        // Cover art stands in its place while it is open.
        visible: !screen.page.coverArt
        form: Kit.Forms.BigScreen
        // The viewer and the keyboard take the keys while they are open; the page has them back when they close.
        focus: screen.inputs && !screen.page.viewing && !screen.page.typing && !screen.page.coverArt
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
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
        // The viewer and the keyboard likewise, once closed.
        Connections {
            target: screen.page
            function onReturned() { frame.forceActiveFocus() }
            // B with nothing of the page's own open leaves the page.
            function onLeft() { screen.backed() }
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { screen.page.back() }
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
            titleFont: Qt.font({ family: Kit.Tokens.faceLabel, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
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
    // Details ▸ Cover art: the game's covers as cards four across, the one in use marked, the focused one large in the
    // pane beside, as the Eclipse board draws it (EmuSen_Interface.md §18.2; EmuSen_Qt.md §7.8.5).
    Kit.Frame {
        id: coverArt
        objectName: "pageCoverArt"
        anchors.fill: parent
        visible: screen.page.coverArt
        focus: screen.inputs && screen.page.coverArt
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.navigationShown: true
        controller.hasDetail: true
        controller.detailBeside: true
        // The board's four cards across, and its card's height per unit of its width (EclipseGameCover), constants of the design.
        readonly property int columns: 4
        readonly property real cardShape: 270 / 230
        readonly property real gap: Kit.Tokens.tileGap
        readonly property real cardWidth: (coverCards.width - 2 * coverArt.ringRoom - (coverArt.columns - 1) * coverArt.gap) / coverArt.columns
        readonly property real cardHeight: coverArt.cardWidth * coverArt.cardShape
        // Two lines of a card's words under it, clear of the focus ring's room.
        readonly property real ringRoom: Kit.Tokens.ringOffset + Kit.Tokens.ringThickness
        readonly property real wordsHeight: coverArt.ringRoom + 2 * Math.round(Kit.Tokens.fontCaption * 1.2)
        help: HelpModel {
            objectName: "coverArtHelp"
            screen: screen.page.help
        }
        Connections {
            target: coverArt.controller
            function onBackDeclined() { screen.page.back() }
            function onActionInvoked(key) { screen.page.declared(key) }
        }
        navigation: Column {
            width: parent.width
            Text {
                objectName: "coverArtTrail"
                width: parent.width
                text: screen.page.coverTrail
                color: Kit.Tokens.muted
                font.family: Kit.Tokens.faceText
                font.pixelSize: Kit.Tokens.fontCaption
                elide: Text.ElideRight
            }
            Text {
                objectName: "coverArtTitle"
                width: parent.width
                text: screen.page.coverTitle
                color: Kit.Tokens.textStrong
                font.family: Kit.Tokens.faceLabel
                font.pixelSize: Kit.Tokens.fontTitle
                elide: Text.ElideRight
            }
        }
        GridView {
            id: coverCards
            objectName: "coverCards"
            // The focus ring's room around the cards, inside the view's clip, as the page's tiles keep it.
            anchors.fill: parent
            anchors.margins: -coverArt.ringRoom
            leftMargin: coverArt.ringRoom
            topMargin: coverArt.ringRoom
            rightMargin: coverArt.ringRoom
            bottomMargin: coverArt.ringRoom
            model: screen.page.coverArt ? screen.page : null
            // A model that fills after the view was made leaves it with no current item, so no delegate would take the focus.
            onCountChanged: if (currentIndex < 0 && count > 0) currentIndex = 0
            focus: true
            clip: true
            keyNavigationEnabled: true
            keyNavigationWraps: false
            boundsBehavior: Flickable.StopAtBounds
            highlightMoveDuration: Kit.Tokens.instant
            cellWidth: coverArt.cardWidth + coverArt.gap
            cellHeight: coverArt.cardHeight + coverArt.wordsHeight + coverArt.gap
            delegate: CoverCardItem {
                id: card
                width: coverArt.cardWidth
                height: coverArt.cardHeight + coverArt.wordsHeight
                controller: coverArt.controller
                focus: GridView.isCurrentItem
                Accessible.role: Accessible.ListItem
                Accessible.name: card.label
                onActivated: screen.page.choose(card.index)
                onActiveFocusChanged: coverCards.currentIndex = card.activeFocus ? card.index : coverCards.currentIndex
                Kit.CoverTile {
                    id: cardCover
                    objectName: "coverCard"
                    width: coverArt.cardWidth
                    height: coverArt.cardHeight
                    source: card.picture
                    title: card.label
                    lit: card.activeFocus
                    lifts: false
                }
                Rectangle {
                    objectName: "coverInUse"
                    visible: card.marked
                    // On the picture's top left, where the cover stands at the box's foot.
                    x: cardCover.paintedX + Kit.Tokens.base * 2
                    y: cardCover.paintedY + Kit.Tokens.base * 2
                    width: inUse.implicitWidth + 2 * Kit.Tokens.base * 3
                    height: inUse.implicitHeight + 2 * Kit.Tokens.base
                    radius: height / 2
                    color: Kit.Tokens.accent
                    Text {
                        id: inUse
                        anchors.centerIn: parent
                        text: card.line
                        color: Kit.Tokens.overAccent
                        font.family: Kit.Tokens.faceText
                        font.weight: Font.DemiBold
                        font.pixelSize: Kit.Tokens.fontCaption
                    }
                }
                Text {
                    objectName: "coverCardWords"
                    y: coverArt.cardHeight + coverArt.ringRoom
                    width: coverArt.cardWidth
                    height: coverArt.wordsHeight - coverArt.ringRoom
                    text: card.label
                    lineHeightMode: Text.FixedHeight
                    lineHeight: Math.round(Kit.Tokens.fontCaption * 1.2)
                    color: card.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
                    font.family: Kit.Tokens.faceText
                    font.pixelSize: Kit.Tokens.fontCaption
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                }
            }
        }
        // The cover in focus, large, with its words.
        readonly property CoverCardItem focused: coverCards.currentItem as CoverCardItem
        detail: Kit.Card {
            objectName: "coverInFocus"
            width: parent.width
            Column {
                width: parent.width
                spacing: Kit.Tokens.paneGutter
                Kit.CoverTile {
                    objectName: "coverInFocusPicture"
                    width: parent.width
                    // The board's large cover, its height per unit of its width (EclipseGameCover), a constant of the design.
                    height: parent.width * 440 / 512
                    source: coverArt.focused?.picture ?? ""
                    title: coverArt.focused?.label ?? ""
                    lifts: false
                }
                Text {
                    objectName: "coverInFocusWords"
                    width: parent.width
                    text: coverArt.focused?.label ?? ""
                    color: Kit.Tokens.textStrong
                    font.family: Kit.Tokens.faceText
                    font.weight: Font.DemiBold
                    font.pixelSize: Kit.Tokens.fontStrong
                    wrapMode: Text.Wrap
                }
            }
        }
        // Where the focused card stands among them, at the help bar's end.
        Text {
            objectName: "coverArtPlace"
            parent: coverArt
            anchors.right: parent.right
            anchors.rightMargin: coverArt.safeX
            anchors.bottom: parent.bottom
            height: Kit.Tokens.helpBar
            verticalAlignment: Text.AlignVCenter
            text: coverCards.count > 0 ? screen.page.place(coverCards.currentIndex, coverCards.count) : ""
            color: Kit.Tokens.muted
            font.family: Kit.Tokens.faceText
            font.pixelSize: Kit.Tokens.fontCaption
        }
    }
    // The media viewer, from A on Media's tile: the picture, the clip with its sound, or the manual's page, on Void with
    // the caption bar over the help bar; ES-DE's viewer's buttons step it (EmuSen_Interface.md §18.2; EmuSen_Qt.md §7.8.2).
    Kit.Frame {
        id: viewer
        objectName: "pageViewer"
        anchors.fill: parent
        visible: screen.page.viewing
        focus: screen.inputs && screen.page.viewing
        surfaceDrawn: false
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.navigationShown: false
        help: HelpModel {
            objectName: "viewerHelp"
            screen: screen.page.viewHelp
        }
        Connections {
            target: viewer.controller
            function onBackDeclined() { screen.page.viewClose() }
            // X, Set as cover, where the help bar offers it for the picture shown.
            function onActionInvoked(key) { screen.page.declared(key) }
        }
        Item {
            id: shown
            objectName: "viewerShown"
            x: -viewer.safeX
            y: -viewer.safeY
            width: screen.width
            height: screen.height
            focus: true
            Keys.onPressed: (event) => event.accepted = screen.page.viewKey(event.key, event.modifiers)
            // The caption bar is 80 tall (EmuSen_Interface.md §18.2).
            readonly property real captionHeight: 80
            readonly property real areaHeight: screen.height - Kit.Tokens.helpBar - shown.captionHeight
            Rectangle {
                anchors.fill: parent
                color: Kit.Tokens.voidFill
            }
            Image {
                objectName: "viewerPicture"
                width: parent.width
                height: shown.areaHeight
                visible: screen.page.viewKind === 0
                source: screen.page.viewing && screen.page.viewKind === 0 ? screen.page.viewSource : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Kit.VideoTile {
                objectName: "viewerClip"
                width: parent.width
                height: shown.areaHeight
                visible: screen.page.viewKind === 1
                source: screen.page.viewKind === 1 ? screen.page.viewSource : ""
                playing: viewer.visible && screen.page.viewKind === 1
                // Heard at the player's Volume, as the page's clip is (EmuSen_Interface.md §5.9; R-20261111-100).
                muted: false
                volume: screen.page.clipVolume
                fillMode: Image.PreserveAspectFit
                delay: 0
            }
            // The manual's page fitted to the area, each zoom step half the area's height more, panned an eighth of its
            // width a step; made only while the manual is shown, so Qt PDF opens no file else.
            Loader {
                id: manualLoader
                width: parent.width
                height: shown.areaHeight
                active: screen.page.viewing && screen.page.viewKind === 2
                sourceComponent: Kit.PdfPage {
                    id: manual
                    objectName: "viewerManual"
                    readonly property real fit: pageSize.width > 0 && pageSize.height > 0 ? Math.min(manualLoader.width / pageSize.width, manualLoader.height / pageSize.height) : 0
                    readonly property real zoom: 1 + screen.page.viewZoom * 0.5
                    source: screen.page.viewManual
                    page: screen.page.viewPage
                    width: pageSize.width * fit * zoom
                    height: pageSize.height * fit * zoom
                    x: (manualLoader.width - width) / 2 - screen.page.viewPanX * manualLoader.width / 8
                    y: (manualLoader.height - height) / 2 - screen.page.viewPanY * manualLoader.width / 8
                    onPagesChanged: screen.page.viewPages(manual.pages)
                }
            }
            Rectangle {
                objectName: "viewerCaptionBar"
                y: shown.areaHeight
                width: parent.width
                height: shown.captionHeight
                color: Kit.Tokens.captionSurface
                Text {
                    objectName: "viewerCaption"
                    x: Kit.Tokens.safeAreaX
                    anchors.verticalCenter: parent.verticalCenter
                    text: screen.page.viewCaption
                    color: Kit.Tokens.textStrong
                    font.family: Kit.Tokens.faceText
                    font.pixelSize: Kit.Tokens.fontBody
                }
            }
        }
    }
    // A detail typed on the docked keyboard for Edit Details: Done saves it at once, B on an empty field lets it go.
    Kit.KeyboardModel {
        id: typist
        onClosed: (kept) => screen.page.typed(kept, typist.text)
    }
    Connections {
        target: screen.page
        function onTypingStartChanged() { typist.setText(screen.page.typingStart, screen.page.typingStart.length) }
    }
    Kit.Frame {
        id: typeFrame
        objectName: "pageTyping"
        anchors.fill: parent
        visible: screen.page.typing
        focus: screen.inputs && screen.page.typing
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.ToolSearch
        }
        Connections {
            target: typeFrame.controller
            function onActionInvoked(key) { typist.act(key) }
        }
        Column {
            spacing: Kit.Tokens.paneGap
            Text {
                text: screen.page.typingTitle
                color: Kit.Tokens.textStrong
                font.family: Kit.Tokens.faceText
                font.pixelSize: Kit.Tokens.fontTitle
            }
            Text {
                objectName: "pageTyped"
                text: typist.text
                color: Kit.Tokens.text
                font.family: Kit.Tokens.faceText
                font.pixelSize: Kit.Tokens.fontRow
            }
        }
        Item {
            x: -typeFrame.safeX
            y: -typeFrame.safeY
            width: screen.width
            height: screen.height
            Kit.Keyboard {
                objectName: "pageKeyboard"
                model: typist
                y: 398
                width: screen.width
                focus: true
                labelFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
            }
        }
    }
}
