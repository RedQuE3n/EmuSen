pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The questions before a game opens: its core's approval, then the firmware it needs (EmuSen_Qt.md §7.7).
Kit.NormalizedCanvas {
    id: launchCanvas
    objectName: "launchCanvas"
    required property LaunchModel launch
    required property int padFamily
    visible: launchCanvas.launch.asking
    Kit.Frame {
        id: launchFrame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        focus: launchCanvas.launch.asking
        controller.navigationShown: false
        Kit.Question {
            id: launchQuestion
            objectName: "launchQuestion"
            controller: launchFrame.controller
            title: launchCanvas.launch.title
            sentences: launchCanvas.launch.sentences
            caption: launchCanvas.launch.caption
            choices: launchCanvas.launch.choices
            backIndex: launchCanvas.launch.backIndex
            padFamily: launchCanvas.padFamily
            titleFont: Qt.font({ family: launchFaces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: launchFaces.regular, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: launchFaces.regular, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: launchFaces.regular, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: launchFaces.regular, pixelSize: 20 })
            help: HelpModel {
                screen: launchCanvas.launch.help
            }
            onChosen: (index) => launchCanvas.launch.answer(index)
        }
        Binding {
            launchQuestion.visible: launchCanvas.launch.asking
        }
    }
    Faces {
        id: launchFaces
    }
}
