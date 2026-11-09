pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A tool on the frame: the header row with its facts, the section strip with search, the columned list kept inside its width, the apply bar with its meter, and the pane spanning the list and the bar, all from the host's models; it knows no tool (EmuSen_Kit.md §7.43).
Kit.Frame {
    id: root
    property string title
    // Roles `words` and `tone`.
    property alias facts: factRepeater.model
    // The rows, any list model; each column's `key` names the role its cell reads.
    property alias rows: list.model
    // Roles `key`, `fill` (a weight, or 0 for the words' width), `least`, `trailing` (aligned right) and `toneKey` (the role holding the cell's tone).
    property var columns
    // The role a reader hears for a row.
    property string nameRole
    property alias currentIndex: list.currentIndex
    property alias searchText: search.text
    property string searchHint
    property string paneTitle
    property string paneCaption
    // Roles `words` and `tone`.
    property alias sentences: sentenceRepeater.model
    property string summary
    property int summaryTone: Kit.Tones.Text
    // Below zero no job runs; from 0 to 1, the running job's share done.
    property real progress: -1
    // Where a docked keyboard's band begins, in design units; below zero none is docked. While one is, the list keeps the whole rows above it and the bar and the pane are not drawn, so nothing lies under the band.
    property real keysTop: -1
    readonly property bool keysDocked: root.keysTop >= 0
    property string applyLabel
    property bool applyEnabled: false
    property real meter: 0
    property string meterWords
    property font cellFont: Qt.font({ pixelSize: Kit.Tokens.fontRow })
    // The screen's faces for its title, its facts and meter, and its pane's and bar's words; the platform's sans serif at the tokens' sizes by default.
    property font titleFont: Qt.font({ pixelSize: Kit.Tokens.fontTitle })
    property font captionFont: Qt.font({ pixelSize: Kit.Tokens.fontCaption })
    property font bodyFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    readonly property alias list: list
    signal activated(int index)
    signal applied()
    controller.navigationShown: false
    Kit.ToolLayout {
        id: places
        width: root.width
        height: root.height
    }
    Kit.ColumnFit {
        id: fit
        width: places.list.width
        rows: root.rows ?? null
        columns: root.columns
        font: root.cellFont
    }
    Connections {
        target: root.rows ?? null
        ignoreUnknownSignals: true
        function onDataChanged() { fit.measure() }
        function onModelReset() { fit.measure() }
        function onRowsInserted() { fit.measure() }
        function onRowsRemoved() { fit.measure() }
    }
    // The screen's own places over the whole frame, inside its focus scope, so the frame's walk and help bar serve them.
    Item {
        objectName: "toolScreen"
        parent: root
        width: root.width
        height: root.height
        Text {
            objectName: "toolTitle"
            x: places.header.x
            width: places.header.width - facts.width - Kit.Tokens.paneGap
            height: places.header.height
            verticalAlignment: Text.AlignVCenter
            text: root.title
            color: Kit.Tokens.text
            font: Qt.font({ family: root.titleFont.family, pixelSize: root.titleFont.pixelSize, capitalization: Font.AllUppercase })
            y: places.header.y
        }
        Row {
            id: facts
            objectName: "toolFacts"
            x: places.header.x + places.header.width - facts.width
            y: places.header.y
            height: places.header.height
            spacing: Kit.Tokens.base * 3
            Repeater {
                id: factRepeater
                delegate: Row {
                    id: fact
                    required property string words
                    required property int tone
                    required property int index
                    anchors.verticalCenter: parent?.verticalCenter
                    spacing: Kit.Tokens.base * 3
                    Rectangle {
                        visible: fact.index > 0
                        anchors.verticalCenter: parent.verticalCenter
                        width: Kit.Tokens.base
                        height: Kit.Tokens.base
                        radius: Kit.Tokens.base / 2
                        color: Kit.Tokens.muted
                    }
                    Kit.ToneText {
                        text: fact.words
                        tone: fact.tone
                        font: root.captionFont
                    }
                }
            }
        }
        Kit.SectionStrip {
            objectName: "toolSections"
            controller: root.controller
            x: places.strip.x
            y: places.strip.y
            width: places.strip.width - Kit.Tokens.paneGap
            height: places.strip.height
        }
        TextField {
            id: search
            objectName: "toolSearch"
            x: places.search.x
            y: places.search.y + (places.search.height - search.height) / 2
            width: places.search.width
            placeholderText: root.searchHint
            Accessible.name: root.searchHint
            KeyNavigation.down: list
        }
        Rectangle {
            objectName: "toolList"
            x: places.list.x
            y: places.list.y
            width: places.list.width
            height: root.keysDocked ? Math.min(places.list.height, Math.floor((root.keysTop - places.list.y) / Kit.Tokens.rowHeight) * Kit.Tokens.rowHeight) : places.list.height
            color: Kit.Tokens.inputSurface
            Kit.EdgeFade {
                anchors.fill: parent
                fadeTop: !list.atYBeginning
                fadeBottom: !list.atYEnd
                ListView {
                    id: list
                    objectName: "toolRows"
                    anchors.fill: parent
                    clip: true
                    focus: true
                    keyNavigationEnabled: true
                    boundsBehavior: Flickable.StopAtBounds
                    highlightMoveDuration: Kit.Tokens.instant
                    highlightResizeDuration: Kit.Tokens.instant
                    KeyNavigation.up: search
                    KeyNavigation.down: apply
                    KeyNavigation.right: apply
                    onCountChanged: fit.measure()
                    // The edges the audit reads as faded, as the EdgeFade around the list draws them.
                    property bool fadeTop: !list.atYBeginning
                    property bool fadeBottom: !list.atYEnd
                    delegate: Kit.Walkable {
                        id: row
                        required property var model
                        required property int index
                        width: ListView.view.width
                        height: Kit.Tokens.rowHeight
                        controller: root.controller
                        focus: ListView.isCurrentItem
                        Accessible.role: Accessible.ListItem
                        Accessible.name: row.model[root.nameRole] ?? ""
                        onActivated: root.activated(row.index)
                        onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                        Rectangle {
                            anchors.fill: parent
                            color: row.activeFocus ? Kit.Tokens.surfaceRaised : Kit.Tokens.inputSurface
                            border.width: row.hovered ? Kit.Tokens.hoverBorder : 0
                            border.color: Kit.Tokens.border
                            Rectangle {
                                width: Kit.Tokens.barWidth
                                height: parent.height
                                color: Kit.Tokens.accent
                                visible: row.activeFocus
                            }
                        }
                        Rectangle {
                            anchors.left: parent.left
                            anchors.right: parent.right
                            anchors.bottom: parent.bottom
                            height: Kit.Tokens.hoverBorder
                            color: Kit.Tokens.rule
                        }
                        Repeater {
                            model: root.columns
                            delegate: Kit.ToneText {
                                id: cell
                                required property string key
                                required property bool trailing
                                required property string toneKey
                                required property int index
                                x: fit.xs[cell.index] ?? 0
                                width: fit.widths[cell.index] ?? 0
                                anchors.verticalCenter: parent?.verticalCenter
                                text: row.model[cell.key] ?? ""
                                tone: cell.toneKey.length > 0 ? (row.model[cell.toneKey] ?? Kit.Tones.Text) : Kit.Tones.Text
                                font: root.cellFont
                                elide: Text.ElideRight
                                horizontalAlignment: cell.trailing ? Text.AlignRight : Text.AlignLeft
                            }
                        }
                    }
                }
            }
        }
        Item {
            objectName: "toolBar"
            visible: !root.keysDocked
            x: places.bar.x
            y: places.bar.y
            width: places.bar.width
            height: places.bar.height
            Kit.ToneText {
                objectName: "toolSummary"
                y: places.barInset
                width: parent.width
                text: root.summary
                tone: root.summaryTone
                font: root.bodyFont
                elide: Text.ElideRight
            }
            ProgressBar {
                objectName: "toolProgress"
                visible: root.progress >= 0
                y: places.barInset + Kit.Tokens.fontBody * 1.5 + Kit.Tokens.base
                width: parent.width
                value: Math.max(0, root.progress)
                Accessible.name: root.summary
            }
            Row {
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: Kit.Tokens.controlHeight
                spacing: Kit.Tokens.base * 3
                Kit.ActionButton {
                    id: apply
                    objectName: "toolApply"
                    controller: root.controller
                    label: root.applyLabel
                    button: Kit.Pad.Menu
                    family: root.padFamily
                    enabled: root.applyEnabled
                    KeyNavigation.up: list
                    KeyNavigation.left: list
                    onActivated: root.applied()
                }
                ProgressBar {
                    objectName: "toolMeter"
                    anchors.verticalCenter: parent.verticalCenter
                    width: Kit.Tokens.controlHeight * 4.5
                    value: root.meter
                    Accessible.name: root.meterWords
                }
                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: root.meterWords
                    color: Kit.Tokens.text
                    font: root.captionFont
                }
            }
        }
        Rectangle {
            objectName: "toolPane"
            visible: !root.keysDocked
            x: places.pane.x
            y: places.pane.y
            width: places.pane.width
            height: places.pane.height
            radius: Kit.Tokens.tileRadius
            color: Kit.Tokens.surfaceRaised
            Kit.EdgeFade {
                anchors.fill: parent
                anchors.margins: places.panePadding
                fadeTop: !pane.atYBeginning
                fadeBottom: !pane.atYEnd
                Flickable {
                    id: pane
                    objectName: "toolPaneWords"
                    anchors.fill: parent
                    clip: true
                    boundsBehavior: Flickable.StopAtBounds
                    contentWidth: width
                    contentHeight: paneColumn.implicitHeight
                    property bool fadeTop: !pane.atYBeginning
                    property bool fadeBottom: !pane.atYEnd
                    Column {
                        id: paneColumn
                        width: pane.width
                        spacing: Kit.Tokens.base * 2
                        Text {
                            width: parent.width
                            text: root.paneTitle
                            color: Kit.Tokens.textStrong
                            font: root.titleFont
                            wrapMode: Text.Wrap
                        }
                        Text {
                            visible: root.paneCaption.length > 0
                            width: parent.width
                            text: root.paneCaption
                            color: Kit.Tokens.muted
                            font: root.bodyFont
                            wrapMode: Text.Wrap
                        }
                        Repeater {
                            id: sentenceRepeater
                            delegate: Kit.ToneText {
                                id: sentence
                                required property string words
                                required tone
                                width: paneColumn.width
                                text: sentence.words
                                font: root.bodyFont
                                wrapMode: Text.Wrap
                            }
                        }
                    }
                }
            }
        }
    }
}
