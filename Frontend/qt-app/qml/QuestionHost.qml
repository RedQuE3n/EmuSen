pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// Question host: Kit.Question under reference tokens, QuestionEclipse when coronaRim > 0 (R-20261113-5, R-20261114-21).
Item {
    id: root
    required property Kit.FrameController controller
    property string title
    property string message
    property list<string> sentences
    property string caption
    property string picture
    property list<string> choices
    property int backIndex: -1
    property var help: null
    property int padFamily: Kit.Families.Generic
    property font titleFont: Qt.font({ family: Kit.Tokens.faceInterface, pixelSize: Kit.Tokens.fontTitle, weight: Kit.Tokens.weightTitle })
    property font bodyFont: Qt.font({ family: Kit.Tokens.faceInterface, pixelSize: Kit.Tokens.fontBody })
    property font captionFont: Qt.font({ family: Kit.Tokens.faceInterface, pixelSize: Kit.Tokens.fontCaption })
    property font rowFont: Qt.font({ family: Kit.Tokens.faceInterface, pixelSize: Kit.Tokens.fontRow })
    property font helpFont: Qt.font({ family: Kit.Tokens.faceInterface, pixelSize: Kit.Tokens.fontBody })
    property real helpSpacing: Kit.Tokens.paneGap
    property real helpGlyphGap: Kit.Tokens.base * 2
    property bool showing: false
    readonly property bool eclipse: Kit.Tokens.coronaRim > 0
    readonly property Kit.Question refSheet: refLoader.item as Kit.Question
    readonly property QuestionEclipse eclipseSheet: eclipseLoader.item as QuestionEclipse
    readonly property bool opened: root.eclipse ? (root.eclipseSheet?.opened ?? false) : (root.refSheet?.opened ?? false)
    signal chosen(int index)
    visible: false
    Loader {
        id: refLoader
        active: !root.eclipse
        sourceComponent: Kit.Question {
            id: refQuestion
            controller: root.controller
            title: root.title
            message: root.message
            sentences: root.sentences
            caption: root.caption
            choices: root.choices
            backIndex: root.backIndex
            help: root.help
            padFamily: root.padFamily
            titleFont: root.titleFont
            bodyFont: root.bodyFont
            captionFont: root.captionFont
            rowFont: root.rowFont
            helpFont: root.helpFont
            helpSpacing: root.helpSpacing
            helpGlyphGap: root.helpGlyphGap
            onChosen: (index) => root.chosen(index)
            Binding {
                refQuestion.visible: root.showing
            }
        }
    }
    Loader {
        id: eclipseLoader
        active: root.eclipse
        sourceComponent: QuestionEclipse {
            id: eclipseQuestion
            controller: root.controller
            title: root.title
            message: root.message
            sentences: root.sentences
            caption: root.caption
            picture: root.picture
            choices: root.choices
            backIndex: root.backIndex
            padFamily: root.padFamily
            onChosen: (index) => root.chosen(index)
            Binding {
                eclipseQuestion.visible: root.showing
            }
        }
    }
}
