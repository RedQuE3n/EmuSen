pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme 1.0

// A fixture of theme-audit: a column of walkable rows over one of the library's lists, in canvas units, walked by Up and Down,
// scrolling under faded edges when the list is longer than the screen.
NormalizedCanvas {
    id: canvas
    required property var rows
    required property string role
    Frame {
        id: frame
        anchors.fill: parent
        form: Tokens.form
        focus: true
        ListView {
            id: list
            objectName: "rows"
            x: 64
            y: 64
            width: 480
            height: Math.min(list.count * 64, parent.height - 64)
            spacing: 8
            clip: true
            focus: true
            keyNavigationEnabled: true
            highlightMoveDuration: Tokens.instant
            model: canvas.rows
            property bool fadeTop: !list.atYBeginning
            property bool fadeBottom: !list.atYEnd
            // The library is read after the screen is placed: the first row takes the focus when rows come.
            onCountChanged: list.currentIndex = list.count > 0 ? Math.max(list.currentIndex, 0) : -1
            delegate: Walkable {
                id: row
                required property int index
                required property var model
                objectName: "row." + row.index
                controller: frame.controller
                width: 480
                height: 56
                focus: ListView.isCurrentItem
                onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                Rectangle {
                    anchors.fill: parent
                    radius: Tokens.tileRadius
                    color: row.activeFocus ? Tokens.surfaceRaised : Tokens.surface
                }
                Image {
                    x: 432
                    width: 40
                    height: 56
                    source: row.model.cover
                    fillMode: Image.PreserveAspectCrop
                }
                Text {
                    x: 16
                    anchors.verticalCenter: parent.verticalCenter
                    text: row.model.cover !== "" ? row.model[canvas.role] : ""
                    color: Tokens.text
                    font.family: "Barlow"
                    font.pixelSize: 26
                }
            }
        }
        Rectangle {
            x: list.x
            y: list.y
            width: list.width
            height: 28
            visible: list.fadeTop
            enabled: false
            gradient: Gradient {
                GradientStop { position: 0; color: Tokens.surface }
                GradientStop { position: 1; color: "transparent" }
            }
        }
        Rectangle {
            x: list.x
            y: list.y + list.height - 28
            width: list.width
            height: 28
            visible: list.fadeBottom
            enabled: false
            gradient: Gradient {
                GradientStop { position: 0; color: "transparent" }
                GradientStop { position: 1; color: Tokens.surface }
            }
        }
    }
}
