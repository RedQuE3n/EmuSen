pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Kit as Kit
import EmuSen.Models

// The Pause Card over the paused game, in canvas units as Mistress draws it: the scrim, the panel at the left edge with its shadow, the title row, the page's rows, the help band, sliding in and out (EmuSen_Interface.md §2.8, §5 item 4, §20.6; EmuSen_Qt.md §7.1 step 7).
FocusScope {
    id: root
    objectName: "PadCard"
    required property LibraryModel library
    required property SessionModel session
    property int padFamily: Kit.Families.Generic
    // Whether the card is on screen; it slides in as this turns on and out as it turns off.
    property bool open: false
    readonly property alias model: pause
    // The panel is 40% of the width, as Mistress's card is, and the band under it 98 units high.
    readonly property real panelWidth: Math.round(root.width * 0.4)
    readonly property real bandHeight: 98
    // 1 out of place, 0 in place: the panel moves left by its right edge and 40 units, the band down by its height, the pane fades (LunaP §218).
    property real slide: 1
    readonly property bool leaving: !root.open && root.slide < 1
    readonly property int coverRow: games.revision >= 0 ? games.rowOf(root.session.game) : -1
    signal resumed()
    signal backToLibrary()
    // The card has left the screen: the game it paused may resume.
    signal gone()
    Accessible.role: Accessible.Pane
    // Opened on its first page, sliding in; closed, sliding out. Page turns move nothing (EmuSen_Interface.md §18.5).
    onOpenChanged: {
        if (root.open) {
            pause.open()
            enter.restart()
        } else {
            exit.restart()
        }
    }
    NumberAnimation {
        id: enter
        target: root
        property: "slide"
        to: 0
        duration: Kit.Tokens.enter
        easing.type: Kit.Tokens.enterCurve
    }
    NumberAnimation {
        id: exit
        target: root
        property: "slide"
        to: 1
        duration: Kit.Tokens.exit
        easing.type: Kit.Tokens.exitCurve
        onFinished: root.gone()
    }
    Faces {
        id: faces
    }
    PauseModel {
        id: pause
        objectName: "pause"
        game: root.library.titleOf(root.session.game)
        facts: root.session.cardFacts
        onResumed: root.resumed()
        onLibrary: root.backToLibrary()
        onCurrentChanged: rows.currentIndex = pause.current
        onSlotChosen: slot => root.session.chooseSlot(slot)
        onSaveAsked: slot => root.session.saveState(slot)
        onLoadAsked: slot => root.session.loadState(slot)
        onRewindAsked: frame => root.session.rewindTo(frame)
    }
    GameModel {
        id: games
        games: root.library.recent
    }
    Rectangle {
        objectName: "AdaptiveCardScrim"
        anchors.fill: parent
        color: Kit.Tokens.scrim
    }
    CardPane {
        x: 576
        y: 156
        pane: pause.pane
        opacity: 1 - root.slide
    }
    Rectangle {
        id: band
        objectName: "pauseBand"
        x: 0
        y: root.height - root.bandHeight + root.bandHeight * root.slide
        width: root.width
        height: root.bandHeight
        color: Kit.Tokens.surface
        Rectangle {
            width: parent.width
            height: Kit.Tokens.hoverBorder
            color: Kit.Tokens.border
        }
    }
    Kit.Frame {
        id: frame
        x: -(root.panelWidth + 40) * root.slide
        width: root.width
        height: root.height
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        help: HelpModel {
            screen: pause.helpScreen
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { pause.back() }
            function onActionInvoked(key) { pause.act(key) }
        }
        Item {
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            Rectangle {
                id: caster
                width: root.panelWidth
                height: root.height
                color: Kit.Tokens.surface
                visible: false
            }
            // One shadow on the card's open edge: offset 0,8, blur 24, black at 50% (EmuSen_Interface.md §5.4).
            MultiEffect {
                source: caster
                anchors.fill: caster
                shadowEnabled: true
                shadowColor: "#000000"
                shadowOpacity: 0.5
                shadowVerticalOffset: 8
                blurMax: 24
                shadowBlur: 1
            }
            Rectangle {
                objectName: "AdaptiveCard"
                width: root.panelWidth
                height: root.height
                color: Kit.Tokens.surface
                Accessible.role: Accessible.PopupMenu
            }
            Image {
                id: cover
                objectName: "pauseCover"
                x: 64
                y: 50
                width: 60
                height: 80
                visible: root.coverRow >= 0 && cover.source.toString().length > 0
                source: root.coverRow >= 0 ? games.coverAt(root.coverRow) : ""
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            Column {
                x: cover.visible ? cover.x + cover.width + 16 : 64
                width: root.panelWidth - x - 24
                anchors.verticalCenter: cover.verticalCenter
                Text {
                    objectName: "pauseBreadcrumb"
                    width: parent.width
                    visible: text.length > 0
                    text: pause.breadcrumb
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: 20
                    elide: Text.ElideRight
                }
                Text {
                    objectName: "pauseTitle"
                    width: parent.width
                    text: pause.title
                    color: Kit.Tokens.textStrong
                    font.family: faces.condensedRegular
                    font.weight: Font.Normal
                    font.pixelSize: 32
                    fontSizeMode: Text.HorizontalFit
                    minimumPixelSize: 22
                    elide: Text.ElideRight
                }
            }
            Item {
                objectName: "AdaptiveCardRows"
                x: 64
                y: 156
                width: root.panelWidth - 88
                height: rows.count * Kit.Tokens.rowHeight
                ListView {
                    id: rows
                    objectName: "PadMenuList"
                    anchors.fill: parent
                    model: pause
                    focus: true
                    interactive: false
                    keyNavigationEnabled: true
                    keyNavigationWraps: false
                    highlightMoveDuration: Kit.Tokens.instant
                    highlightResizeDuration: Kit.Tokens.instant
                    onCurrentIndexChanged: pause.focusRow(rows.currentIndex)
                    Accessible.role: Accessible.List
                    delegate: Kit.MenuRow {
                        id: row
                        required property int index
                        required property var model
                        width: ListView.view.width
                        controller: frame.controller
                        label: row.model.label
                        value: row.model.value
                        steps: row.model.steps
                        opens: row.model.opens
                        ruled: true
                        sectionEnd: row.model.sectionEnd
                        inset: 24
                        labelFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
                        current: ListView.isCurrentItem
                        focus: ListView.isCurrentItem
                        onActivated: pause.choose(row.index)
                        Keys.onLeftPressed: event => pause.step(row.index, -1, event.isAutoRepeat)
                        Keys.onRightPressed: event => pause.step(row.index, 1, event.isAutoRepeat)
                        onActiveFocusChanged: rows.currentIndex = row.activeFocus ? row.index : rows.currentIndex
                    }
                }
            }
        }
    }
    Kit.HelpBar {
        objectName: "pauseHelp"
        y: band.y + (root.bandHeight - Kit.Tokens.helpBar) / 2
        width: root.width
        controller: frame.controller
        model: frame.help
        family: root.padFamily
        color: Kit.Tokens.surface
        ruled: false
        leftInset: 64
        labelFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        entrySpacing: 40
        glyphGap: 10
    }
}
