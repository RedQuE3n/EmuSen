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
    property bool bigScreen: true
    property LibraryModel library: null
    property Clock clock: null
    property int battery: -1
    readonly property bool eclipseMode: Kit.Tokens.coronaRim > 0
    readonly property bool sheetOpen: resumeCanvas.eclipseMode && resumeFrame.controller.sheetOpen
    readonly property string subjectTitle: resumeCanvas.library && resumeCanvas.session.game.length > 0 && resumeCanvas.library.titleOf(resumeCanvas.session.game).length > 0 ? resumeCanvas.library.titleOf(resumeCanvas.session.game) : resumeCanvas.session.resumeSubject
    readonly property string coverUrl: (resumeCanvas.library && resumeCanvas.library.places >= 0 && resumeCanvas.library.recent.length >= 0 && !resumeCanvas.library.loading && resumeCanvas.session.game.length > 0) ? resumeCanvas.library.coverOf(resumeCanvas.session.game) : ""
    visible: resumeCanvas.session.resumeAsked || resumeCanvas.sheetOpen
    QuestionModel {
        id: resumeWords
        subject: resumeCanvas.subjectTitle
    }
    Kit.Frame {
        id: resumeFrame
        anchors.fill: parent
        form: resumeCanvas.bigScreen ? Kit.Forms.BigScreen : Kit.Forms.Desktop
        surfaceDrawn: false
        helpShown: false
        focus: resumeCanvas.session.resumeAsked || resumeCanvas.sheetOpen
        controller.navigationShown: false
        QuestionHost {
            id: resumeQuestion
            objectName: "resumeQuestion"
            controller: resumeFrame.controller
            title: resumeWords.title
            sentences: resumeWords.sentences
            caption: resumeWords.caption
            choices: resumeWords.choices
            backIndex: resumeWords.backIndex
            padFamily: resumeCanvas.padFamily
            titleFont: Qt.font({ family: Kit.Tokens.faceLabel, pixelSize: Kit.Tokens.fontTitle, weight: Kit.Tokens.weightTitle })
            bodyFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
            captionFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontCaption })
            rowFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
            helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
            help: HelpModel {
                screen: HelpModel.QuestionBack
            }
            visible: !resumeCanvas.eclipseMode
            showing: !resumeCanvas.eclipseMode && resumeCanvas.session.resumeAsked
            onChosen: (index) => resumeCanvas.session.answerResume(index)
        }
        Loader {
            id: eclipseResumeLoader
            active: resumeCanvas.eclipseMode
            sourceComponent: ResumeEclipse {
                id: eclipseResume
                controller: resumeFrame.controller
                session: resumeCanvas.session
                question: resumeWords
                titleText: resumeCanvas.subjectTitle
                cover: resumeCanvas.coverUrl
                library: resumeCanvas.library
                clock: resumeCanvas.clock
                battery: resumeCanvas.battery
                padFamily: resumeCanvas.padFamily
                bigScreen: resumeCanvas.bigScreen
                showing: resumeCanvas.eclipseMode && resumeCanvas.session.resumeAsked
                onChosen: (index) => resumeCanvas.session.answerResume(index)
                Binding {
                    eclipseResume.visible: resumeCanvas.eclipseMode && resumeCanvas.session.resumeAsked
                }
            }
        }
    }
}
