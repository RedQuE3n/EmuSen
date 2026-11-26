import QtQuick
import QtQml
import EmuSen.Kit as Kit

// Question Escape dismissal, onClosed fallback, and synchronous step transitions.
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: "#000000"
    property bool showing: false
    property string titleText: ""
    property string stepTwoTitle: ""
    property string choicesJoined: ""
    property string stepTwoChoices: ""
    property string separator: ""
    property int chosen: -2
    property int chosenCount: 0
    property int step: 1
    property bool autoAdvance: false
    property bool reopenViaToggle: true
    property bool dismissExternally: false
    readonly property bool sheetOpen: frame.controller.sheetOpen
    onDismissExternallyChanged: if (scene.dismissExternally) question.close()
    Binding {
        target: question
        property: "visible"
        value: scene.showing
    }
    Kit.Frame {
        id: frame
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: true
        Kit.Question {
            id: question
            objectName: "question"
            controller: frame.controller
            title: scene.titleText
            choices: scene.choicesJoined.length > 0 ? scene.choicesJoined.split(scene.separator) : []
            backIndex: 0
            onChosen: (index) => {
                scene.chosen = index
                scene.chosenCount += 1
                if (scene.autoAdvance && scene.step === 1) {
                    scene.step = 2
                    if (scene.reopenViaToggle)
                        scene.showing = false
                    scene.titleText = scene.stepTwoTitle
                    scene.choicesJoined = scene.stepTwoChoices
                    scene.showing = true
                } else {
                    scene.showing = false
                }
            }
        }
    }
}
