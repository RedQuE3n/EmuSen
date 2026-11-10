pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQml.Models
import EmuSen.Kit as Kit

// The settings frame on the kit's frame: the strip of sections or the trail of pages, rows from any model by kind with a value each, and the pane with one slot over the focused row's sentence; it knows no setting (EmuSen_Kit.md §7.40).
Kit.Frame {
    id: root
    // The rows: label, value, sentence, kind (Kit.RowKinds), held, marked, and a slider's minimum, maximum, step and amount.
    property alias model: list.model
    property list<string> sections
    // The pages open under the section, outermost first: the strip draws them, the last chosen, and the shoulders are the host's.
    property list<string> trail
    // The chosen section, which the shoulders and a click change.
    property alias section: layout.section
    // The tone of the focused row's sentence (Kit.Tones), which the host says by what the sentence says (EmuSen_Kit.md §7.52).
    property int sentenceTone: Kit.Tones.Text
    // The pane's one slot: a picture, a meter or the two variants, drawn over the sentence.
    property alias preview: previewSlot.data
    // Further lines of the pane, under the sentence.
    property alias lines: linesSlot.data
    readonly property alias layout: layout
    readonly property alias currentRow: layout.current
    signal sectionChosen(int section)
    signal stepped(int row, int by)
    signal chosen(int row)
    // A slider row moved by the pad or the pointer, to its new amount.
    signal slid(int row, real amount)
    controller.navigationShown: false
    Kit.SettingsLayout {
        id: layout
        objectName: "settingsLayout"
        // Read after the view has its rows, so the walk's focus, not the view's reset, is the one that stands.
        model: list.model
        width: root.width
        sections: root.sections.length
        trailShown: root.trail.length > 0
        onSectionChosen: (section) => root.sectionChosen(section)
        onStepped: (row, by) => root.stepped(row, by)
        onChosen: (row) => root.chosen(row)
        onSlid: (row, amount) => root.slid(row, amount)
        onWalked: {
            if (!list.moving)
                list.contentY = layout.first * Kit.Tokens.rowHeight
        }
    }
    // The host's rows changed: the walk reads them again, keeping its focus.
    Connections {
        target: list.model
        ignoreUnknownSignals: true
        function onDataChanged() { layout.read() }
        function onRowsInserted() { layout.read() }
        function onRowsRemoved() { layout.read() }
        function onModelReset() { layout.read() }
    }
    Row {
        objectName: "settingsStrip"
        x: layout.side - root.safeX
        y: layout.stripTop - root.safeY
        height: layout.stripHeight
        spacing: layout.stripGap
        Repeater {
            model: root.trail.length > 0 ? root.trail : root.sections
            delegate: TabButton {
                id: tab
                required property string modelData
                required property int index
                width: tab.implicitContentWidth + tab.leftPadding + tab.rightPadding
                height: layout.stripHeight
                text: tab.modelData.toUpperCase()
                font.pixelSize: Kit.Tokens.fontCaption
                horizontalPadding: layout.tabPadding
                focusPolicy: Qt.NoFocus
                checkable: false
                checked: root.trail.length > 0 ? tab.index === root.trail.length - 1 : tab.index === layout.section
                Accessible.name: tab.modelData
                onClicked: {
                    if (root.trail.length === 0 && tab.index !== layout.section) {
                        layout.section = tab.index
                        root.sectionChosen(tab.index)
                    }
                }
            }
        }
    }
    FocusScope {
        objectName: "settingsRows"
        x: layout.side - root.safeX
        y: layout.contentTop - root.safeY
        width: layout.rowsWidth
        height: Math.min(layout.count, layout.rowsShown) * Kit.Tokens.rowHeight
        focus: true
        Keys.onPressed: (event) => event.accepted = layout.key(event.key, event.modifiers)
        ListView {
            id: list
            objectName: "settingsList"
            anchors.fill: parent
            clip: true
            focus: true
            interactive: true
            boundsBehavior: Flickable.StopAtBounds
            keyNavigationEnabled: false
            highlightFollowsCurrentItem: false
            // Cleared, so the view never moves the focus to a row of its own choosing as its model changes; the walk's is the one.
            currentIndex: -1
            // Qt's item views set a current index of their own as rows arrive in an empty view, and focus its row; cleared again at once, so the walk's focus stands (EmuSen_Kit.md §7.51).
            onCurrentIndexChanged: {
                if (list.currentIndex !== -1)
                    list.currentIndex = -1
            }
            onContentYChanged: {
                if (!list.moving)
                    layout.scrollTo(Math.round(list.contentY / Kit.Tokens.rowHeight))
            }
            onMovementEnded: {
                layout.scrollTo(Math.round(list.contentY / Kit.Tokens.rowHeight))
                list.contentY = layout.first * Kit.Tokens.rowHeight
            }
            delegate: DelegateChooser {
                role: "kind"
                DelegateChoice {
                    roleValue: Kit.RowKinds.Slider
                    delegate: Kit.SliderRow {
                        id: slider
                        required property var model
                        required property int index
                        width: ListView.view.width
                        controller: root.controller
                        label: slider.model.label
                        value: slider.model.value
                        minimum: slider.model.minimum
                        maximum: slider.model.maximum
                        step: slider.model.step
                        amount: slider.model.amount
                        held: slider.model.held === true
                        focus: slider.index === layout.current
                        onActiveFocusChanged: {
                            if (slider.activeFocus)
                                layout.focusRow(slider.index)
                        }
                        onActivated: layout.press(Kit.Pad.Accept)
                        onSlid: (amount) => root.slid(slider.index, amount)
                    }
                }
                DelegateChoice {
                    roleValue: Kit.RowKinds.Choice
                    delegate: Kit.ChoiceRow {
                        id: choice
                        required property var model
                        required property int index
                        width: ListView.view.width
                        controller: root.controller
                        label: choice.model.label
                        value: choice.model.value
                        marked: choice.index === layout.marked
                        current: choice.index === layout.current
                        focus: choice.index === layout.current
                        onActiveFocusChanged: {
                            if (choice.activeFocus)
                                layout.focusRow(choice.index)
                        }
                        onActivated: layout.press(Kit.Pad.Accept)
                    }
                }
                DelegateChoice {
                    delegate: Kit.MenuRow {
                        id: row
                        required property var model
                        required property int index
                        width: ListView.view.width
                        controller: root.controller
                        label: row.model.label
                        value: row.model.value
                        steps: row.model.kind === Kit.RowKinds.Steps
                        opens: row.model.kind === Kit.RowKinds.Opens
                        current: row.index === layout.current
                        focus: row.index === layout.current
                        onActiveFocusChanged: {
                            if (row.activeFocus)
                                layout.focusRow(row.index)
                        }
                        onActivated: layout.press(Kit.Pad.Accept)
                    }
                }
            }
        }
    }
    Rectangle {
        objectName: "settingsPane"
        x: layout.paneLeft - root.safeX
        y: layout.contentTop - root.safeY
        width: layout.paneWidth
        height: layout.paneHeight
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
        Column {
            x: layout.paneInset
            y: layout.paneInset
            width: parent.width - 2 * layout.paneInset
            spacing: layout.lineGap
            Item {
                id: previewSlot
                objectName: "settingsSlot"
                width: parent.width
                height: previewSlot.childrenRect.height > 0 ? previewSlot.childrenRect.height + layout.slotGap - layout.lineGap : 0
                visible: previewSlot.height > 0
            }
            Kit.ToneText {
                objectName: "settingsSentence"
                width: parent.width
                text: layout.sentence
                wrapMode: Text.Wrap
                tone: root.sentenceTone
                font.pixelSize: Kit.Tokens.fontBody
                visible: text.length > 0
            }
            Column {
                id: linesSlot
                objectName: "settingsLines"
                width: parent.width
                spacing: layout.lineGap
            }
        }
    }
}
