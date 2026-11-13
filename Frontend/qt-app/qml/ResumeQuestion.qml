pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// The Resume question at a game's start, over the game held unplayed until it is answered (EmuSen_Interface.md §18.3).
Kit.NormalizedCanvas {
    id: resumeCanvas
    objectName: "resumeCanvas"
    required property SessionModel session
    required property int padFamily
    visible: resumeCanvas.session.resumeAsked
    QuestionModel {
        id: resumeWords
        subject: resumeCanvas.session.resumeSubject
    }
    Kit.Frame {
        id: resumeFrame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        focus: resumeCanvas.session.resumeAsked
        controller.navigationShown: false
        Kit.Question {
            id: resumeQuestion
            objectName: "resumeQuestion"
            controller: resumeFrame.controller
            title: resumeWords.title
            sentences: resumeWords.sentences
            caption: resumeWords.caption
            choices: resumeWords.choices
            backIndex: resumeWords.backIndex
            padFamily: resumeCanvas.padFamily
            titleFont: Qt.font({ family: resumeFaces.condensedRegular, pixelSize: Kit.Tokens.fontTitle })
            bodyFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: resumeFaces.regular, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: resumeFaces.regular, pixelSize: 20 })
            help: HelpModel {
                screen: HelpModel.QuestionBack
            }
            onChosen: (index) => resumeCanvas.session.answerResume(index)
        }
        // Opened each time it is asked; a Binding, since B closing the popup would undo a plain one.
        Binding {
            resumeQuestion.visible: resumeCanvas.session.resumeAsked
        }
    }
    Faces {
        id: resumeFaces
    }
}
