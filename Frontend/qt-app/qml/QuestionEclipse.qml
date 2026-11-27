pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// Modal Sheet wrapper for Eclipse's circular Question dialog (board #14, EclipseDialog.dc.html).
Kit.Sheet {
    id: root
    property string title
    property string message
    property list<string> sentences
    property string caption
    property string picture
    property list<string> choices
    property int backIndex: -1
    property int padFamily: Kit.Families.Xbox
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
    onAboutToShow: view.choiceList.currentIndex = 0
    onOpened: view.choiceList.forceActiveFocus()
    onChosen: root.close()
    Connections {
        target: root.controller
        function onSheetDismissed() { root.chosen(root.backIndex) }
    }
    Kit.NormalizedCanvas {
        id: canvas
        active: root.controller.form !== Kit.Forms.Desktop
        QuestionDialogView {
            id: view
            anchors.fill: parent
            focus: true
            controller: root.controller
            title: root.title
            message: root.message
            sentences: root.sentences
            caption: root.caption
            picture: root.picture
            choices: root.choices
            backIndex: root.backIndex
            padFamily: root.padFamily
            pixelScale: canvas.unit
            onChosen: (index) => root.chosen(index)
        }
    }
}
