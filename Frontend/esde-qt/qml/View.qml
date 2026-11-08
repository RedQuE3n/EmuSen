pragma ComponentBehavior: Bound
import QtQuick
import QtQml.Models
import EmuSen.Kit as Kit
import EmuSen.Esde

// The ES-DE view: the scene's frame bound element by element, a frame asked for only while something moves (EmuSen_Qt.md §4.8.4).
Item {
    id: root
    property alias settings: esde.settings
    property alias shelves: esde.shelves
    property alias media: esde.media
    property alias presence: esde.presence
    property alias swap: esde.swap
    property int family: Kit.Families.Generic
    readonly property alias scene: esde
    focus: true
    EsdeModel {
        id: esde
        objectName: "esdeModel"
        width: root.width
        height: root.height
    }
    ElementModel {
        id: elements
        scene: esde
        revision: esde.revision
    }
    FrameAnimation {
        running: esde.moving
        onTriggered: esde.tick(frameTime * 1000)
    }
    Timer {
        interval: Math.max(1, esde.nextChangeMs)
        running: !esde.moving && esde.nextChangeMs > 0
        repeat: true
        onTriggered: esde.tick(interval)
    }
    Connections {
        target: Kit.PadRouter
        enabled: root.activeFocus
        function onPressed(button, repeat, jump) { esde.pressed(button, repeat, jump) }
        function onReleased(button) { esde.released(button) }
    }
    Keys.onPressed: (event) => { event.accepted = Kit.PadRouter.pushKey(event.key, event.modifiers) }
    WheelHandler {
        acceptedDevices: PointerDevice.Mouse | PointerDevice.TouchPad
        onWheel: (event) => esde.wheel(Math.round(event.angleDelta.y / 120))
    }
    TapHandler {
        acceptedButtons: Qt.RightButton | Qt.BackButton
        onTapped: esde.pressed(Kit.Pad.Back, false, 0)
    }
    Repeater {
        model: elements
        delegate: DelegateChooser {
            role: "paint"
            DelegateChoice {
                roleValue: ElementModel.Picture
                PictureElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Text
                TextElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Rating
                RatingElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Badges
                BadgesElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Help
                HelpElement {
                    esde: esde
                    family: root.family
                }
            }
            DelegateChoice {
                roleValue: ElementModel.Info
                InfoElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Status
                StatusElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Carousel
                CarouselElement { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.TextList
                TextListView { esde: esde }
            }
            DelegateChoice {
                roleValue: ElementModel.Grid
                GridElement { esde: esde }
            }
        }
    }
    Item {
        objectName: "scrollLetters"
        anchors.centerIn: parent
        width: letters.contentWidth + esde.lettersPx
        height: esde.lettersPx * 1.5
        visible: esde.letters.length > 0 || esde.lettersMark.length > 0
        opacity: esde.lettersOpacity
        Rectangle {
            anchors.fill: parent
            radius: height / 4
            color: Qt.rgba(0, 0, 0, 0.6)
        }
        Mark {
            anchors.centerIn: parent
            width: esde.lettersPx
            height: esde.lettersPx
            mark: esde.lettersMark
            color: Qt.rgba(1, 1, 1, 1)
        }
        TextBlock {
            id: letters
            anchors.fill: parent
            visible: esde.lettersMark.length === 0
            text: esde.letters
            px: esde.lettersPx
            color: Qt.rgba(1, 1, 1, 1)
            hAlign: TextBlock.Align.Center
        }
    }
}
