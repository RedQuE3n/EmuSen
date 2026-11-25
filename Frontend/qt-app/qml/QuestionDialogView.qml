pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Circular 780x780 CoronaDisc question dialog content (board #14, EclipseDialog.dc.html, R-20261115-2).
FocusScope {
    id: root
    required property Kit.FrameController controller
    property string title
    property string message
    property list<string> sentences
    property string caption
    property string picture
    property list<string> choices
    property int backIndex: -1
    property int padFamily: Kit.Families.Xbox
    property real pixelScale: 1
    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    readonly property bool showPreview: root.picture.length > 0
    readonly property alias choiceList: choiceList
    signal chosen(int index)
    Rectangle {
        objectName: "dialogScrim"
        anchors.fill: parent
        color: Qt.rgba(4 / 255, 5 / 255, 10 / 255, 0.72)
    }
    Kit.CoronaDisc {
        id: disc
        objectName: "dialogDisc"
        x: (root.width - width) / 2
        y: 300 * root.unit
        width: 780 * root.unit
        height: 780 * root.unit
        fill: Kit.Tokens.surface
        pixelScale: root.pixelScale
    }
    Column {
        id: section
        objectName: "dialogSection"
        x: (root.width - width) / 2
        y: root.showPreview ? 430 * root.unit : Math.max(360 * root.unit, 300 * root.unit + (780 * root.unit - height) / 2)
        width: 600 * root.unit
        spacing: 22 * root.unit
        Item {
            id: preview
            objectName: "dialogPreview"
            visible: root.showPreview
            x: (section.width - width) / 2
            width: 220 * root.unit
            height: 138 * root.unit
            Rectangle {
                objectName: "dialogPreviewRim"
                x: -2 * root.unit
                y: -2 * root.unit
                width: preview.width + 4 * root.unit
                height: preview.height + 4 * root.unit
                radius: 18 * root.unit
                color: Qt.rgba(246 / 255, 231 / 255, 216 / 255, 0.5)
            }
            Rectangle {
                objectName: "dialogPreviewBox"
                anchors.fill: parent
                radius: 16 * root.unit
                color: "#1A2A6B"
                Kit.RoundPicture {
                    anchors.fill: parent
                    source: root.picture
                    radius: 16 * root.unit
                    crop: true
                    pixelScale: root.pixelScale
                }
            }
        }
        CardLine {
            id: titleLine
            objectName: "questionTitle"
            unit: root.unit
            boardLine: true
            width: section.width
            text: root.title
            color: Kit.Tokens.textStrong
            size: 44
            weight: Font.DemiBold
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }
        Flickable {
            id: wordsArea
            objectName: "questionWords"
            visible: bodyCol.implicitHeight > 0
            width: section.width
            height: Math.min(bodyCol.implicitHeight, (root.showPreview ? 180 : 340) * root.unit)
            contentWidth: width
            contentHeight: bodyCol.implicitHeight
            clip: contentHeight > height
            interactive: contentHeight > height
            boundsBehavior: Flickable.StopAtBounds
            Column {
                id: bodyCol
                width: wordsArea.width
                spacing: 14 * root.unit
                CardLine {
                    objectName: "questionMessage"
                    visible: root.message.length > 0
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 30.45
                    width: bodyCol.width
                    text: root.message
                    color: "#C9CDDB"
                    size: 21
                    weight: Font.Normal
                    wrapMode: Text.WordWrap
                    lineHeight: 30.45
                    lineHeightMode: Text.FixedHeight
                    horizontalAlignment: Text.AlignHCenter
                }
                Repeater {
                    model: root.sentences
                    CardLine {
                        id: sentenceLine
                        required property int index
                        required property string modelData
                        objectName: "questionSentence." + sentenceLine.index
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 30.45
                        width: bodyCol.width
                        text: sentenceLine.modelData
                        color: "#C9CDDB"
                        size: 21
                        weight: Font.Normal
                        wrapMode: Text.WordWrap
                        lineHeight: 30.45
                        lineHeightMode: Text.FixedHeight
                        horizontalAlignment: Text.AlignHCenter
                    }
                }
                CardLine {
                    objectName: "questionCaption"
                    visible: root.caption.length > 0
                    unit: root.unit
                    boardLine: true
                    width: bodyCol.width
                    text: root.caption
                    color: Kit.Tokens.muted
                    size: 16
                    weight: Font.Normal
                    wrapMode: Text.WordWrap
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
        Item {
            id: buttonsWrap
            objectName: "dialogButtonsWrap"
            width: section.width
            height: choiceList.height + 10 * root.unit
            ListView {
                id: choiceList
                objectName: "choices"
                readonly property real buttonWidth: (count > 2 ? Math.floor((600 - (count - 1) * 16) / count) : 230) * root.unit
                orientation: ListView.Horizontal
                x: (buttonsWrap.width - width) / 2
                y: 10 * root.unit
                width: count * buttonWidth + Math.max(0, count - 1) * spacing
                height: 72 * root.unit
                spacing: 16 * root.unit
                model: root.choices
                focus: true
                keyNavigationEnabled: true
                keyNavigationWraps: false
                interactive: false
                highlightMoveDuration: Kit.Tokens.instant
                highlightResizeDuration: Kit.Tokens.instant
                delegate: Kit.Walkable {
                    id: answer
                    required property string modelData
                    required property int index
                    readonly property bool current: ListView.isCurrentItem
                    objectName: "answer." + answer.index
                    controller: root.controller
                    width: choiceList.buttonWidth
                    height: 72 * root.unit
                    focus: answer.current
                    Accessible.role: Accessible.Button
                    Accessible.name: answer.modelData
                    onActiveFocusChanged: choiceList.currentIndex = answer.activeFocus ? answer.index : choiceList.currentIndex
                    onActivated: root.chosen(answer.index)
                    Keys.onUpPressed: (event) => {
                        if (wordsArea.interactive) {
                            wordsArea.contentY = Math.max(0, wordsArea.contentY - 48 * root.unit)
                        } else {
                            choiceList.decrementCurrentIndex()
                        }
                        event.accepted = true
                    }
                    Keys.onDownPressed: (event) => {
                        if (wordsArea.interactive) {
                            wordsArea.contentY = Math.min(wordsArea.contentHeight - wordsArea.height, wordsArea.contentY + 48 * root.unit)
                        } else {
                            choiceList.incrementCurrentIndex()
                        }
                        event.accepted = true
                    }
                    EclipsePillHighlight {
                        visible: answer.current
                        anchors.fill: parent
                        unit: root.unit
                        pixelScale: root.pixelScale
                        radius: 36 * root.unit
                        rowIndex: answer.index
                    }
                    Rectangle {
                        objectName: "answerBg." + answer.index
                        anchors.fill: parent
                        radius: 36 * root.unit
                        color: answer.current ? Kit.Tokens.accent : "transparent"
                        border.width: answer.current ? 0 : 2 * root.unit
                        border.color: "#5A6076"
                    }
                    CardLine {
                        objectName: "answerLabel." + answer.index
                        unit: root.unit
                        boardLine: true
                        x: (answer.width - width) / 2
                        y: (answer.height - height) / 2
                        text: answer.modelData
                        color: answer.current ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                        size: 25
                        weight: answer.current ? Font.DemiBold : Font.Normal
                    }
                }
            }
        }
    }
    EclipseFooter {
        id: helpBar
        objectName: "dialogHelp"
        unit: root.unit
        x: (root.width - width) / 2
        y: root.height - 40 * root.unit - height
        controller: root.controller
        family: root.padFamily
        model: HelpModel {
            screen: HelpModel.QuestionBack
            backLabel: Words.questionBackLabel(root.choices, root.backIndex)
        }
    }
}
