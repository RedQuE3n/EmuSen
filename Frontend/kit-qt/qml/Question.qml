pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import EmuSen.Kit

// A question: a title, a message and its choices as buttons in a row the pad walks with Left and Right; A chooses, B dismisses (EmuSen_Kit.md §7.8).
Sheet {
    id: root
    property alias title: titleText.text
    property alias message: messageText.text
    property alias choices: choiceList.model
    signal chosen(int index)
    ColumnLayout {
        width: root.availableWidth
        spacing: Tokens.base * 3
        Label {
            id: titleText
            objectName: "questionTitle"
            Layout.fillWidth: true
            font.pixelSize: Tokens.fontTitle
            color: Tokens.textStrong
            wrapMode: Text.Wrap
        }
        Label {
            id: messageText
            objectName: "questionMessage"
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            visible: messageText.text.length > 0
        }
        ListView {
            id: choiceList
            objectName: "choices"
            Layout.alignment: Qt.AlignRight
            Layout.preferredHeight: Tokens.controlHeight
            Layout.fillWidth: true
            orientation: ListView.Horizontal
            layoutDirection: Qt.RightToLeft
            spacing: Tokens.base * 2
            focus: true
            keyNavigationEnabled: true
            highlightMoveDuration: Tokens.instant
            interactive: false
            delegate: Button {
                id: choice
                required property string modelData
                required property int index
                text: choice.modelData
                highlighted: choice.index === 0
                focus: ListView.isCurrentItem
                onClicked: root.chosen(choice.index)
            }
        }
    }
}
