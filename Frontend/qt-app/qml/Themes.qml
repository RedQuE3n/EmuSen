pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Models

// The Themes tab on the frame: the two kinds above, the themes of the kind listed, the ES-DE theme in use followed by its choices, the focused one's preview, line and check beside them; A uses a theme checked clean, X checks it, A, Left and Right step a choice (EmuSen_Qt.md §3.8).
Kit.NormalizedCanvas {
    id: tab
    required property ThemesModel themes
    property int padFamily
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on the tab, and its declared actions the screens answer (Start, Home).
    signal backed()
    signal acted(string key)
    Kit.Frame {
        id: frame
        objectName: "themesFrame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: tab.inputs
        padFamily: tab.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.sections: tab.themes.kinds
        controller.hasDetail: true
        controller.detailBeside: true
        help: HelpModel {
            screen: tab.themes.help
        }
        navigation: ListView {
            objectName: "themeKinds"
            anchors.fill: parent
            orientation: ListView.Horizontal
            spacing: Kit.Tokens.base * 2
            interactive: false
            model: tab.themes.kinds
            delegate: TabButton {
                id: kind
                required property string modelData
                text: kind.modelData
                font.family: faces.condensed
                font.pixelSize: Kit.Tokens.fontTitle
                focusPolicy: Qt.NoFocus
                checkable: false
                checked: kind.modelData === frame.controller.selectedSection
                onClicked: frame.controller.selectedSection = kind.modelData
            }
        }
        Kit.MenuPanel {
            id: rows
            objectName: "themeRows"
            anchors.fill: parent
            controller: frame.controller
            model: tab.themes
            // The model's focused row, set again after each read; the view's moves are written back.
            currentIndex: tab.themes.focused
            onActivated: tab.themes.accept()
            onCurrentIndexChanged: tab.themes.focused = rows.currentIndex
            // The panel's own row in the screens' face, as the cards' rows are drawn.
            delegate: Kit.MenuRow {
                id: row
                required property var model
                required property int index
                width: ListView.view.width
                controller: frame.controller
                label: row.model.label
                value: row.model.value
                steps: row.model.steps
                labelFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontRow })
                current: ListView.isCurrentItem
                focus: ListView.isCurrentItem
                onActivated: tab.themes.accept()
                // Left and Right step a choice and pass on elsewhere.
                Keys.onLeftPressed: (event) => {
                    event.accepted = row.steps
                    if (row.steps)
                        tab.themes.step(row.index, -1)
                }
                Keys.onRightPressed: (event) => {
                    event.accepted = row.steps
                    if (row.steps)
                        tab.themes.step(row.index, 1)
                }
                // The model holds the focused row and the view's current row is bound to it, so the binding is never overwritten.
                onActiveFocusChanged: tab.themes.focused = row.activeFocus ? row.index : tab.themes.focused
            }
        }
        detail: Column {
            objectName: "themeDetail"
            anchors.fill: parent
            spacing: Kit.Tokens.base * 2
            Rectangle {
                width: parent.width
                height: parent.width * 10 / 16
                radius: Kit.Tokens.tileRadius
                color: Kit.Tokens.surfaceRaised
                Kit.RoundPicture {
                    objectName: "themePreview"
                    anchors.fill: parent
                    visible: tab.themes.preview.length > 0
                    source: tab.themes.preview
                    radius: Kit.Tokens.tileRadius
                }
            }
            Label {
                objectName: "themeName"
                width: parent.width
                text: tab.themes.name
                elide: Text.ElideRight
                color: Kit.Tokens.textStrong
                font.family: faces.condensed
                font.pixelSize: Kit.Tokens.fontTitle
            }
            Label {
                objectName: "themeLine"
                width: parent.width
                text: tab.themes.line
                wrapMode: Text.Wrap
                color: Kit.Tokens.text
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontBody
            }
            Label {
                objectName: "themeFindings"
                width: parent.width
                text: tab.themes.findings
                wrapMode: Text.Wrap
                maximumLineCount: 8
                elide: Text.ElideRight
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontCaption
            }
        }
        Binding {
            target: tab.themes
            property: "kind"
            value: frame.controller.selectedSection
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { tab.themes.act(key) }
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { tab.acted(key) }
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { tab.backed() }
        }
    }
    Faces {
        id: faces
    }
}
