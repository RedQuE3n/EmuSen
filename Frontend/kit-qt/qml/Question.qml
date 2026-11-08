pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// The question form: a card centred over the screen under the cover's dim, the question, its sentences scrolling between it and the answers when long, a caption, the answers as rows with the first focused; A chooses, B gives backIndex (EmuSen_Interface.md §15.6, EmuSen_Kit.md §7.24).
Kit.Sheet {
    id: root
    property alias title: titleText.text
    property alias message: messageText.text
    property alias choices: choiceList.model
    // More paragraphs after the message, and a caption in Caption Muted under them (the setting that stops the asking, a hash).
    property alias sentences: paragraphs.model
    property alias caption: captionText.text
    // The answer B gives, as an index of the choices; -1 closes the question with no answer.
    property int backIndex: -1
    // The question's own help bar on the foot, from the host's model; none without one.
    property var help: null
    property alias padFamily: helpBar.family
    property alias helpFont: helpBar.labelFont
    property alias helpSpacing: helpBar.entrySpacing
    property alias helpGlyphGap: helpBar.glyphGap
    property font titleFont: Qt.font({ pixelSize: Kit.Tokens.fontTitle })
    property font bodyFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    property font captionFont: Qt.font({ pixelSize: Kit.Tokens.fontCaption })
    property font rowFont: Qt.font({ pixelSize: Kit.Tokens.fontRow })
    // The card's width in design units, the words' distance from its border, and the room it leaves the screen.
    property real cardWidth: 720
    property real cardPadding: Kit.Tokens.base * 8
    property real cardMargin: 100
    signal chosen(int index)
    anchors.centerIn: undefined
    parent: Overlay.overlay
    x: 0
    y: 0
    width: parent ? parent.width : 0
    height: parent ? parent.height : 0
    padding: 0
    background: null
    Overlay.modal: Item {}
    onAboutToShow: choiceList.currentIndex = 0
    onChosen: root.close()
    Connections {
        target: root.controller
        function onSheetDismissed() { root.chosen(root.backIndex) }
    }
    Kit.NormalizedCanvas {
        id: canvas
        active: root.controller.form !== Kit.Forms.Desktop
        Rectangle {
            id: cover
            objectName: "questionCover"
            width: canvas.width
            height: canvas.height - (helpBar.visible ? helpBar.height : 0)
            color: Kit.Tokens.overArtScrim
        }
        Rectangle {
            id: card
            objectName: "questionCard"
            readonly property real fixed: 2 * (root.cardPadding + border.width) + titleText.height + choiceList.height + 2 * Kit.Tokens.base * 3
            anchors.centerIn: cover
            width: root.cardWidth
            height: Math.min(card.fixed + words.contentHeight, cover.height - root.cardMargin)
            radius: Kit.Tokens.tileRadius
            color: Kit.Tokens.surfaceRaised
            border.width: Kit.Tokens.hoverBorder
            border.color: Kit.Tokens.border
            Text {
                id: titleText
                objectName: "questionTitle"
                x: root.cardPadding + card.border.width
                y: x
                width: card.width - 2 * x
                color: Kit.Tokens.text
                font: root.titleFont
                wrapMode: Text.Wrap
            }
            Flickable {
                id: words
                objectName: "questionWords"
                x: titleText.x
                y: titleText.y + titleText.height + Kit.Tokens.base * 3
                width: titleText.width
                height: choiceList.y - y - Kit.Tokens.base * 3
                contentHeight: paragraphColumn.height
                clip: true
                interactive: contentHeight > height
                boundsBehavior: Flickable.StopAtBounds
                Column {
                    id: paragraphColumn
                    width: words.width
                    spacing: Kit.Tokens.base * 2
                    Text {
                        id: messageText
                        objectName: "questionMessage"
                        width: paragraphColumn.width
                        visible: messageText.text.length > 0
                        color: Kit.Tokens.text
                        font: root.bodyFont
                        wrapMode: Text.Wrap
                    }
                    Repeater {
                        id: paragraphs
                        Text {
                            required property string modelData
                            width: paragraphColumn.width
                            text: modelData
                            color: Kit.Tokens.text
                            font: root.bodyFont
                            wrapMode: Text.Wrap
                        }
                    }
                    Text {
                        id: captionText
                        objectName: "questionCaption"
                        width: paragraphColumn.width
                        visible: captionText.text.length > 0
                        color: Kit.Tokens.muted
                        font: root.captionFont
                        wrapMode: Text.Wrap
                    }
                }
            }
            ListView {
                id: choiceList
                objectName: "choices"
                x: titleText.x
                y: card.height - root.cardPadding - card.border.width - height
                width: titleText.width
                height: count * Kit.Tokens.rowHeight + Math.max(0, count - 1) * spacing
                spacing: Kit.Tokens.base
                focus: true
                keyNavigationEnabled: true
                interactive: false
                highlightMoveDuration: Kit.Tokens.instant
                highlightResizeDuration: Kit.Tokens.instant
                delegate: Kit.MenuRow {
                    id: answer
                    required property string modelData
                    required property int index
                    objectName: "answer." + answer.index
                    controller: root.controller
                    width: choiceList.width
                    height: Kit.Tokens.rowHeight
                    label: answer.modelData
                    overRaised: true
                    inset: Kit.Tokens.base * 7
                    labelFont: root.rowFont
                    focus: ListView.isCurrentItem
                    onActiveFocusChanged: choiceList.currentIndex = answer.activeFocus ? answer.index : choiceList.currentIndex
                    onActivated: root.chosen(answer.index)
                }
            }
        }
        Kit.HelpBar {
            id: helpBar
            objectName: "questionHelp"
            controller: root.controller
            model: root.help
            visible: root.help !== null
            y: canvas.height - height
            width: canvas.width
            leftInset: Kit.Tokens.safeAreaX
        }
    }
}
