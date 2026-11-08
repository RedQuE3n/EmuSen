pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import EmuSen.Kit as Kit

// The kit's gallery: every control and drawn item in three sections the shoulders step, in the form the host sets and the variant the tokens paint; the gate runs on it at the four sizes (EmuSen_Kit.md §7.10).
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: Kit.Tokens.voidFill
    property int form: Kit.Forms.BigScreen
    property int activated: -1
    property int chosen: -1
    readonly property int page: frame.controller.sections.indexOf(frame.controller.selectedSection)
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "canvas"
        active: scene.form !== Kit.Forms.Desktop
        Kit.Frame {
            id: frame
            anchors.fill: parent
            form: scene.form
            focus: true
            opacity: scene.page === 2 ? 0 : 1
            controller.sections: ["Menu", "Controls", "Moonrise"]
            help: ListModel {
                ListElement { button: 4; label: "Choose" }
                ListElement { button: 5; label: "Back" }
                ListElement { button: 6; label: "Menu" }
            }
            navigation: ListView {
                objectName: "sections"
                anchors.fill: parent
                orientation: frame.desktop ? ListView.Vertical : ListView.Horizontal
                spacing: Kit.Tokens.base
                interactive: false
                model: frame.controller.sections
                delegate: TabButton {
                    id: section
                    required property string modelData
                    width: frame.desktop ? ListView.view.width : implicitWidth
                    text: section.modelData
                    focusPolicy: Qt.NoFocus
                    checkable: false
                    checked: section.modelData === frame.controller.selectedSection
                    onClicked: frame.controller.selectedSection = section.modelData
                }
            }
            StackLayout {
                anchors.fill: parent
                currentIndex: scene.page
                Item {
                    RowLayout {
                        anchors.fill: parent
                        spacing: Kit.Tokens.paneGap
                        Kit.MenuPanel {
                            id: panel
                            objectName: "panel"
                            controller: frame.controller
                            Layout.preferredWidth: Kit.Tokens.cardWidth
                            Layout.fillHeight: true
                            KeyNavigation.right: carousel
                            onActivated: (index) => scene.activated = index
                            model: ListModel {
                                ListElement { label: "Display"; value: "4:3" }
                                ListElement { label: "Shader"; value: "CRT" }
                                ListElement { label: "Sound"; value: "On" }
                                ListElement { label: "Pads"; value: "2" }
                                ListElement { label: "Saves"; value: "Auto" }
                                ListElement { label: "Rewind"; value: "Off" }
                                ListElement { label: "Speed"; value: "100%" }
                                ListElement { label: "Region"; value: "Auto" }
                            }
                        }
                        Column {
                            Layout.alignment: Qt.AlignTop
                            visible: scene.form !== Kit.Forms.Card
                            spacing: Kit.Tokens.paneGap
                            Kit.Sky {
                                id: sky
                                objectName: "sky"
                                readonly property real unit: sky.height / 120
                                width: carousel.width
                                height: Kit.Tokens.tileHeight / 2
                                glowHue: 212
                                Row {
                                    anchors.verticalCenter: parent.verticalCenter
                                    x: Kit.Tokens.tileGap
                                    spacing: Kit.Tokens.tileGap
                                    Kit.Body { objectName: "planetFocused"; anchors.verticalCenter: parent.verticalCenter; width: 84 * sky.unit; height: 84 * sky.unit; hue: 352; focused: true; craters: 0.3; bands: 0.5; cap: 0.9; ring: Kit.Rings.Big; moons: 2 }
                                    Kit.Body { objectName: "planet"; anchors.verticalCenter: parent.verticalCenter; width: 58 * sky.unit; height: 58 * sky.unit; hue: 212 }
                                    Kit.Body { objectName: "shards"; anchors.verticalCenter: parent.verticalCenter; width: 42 * sky.unit; height: 68 * sky.unit; shape: Kit.Bodies.Shards; hue: 220 }
                                    Kit.Body { objectName: "gem"; anchors.verticalCenter: parent.verticalCenter; width: 54 * sky.unit; height: 86 * sky.unit; shape: Kit.Bodies.Gem; hue: 230; focused: true }
                                }
                            }
                            Kit.Carousel {
                                id: carousel
                                objectName: "carousel"
                                controller: frame.controller
                                Accessible.name: "Worlds"
                                width: Kit.Tokens.tileWidth * 3 + Kit.Tokens.tileGap * 2
                                height: Kit.Tokens.tileHeight
                                KeyNavigation.up: panel
                                model: 8
                                delegate: Kit.Card {
                                    id: tile
                                    required property int index
                                    width: Kit.Tokens.tileWidth
                                    height: Kit.Tokens.tileHeight
                                    Kit.Body { id: world; anchors.horizontalCenter: parent.horizontalCenter; width: Kit.Tokens.tileWidth * 0.32; height: width; hue: tile.index * 45; focused: tile.PathView.isCurrentItem && carousel.activeFocus }
                                    Text { anchors.horizontalCenter: parent.horizontalCenter; y: world.height + Kit.Tokens.base * 3; text: "World"; color: Kit.Tokens.text; font.pixelSize: Kit.Tokens.fontCaption }
                                }
                            }
                        }
                    }
                }
                Column {
                    spacing: Kit.Tokens.base
                    Kit.WholeRow {
                        id: row0
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        Button { objectName: "play"; text: "Play"; focus: true; KeyNavigation.down: row1 }
                    }
                    Kit.WholeRow {
                        id: row1
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        Button { objectName: "primary"; text: "Primary"; highlighted: true; focus: true; KeyNavigation.up: row0; KeyNavigation.down: row2 }
                    }
                    Kit.WholeRow {
                        id: row2
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        CheckBox { objectName: "check"; text: "Keep saves"; checked: true; focus: true; KeyNavigation.up: row1; KeyNavigation.down: row3 }
                    }
                    Kit.WholeRow {
                        id: row3
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        RadioButton { objectName: "radio"; text: "Sharp"; checked: true; focus: true; KeyNavigation.up: row2; KeyNavigation.down: row4 }
                    }
                    Kit.WholeRow {
                        id: row4
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        Switch { objectName: "switch"; text: "Rewind"; checked: true; focus: true; KeyNavigation.up: row3; KeyNavigation.down: row5 }
                    }
                    Kit.WholeRow {
                        id: row5
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        Slider { objectName: "slider"; Accessible.name: "Volume"; from: 0; to: 100; value: 40; width: Kit.Tokens.controlHeight * 6; focus: true; KeyNavigation.up: row4; KeyNavigation.down: row6 }
                    }
                    Kit.WholeRow {
                        id: row6
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        TabBar {
                            objectName: "tabs"
                            focus: true
                            KeyNavigation.up: row5
                            KeyNavigation.down: row7
                            TabButton { text: "Games" }
                            TabButton { text: "Settings" }
                        }
                    }
                    Kit.WholeRow {
                        id: row7
                        width: Kit.Tokens.cardWidth
                        controller: frame.controller
                        Row {
                            spacing: Kit.Tokens.paneGap
                            ComboBox { id: combo; objectName: "combo"; model: ["Auto", "NTSC", "PAL"]; focus: true; KeyNavigation.priority: KeyNavigation.BeforeItem; KeyNavigation.up: row6; KeyNavigation.right: field }
                            TextField { id: field; objectName: "field"; placeholderText: "Search"; KeyNavigation.up: row6; KeyNavigation.left: combo }
                        }
                    }
                    ProgressBar { objectName: "progress"; value: 0.6; width: Kit.Tokens.cardWidth }
                    Label { objectName: "label"; text: "A label in the tokens" }
                }
                Item {}
            }
        }
        Kit.Question {
            id: question
            objectName: "question"
            controller: frame.controller
            title: "Leave the game?"
            message: "Unsaved progress is lost."
            choices: ["Stay", "Leave"]
            onChosen: (index) => scene.chosen = index
        }
        Shortcut {
            sequence: "F1"
            onActivated: question.open()
        }
    }
    Kit.Moonrise {
        objectName: "moonrise"
        anchors.fill: parent
        visible: scene.page === 2
        title: "EMUSEN"
        caption: "11 CONSOLES"
        hues: [330, 38, 352, 212, 190, 220, 0, 190, 230, 250, 25]
    }
}
