pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Docked Stage Resume Card (03-docked-stage.html) over the held unplayed game.
Kit.Sheet {
    id: root
    objectName: "eclipseResumeSheet"
    required property SessionModel session
    required property QuestionModel question
    property string titleText: ""
    property string cover: ""
    property LibraryModel library: null
    property Clock clock: null
    property int battery: -1
    property int padFamily: Kit.Families.Xbox
    property bool bigScreen: true
    property bool showing: false
    property bool awaiting: false
    readonly property real boardWidth: 1920
    readonly property real boardHeight: 1200
    readonly property string resolvedTitle: root.titleText.length > 0 ? root.titleText : root.session.resumeSubject
    readonly property real u: cardContainer.height / root.boardHeight
    readonly property real canvasUnit: canvas.unit
    readonly property real pixelScale: canvas.unit
    readonly property real screenPerBoard: Math.max(0.01, root.u * root.canvasUnit)
    readonly property bool desktopMode: !root.bigScreen
    readonly property real dockWidth: root.desktopMode ? Math.min(cardContainer.width * 0.36, Math.max(576 * root.u, 360)) : Math.min(576 * root.u, cardContainer.width * 0.34)
    readonly property real dockHeight: Math.max(200 * root.u, cardContainer.height - footer.height - (root.desktopMode ? 36 : 88) * root.u)
    readonly property bool stageWaitingPicture: root.session.resumePicture.length > 0 && (stageImage.status === Image.Loading || stageImage.status === Image.Null)
    readonly property bool stageCoverWaiting: root.cover.length > 0 && (stageCover.status === Image.Loading || stageCover.status === Image.Null)
    readonly property bool stageHasPicture: root.session.resumePicture.length > 0 && stageImage.status === Image.Ready
    readonly property bool stageHasCover: !root.stageHasPicture && !root.stageWaitingPicture && root.cover.length > 0 && stageCover.status === Image.Ready
    readonly property bool stageShowTier2: !root.stageHasPicture && !root.stageWaitingPicture && !root.stageHasCover && !root.stageCoverWaiting

    signal chosen(int index)
    signal pick(int index)
    signal rearm()
    signal dismissed()

    component FallbackPill: Rectangle {
        id: pill
        required property string pillText
        required property real maxWidth
        anchors.horizontalCenter: parent.horizontalCenter
        width: Math.min(pill.maxWidth, pillRow.implicitWidth + 28 * root.u)
        height: Math.max(34 * root.u, 26 / root.canvasUnit)
        radius: height / 2
        color: ThemeInk.glassPill
        border.width: Math.max(1, 1 * root.u)
        border.color: ThemeInk.coronaSoft

        Row {
            id: pillRow
            anchors.centerIn: parent
            spacing: 8 * root.u

            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(7 * root.u, 5 / root.canvasUnit)
                height: width
                radius: width / 2
                color: ThemeInk.corona
            }

            CardLine {
                anchors.verticalCenter: parent.verticalCenter
                unit: root.u
                text: pill.pillText
                size: Math.max(14, Math.ceil(12 / root.screenPerBoard))
                color: Kit.Tokens.text
                elide: Text.ElideRight
            }
        }
    }

    anchors.centerIn: undefined
    parent: Overlay.overlay
    x: 0
    y: 0
    width: parent ? parent.width : 0
    height: parent ? parent.height : 0
    padding: 0
    background: null
    Overlay.modal: Item {}
    closePolicy: Popup.NoAutoClose

    onRearm: {
        actionList.currentIndex = 0
        root.awaiting = true
    }
    onPick: (index) => {
        if (!root.awaiting)
            return
        root.awaiting = false
        root.chosen(index)
        if (root.showing && root.opened) {
            actionList.currentIndex = 0
            root.awaiting = true
        }
    }
    onDismissed: root.pick(-1)
    onAboutToShow: {
        root.controller.sheetOpen = true
        root.rearm()
        actionList.forceActiveFocus()
    }
    onOpened: actionList.forceActiveFocus()
    onVisibleChanged: if (!root.visible) root.controller.sheetOpen = false
    onShowingChanged: {
        if (root.showing)
            root.rearm()
        else
            root.awaiting = false
    }
    onClosed: {
        if (root.showing) {
            root.open()
            root.rearm()
        } else if (root.awaiting) {
            root.dismissed()
        }
    }
    Connections {
        target: root.controller
        enabled: root.awaiting
        function onSheetDismissed() { root.dismissed() }
    }

    Kit.NormalizedCanvas {
        id: canvas
        active: root.bigScreen

        Rectangle {
            id: backdrop
            objectName: "eclipseResumeBackdrop"
            anchors.fill: parent
            color: root.bigScreen ? ThemeInk.voidFill : ThemeInk.scrimDialog

            Shape {
                anchors.fill: parent
                visible: root.bigScreen
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    strokeColor: "transparent"
                    fillGradient: RadialGradient {
                        centerX: canvas.width * 0.78
                        centerY: canvas.height * 0.10
                        centerRadius: canvas.height * 0.78
                        focalX: canvas.width * 0.78
                        focalY: canvas.height * 0.10
                        GradientStop { position: 0.0; color: ThemeInk.glowWeak }
                        GradientStop { position: 1.0; color: "transparent" }
                    }
                    PathRectangle {
                        width: canvas.width
                        height: canvas.height
                    }
                }
                ShapePath {
                    strokeColor: "transparent"
                    fillGradient: RadialGradient {
                        centerX: canvas.width * 0.12
                        centerY: canvas.height * 0.90
                        centerRadius: canvas.height * 0.65
                        focalX: canvas.width * 0.12
                        focalY: canvas.height * 0.90
                        GradientStop { position: 0.0; color: ThemeInk.coronaGlowSoft }
                        GradientStop { position: 1.0; color: "transparent" }
                    }
                    PathRectangle {
                        width: canvas.width
                        height: canvas.height
                    }
                }
            }
        }

        Item {
            id: cardContainer
            objectName: "eclipseResumeCard"
            readonly property real screenPerBoard: root.screenPerBoard
            width: root.bigScreen ? canvas.width : Math.max(320, Math.min(canvas.width - 48, 1160))
            height: root.bigScreen ? canvas.height : Math.max(240, Math.min(canvas.height - 48, 600))
            anchors.centerIn: parent

            Kit.Glow {
                visible: root.desktopMode
                anchors.fill: parent
                radius: 28 * root.u
                color: ThemeInk.shadowHeavy
                blur: 64 * root.u
                offsetY: 18 * root.u
                pixelScale: root.pixelScale
            }

            Rectangle {
                visible: root.desktopMode
                anchors.fill: parent
                radius: 24 * root.u
                color: ThemeInk.voidFill
                border.width: Math.max(1, 1 * root.u)
                border.color: ThemeInk.hairline
                clip: true
            }

            // Left docked control column.
            Item {
                id: leftDockWrap
                x: (root.desktopMode ? 28 : 52) * root.u
                y: (root.desktopMode ? 20 : 48) * root.u
                width: root.dockWidth
                height: root.dockHeight

                Kit.Glow {
                    anchors.fill: leftDock
                    radius: 28 * root.u
                    color: ThemeInk.shadowHeavy
                    blur: 80 * root.u
                    offsetX: 24 * root.u
                    pixelScale: root.pixelScale
                }

                Rectangle {
                    id: leftDock
                    objectName: "eclipseResumeDock"
                    anchors.fill: parent
                    radius: 28 * root.u
                    border.width: Math.max(1, 1 * root.u)
                    border.color: ThemeInk.hairline
                    gradient: Gradient {
                        GradientStop { position: 0.0; color: ThemeInk.surfaceRaised }
                        GradientStop { position: 0.58; color: ThemeInk.inputSurface }
                        GradientStop { position: 1.0; color: ThemeInk.surface }
                    }

                    Rectangle {
                        objectName: "eclipseResumeDockCoronaEdge"
                        x: parent.width - width
                        y: 96 * root.u
                        width: Math.max(1, 2 * root.u)
                        height: Math.max(24 * root.u, parent.height - 192 * root.u)
                        gradient: Gradient {
                            GradientStop { position: 0.0; color: "transparent" }
                            GradientStop { position: 0.5; color: ThemeInk.corona }
                            GradientStop { position: 1.0; color: "transparent" }
                        }
                        Kit.Glow {
                            anchors.fill: parent
                            color: ThemeInk.coronaGlowSoft
                            blur: 24 * root.u
                            spread: 3 * root.u
                            pixelScale: root.pixelScale
                        }
                    }

                    Item {
                        id: dockInner
                        anchors.fill: parent
                        anchors.topMargin: (root.desktopMode ? 18 : 38) * root.u
                        anchors.bottomMargin: (root.desktopMode ? 16 : 36) * root.u
                        anchors.leftMargin: (root.desktopMode ? 26 : 42) * root.u
                        anchors.rightMargin: (root.desktopMode ? 26 : 42) * root.u

                        Item {
                            id: dockTopBar
                            anchors.top: parent.top
                            anchors.left: parent.left
                            anchors.right: parent.right
                            height: Math.max(28 * root.u, 22 / root.canvasUnit)

                            Row {
                                anchors.left: parent.left
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 12 * root.u

                                Item {
                                    objectName: "eclipseResumeHeaderMark"
                                    width: 28 * root.u
                                    height: 28 * root.u
                                    anchors.verticalCenter: parent.verticalCenter

                                    Shape {
                                        width: 34
                                        height: 34
                                        scale: (28 * root.u) / 34
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: ThemeInk.corona
                                            strokeColor: "transparent"
                                            fillRule: ShapePath.OddEvenFill
                                            PathSvg { path: Marks.menuCrescent }
                                        }
                                    }
                                }

                                CardLine {
                                    objectName: "eclipseResumeHeaderTitle"
                                    anchors.verticalCenter: parent.verticalCenter
                                    unit: root.u
                                    text: Words.resumeLaunch
                                    size: Math.max(19, Math.ceil(14 / root.screenPerBoard))
                                    weight: Font.DemiBold
                                    color: ThemeInk.muted
                                }
                            }

                            CardLine {
                                objectName: "eclipseResumeHeaderClock"
                                anchors.right: parent.right
                                anchors.verticalCenter: parent.verticalCenter
                                visible: text.length > 0
                                unit: root.u
                                text: Words.statusClock(root.battery, root.clock ? root.clock.shortTime : "")
                                size: Math.max(19, Math.ceil(14 / root.screenPerBoard))
                                weight: Font.Normal
                                color: ThemeInk.muted
                            }
                        }

                        Column {
                            id: identityBlock
                            anchors.top: dockTopBar.bottom
                            anchors.topMargin: (root.desktopMode ? 14 : 32) * root.u
                            anchors.left: parent.left
                            anchors.right: parent.right

                            Row {
                                id: eyebrowRow
                                width: parent.width
                                height: Math.max(24 * root.u, eyebrowLeft.height)
                                spacing: 8 * root.u
                                visible: root.session.resumeConsole.length > 0 || root.session.resumeCore.length > 0

                                CardLine {
                                    id: eyebrowLeft
                                    objectName: "eclipseResumeEyebrowLeft"
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: text.length > 0
                                    unit: root.u
                                    text: Words.resumeEyebrowLeft(root.session.resumeConsole, root.session.resumeCore.length > 0)
                                    size: Math.max(16.5, Math.ceil(12 / root.screenPerBoard))
                                    weight: Font.DemiBold
                                    spacing: 0.18 * size
                                    capitalization: Font.AllUppercase
                                    color: ThemeInk.rose
                                }

                                CardPlanet {
                                    objectName: "eclipseResumePlanet"
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: root.session.resumePlanet >= -1
                                    unit: root.u
                                    pixelScale: root.pixelScale
                                    side: 20
                                    planet: root.session.resumePlanet
                                }

                                CardLine {
                                    objectName: "eclipseResumeEyebrowCore"
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: Math.max(0, eyebrowRow.width - eyebrowLeft.width - (root.session.resumePlanet >= -1 ? 28 * root.u : 0) - 16 * root.u)
                                    visible: text.length > 0
                                    unit: root.u
                                    text: Words.resumeEyebrowCore(root.session.resumeCore)
                                    size: Math.max(16.5, Math.ceil(12 / root.screenPerBoard))
                                    weight: Font.DemiBold
                                    spacing: 0.18 * size
                                    capitalization: Font.AllUppercase
                                    color: ThemeInk.rose
                                    elide: Text.ElideRight
                                }
                            }

                            Item { width: 1; height: (root.desktopMode ? 6 : 14) * root.u; visible: eyebrowRow.visible }

                            CardLine {
                                id: gameTitleText
                                objectName: "eclipseResumeTitle"
                                width: parent.width
                                unit: root.u
                                text: root.resolvedTitle
                                size: Math.max(root.resolvedTitle.length > 28 ? 34 : 40, Math.ceil(22 / root.screenPerBoard))
                                weight: Font.DemiBold
                                lineHeight: 1.12
                                lineHeightMode: Text.ProportionalHeight
                                spacing: -0.02 * size
                                color: Kit.Tokens.text
                                wrapMode: Text.Wrap
                                maximumLineCount: 3
                                elide: Text.ElideRight
                            }

                            Item { width: 1; height: (root.desktopMode ? 10 : 20) * root.u }

                            Rectangle {
                                id: savedPill
                                objectName: "eclipseResumeSavedPill"
                                implicitWidth: Math.min(parent.width, savedPillRow.implicitWidth + 20 * root.u)
                                width: implicitWidth
                                height: Math.max(36 * root.u, 28 / root.canvasUnit)
                                radius: height / 2
                                color: Qt.alpha(Kit.Tokens.surface, 0.65)
                                border.width: Math.max(1, 1 * root.u)
                                border.color: Qt.alpha(ThemeInk.corona, 0.34)

                                Row {
                                    id: savedPillRow
                                    anchors.verticalCenter: parent.verticalCenter
                                    anchors.left: parent.left
                                    anchors.leftMargin: 7 * root.u
                                    spacing: 10 * root.u

                                    Rectangle {
                                        objectName: "eclipseResumeHerePill"
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: hereLabel.width + 18 * root.u
                                        height: Math.max(22 * root.u, 18 / root.canvasUnit)
                                        radius: height / 2
                                        color: ThemeInk.corona

                                        CardLine {
                                            id: hereLabel
                                            anchors.centerIn: parent
                                            unit: root.u
                                            text: Words.resumeHere
                                            size: Math.max(11.5, Math.ceil(10 / root.screenPerBoard))
                                            weight: Font.Bold
                                            spacing: 0.08 * size
                                            color: ThemeInk.overAccent
                                        }
                                    }

                                    CardLine {
                                        id: savedBadgeText
                                        objectName: "eclipseResumeSavedLine"
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: Math.min(implicitWidth, dockInner.width - hereLabel.width - 54 * root.u)
                                        unit: root.u
                                        text: Words.resumeSavedLine(root.session.resumeWhen)
                                        size: Math.max(15.5, Math.ceil(12 / root.screenPerBoard))
                                        weight: Font.Normal
                                        color: Kit.Tokens.text
                                        elide: Text.ElideRight
                                    }
                                }
                            }
                        }

                        Item {
                            id: promptBlock
                            anchors.top: identityBlock.bottom
                            anchors.topMargin: (root.desktopMode ? 12 : 28) * root.u
                            anchors.left: parent.left
                            anchors.right: parent.right
                            height: promptTopRule.height + promptCol.anchors.topMargin + promptCol.height

                            Rectangle {
                                id: promptTopRule
                                anchors.top: parent.top
                                anchors.left: parent.left
                                anchors.right: parent.right
                                height: Math.max(1, 1 * root.u)
                                color: ThemeInk.hairline
                            }

                            Column {
                                id: promptCol
                                anchors.top: promptTopRule.bottom
                                anchors.topMargin: (root.desktopMode ? 12 : 24) * root.u
                                anchors.left: parent.left
                                anchors.right: parent.right
                                spacing: (root.desktopMode ? 4 : 10) * root.u

                                CardLine {
                                    objectName: "eclipseResumePromptTitle"
                                    width: parent.width
                                    unit: root.u
                                    text: root.question.title
                                    size: Math.max(26, Math.ceil(17 / root.screenPerBoard))
                                    weight: Font.DemiBold
                                    color: ThemeInk.corona
                                    wrapMode: Text.Wrap
                                    maximumLineCount: 2
                                }

                                CardLine {
                                    objectName: "eclipseResumePromptBody"
                                    width: parent.width
                                    unit: root.u
                                    text: root.question.sentences.length > 0 ? root.question.sentences[0] : ""
                                    size: Math.max(19, Math.ceil(14 / root.screenPerBoard))
                                    weight: Font.Light
                                    lineHeight: root.desktopMode ? 1.32 : 1.46
                                    lineHeightMode: Text.ProportionalHeight
                                    color: ThemeInk.body
                                    wrapMode: Text.Wrap
                                    maximumLineCount: 3
                                    elide: Text.ElideRight
                                }
                            }
                        }

                        Item {
                            id: actionsSlot
                            objectName: "eclipseResumeActionsSlot"
                            anchors.top: promptBlock.bottom
                            anchors.bottom: dockCaptionBlock.top
                            anchors.left: parent.left
                            anchors.right: parent.right

                            ListView {
                                id: actionList
                                objectName: "eclipseResumeChoices"
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.right: parent.right
                                height: contentHeight
                                model: root.question.choices
                                spacing: root.desktopMode ? Math.max(8 * root.u, 8 / root.canvasUnit) : 16 * root.u
                                focus: true
                                keyNavigationWraps: false
                                interactive: false
                                highlightMoveDuration: Kit.Tokens.instant
                                highlightResizeDuration: Kit.Tokens.instant
                                delegate: Kit.Walkable {
                                    id: answer
                                    required property string modelData
                                    required property int index
                                    readonly property bool current: ListView.isCurrentItem
                                    objectName: "eclipseResumeAnswer." + answer.index
                                    controller: root.controller
                                    width: actionList.width
                                    height: answer.current ? Math.max(76 * root.u, 48 / root.canvasUnit) : Math.max(70 * root.u, 48 / root.canvasUnit)
                                    focus: answer.current
                                    Accessible.role: Accessible.Button
                                    Accessible.name: answer.modelData
                                    onActiveFocusChanged: actionList.currentIndex = answer.activeFocus ? answer.index : actionList.currentIndex
                                    onActivated: root.pick(answer.index)
                                    Keys.onUpPressed: (event) => {
                                        actionList.decrementCurrentIndex()
                                        event.accepted = true
                                    }
                                    Keys.onLeftPressed: (event) => {
                                        actionList.decrementCurrentIndex()
                                        event.accepted = true
                                    }
                                    Keys.onDownPressed: (event) => {
                                        actionList.incrementCurrentIndex()
                                        event.accepted = true
                                    }
                                    Keys.onRightPressed: (event) => {
                                        actionList.incrementCurrentIndex()
                                        event.accepted = true
                                    }
                                    Keys.onTabPressed: (event) => {
                                        actionList.currentIndex = (actionList.currentIndex + 1) % Math.max(1, actionList.count)
                                        event.accepted = true
                                    }
                                    Keys.onBacktabPressed: (event) => {
                                        actionList.currentIndex = (actionList.currentIndex + actionList.count - 1) % Math.max(1, actionList.count)
                                        event.accepted = true
                                    }
                                    Keys.onSpacePressed: (event) => {
                                        root.pick(answer.index)
                                        event.accepted = true
                                    }

                                    EclipsePillHighlight {
                                        objectName: "eclipseResumeAnswerHighlight." + answer.index
                                        visible: answer.current
                                        anchors.fill: parent
                                        unit: root.u
                                        pixelScale: root.pixelScale
                                        radius: answer.height / 2
                                        rowIndex: answer.index
                                    }

                                    Rectangle {
                                        objectName: "eclipseResumeAnswerBg." + answer.index
                                        anchors.fill: parent
                                        radius: height / 2
                                        color: answer.current ? Kit.Tokens.accent : Qt.alpha(Kit.Tokens.surface, 0.45)
                                        border.width: answer.current ? 0 : Math.max(1, 2 * root.u)
                                        border.color: ThemeInk.pillBorder
                                    }

                                    CardGlyph {
                                        id: answerGlyph
                                        objectName: "eclipseResumeAnswerGlyph." + answer.index
                                        visible: answer.current
                                        anchors.left: parent.left
                                        anchors.leftMargin: 16 * root.u
                                        anchors.verticalCenter: parent.verticalCenter
                                        unit: root.u
                                        family: root.padFamily
                                        button: Kit.Pad.Accept
                                        side: 46
                                        lead: true
                                        leads: true
                                        leadFill: Kit.Tokens.surface
                                        leadInk: Kit.Tokens.accent
                                    }

                                    CardLine {
                                        id: primaryLabel
                                        objectName: "eclipseResumeAnswerLabel." + answer.index
                                        anchors.left: parent.left
                                        anchors.leftMargin: answer.current ? Math.max(78 * root.u, answerGlyph.width + 28 * root.u) : 26 * root.u
                                        anchors.verticalCenter: parent.verticalCenter
                                        unit: root.u
                                        boardLine: true
                                        text: answer.modelData
                                        size: answer.current ? Math.max(25, Math.ceil(16 / root.screenPerBoard)) : Math.max(23, Math.ceil(15 / root.screenPerBoard))
                                        weight: answer.current ? Font.DemiBold : Font.Normal
                                        color: answer.current ? Kit.Tokens.overAccent : Kit.Tokens.text
                                    }

                                    CardLine {
                                        objectName: "eclipseResumeAnswerSub." + answer.index
                                        anchors.right: parent.right
                                        anchors.rightMargin: 24 * root.u
                                        anchors.left: primaryLabel.right
                                        anchors.leftMargin: 12 * root.u
                                        anchors.verticalCenter: parent.verticalCenter
                                        horizontalAlignment: Text.AlignRight
                                        elide: Text.ElideRight
                                        unit: root.u
                                        text: answer.index === 0
                                            ? (root.session.resumeWhen.length > 0 ? root.session.resumeWhen : Words.resumeContinueFallbackSub)
                                            : Words.resumeStartAgainSub
                                        size: Math.max(16.5, Math.ceil(13 / root.screenPerBoard))
                                        weight: Font.Normal
                                        color: answer.current ? ThemeInk.overAccentMuted : ThemeInk.dim
                                    }
                                }
                            }
                        }

                        Item {
                            id: dockCaptionBlock
                            objectName: "eclipseResumeCaptionBlock"
                            anchors.bottom: parent.bottom
                            anchors.left: parent.left
                            anchors.right: parent.right
                            height: captionRule.height + captionLine.anchors.topMargin + captionLine.height

                            Rectangle {
                                id: captionRule
                                anchors.top: parent.top
                                anchors.left: parent.left
                                anchors.right: parent.right
                                height: Math.max(1, 1 * root.u)
                                color: ThemeInk.hairline
                            }

                            CardLine {
                                id: captionLine
                                objectName: "eclipseResumeCaption"
                                anchors.top: captionRule.bottom
                                anchors.topMargin: (root.desktopMode ? 10 : 18) * root.u
                                anchors.left: parent.left
                                anchors.right: parent.right
                                unit: root.u
                                text: root.question.caption
                                size: Math.max(16, Math.ceil(13 / root.screenPerBoard))
                                weight: Font.Normal
                                lineHeight: root.desktopMode ? 1.32 : 1.46
                                lineHeightMode: Text.ProportionalHeight
                                color: ThemeInk.muted
                                wrapMode: Text.Wrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                            }
                        }
                    }
                }
            }

            // Right stage viewport and foot bar.
            Item {
                id: viewportColumn
                objectName: "eclipseResumeStageCol"
                x: leftDockWrap.x + leftDockWrap.width + (root.desktopMode ? 24 : 52) * root.u
                y: (root.desktopMode ? 24 : 58) * root.u
                width: Math.max(200 * root.u, cardContainer.width - x - (root.desktopMode ? 24 : 56) * root.u)
                height: Math.max(160 * root.u, cardContainer.height - y - footer.height - (root.desktopMode ? 18 : 40) * root.u)

                Item {
                    id: stageWellWrap
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    height: Math.min(888 * root.u, Math.max(100 * root.u, viewportColumn.height - stageFoot.height - 18 * root.u))

                    Kit.Glow {
                        anchors.fill: stageWell
                        radius: 24 * root.u
                        color: ThemeInk.shadowHeavy
                        blur: 96 * root.u
                        offsetY: 32 * root.u
                        pixelScale: root.pixelScale
                    }

                    Kit.Glow {
                        anchors.fill: stageWell
                        radius: 24 * root.u
                        color: Qt.alpha(ThemeInk.rose, 0.16)
                        blur: 60 * root.u
                        pixelScale: root.pixelScale
                    }

                    Rectangle {
                        id: stageWell
                        objectName: "eclipseResumeStageWell"
                        anchors.fill: parent
                        radius: 24 * root.u
                        color: Qt.rgba(0, 0, 0, 1)
                        clip: GraphicsInfo.api === GraphicsInfo.Software

                        Item {
                            id: stageLayers
                            anchors.fill: parent

                            // Pure black well behind slot 0 screenshot without prematurely hiding background while loading.
                            Rectangle {
                                anchors.fill: parent
                                radius: stageWell.radius
                                color: Qt.rgba(0, 0, 0, 1)
                                visible: root.stageHasPicture || root.stageWaitingPicture
                            }

                            // Slot 0 screenshot without sourceSize so implicitWidth/Height stay native.
                            Image {
                                id: stageImage
                                objectName: "eclipseResumeStageImage"
                                anchors.fill: parent
                                source: root.session.resumePicture
                                fillMode: Image.PreserveAspectFit
                                asynchronous: true
                                cache: false
                                visible: root.stageHasPicture
                                smooth: !(stageImage.paintedWidth > 0 && stageImage.implicitWidth > 0 && stageImage.paintedWidth * root.pixelScale >= stageImage.implicitWidth * 2 && stageImage.implicitHeight <= 288)
                            }

                            // Tier 1 fallback: procedural art plate + fitted game cover + pill.
                            Item {
                                id: tier1Stage
                                objectName: "eclipseResumeCoverPlate"
                                anchors.fill: parent
                                visible: root.stageHasCover

                                EclipseArtPlate {
                                    anchors.fill: parent
                                    unit: root.u
                                    pixelScale: root.pixelScale
                                    radius: 24
                                    cover: ""
                                    slot: 0
                                }

                                Column {
                                    anchors.centerIn: parent
                                    spacing: 18 * root.u

                                    Item {
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        width: Math.min(stageWell.width * 0.42, 320 * root.u)
                                        height: Math.min(stageWell.height * 0.56, 420 * root.u)

                                        Kit.Glow {
                                            anchors.fill: stageCover
                                            radius: 12 * root.u
                                            color: ThemeInk.shadowHeavy
                                            blur: 32 * root.u
                                            offsetY: 12 * root.u
                                            pixelScale: root.pixelScale
                                        }

                                        Kit.RoundPicture {
                                            id: stageCover
                                            objectName: "eclipseResumeCoverImage"
                                            anchors.fill: parent
                                            source: !root.stageHasPicture && !root.stageWaitingPicture ? root.cover : ""
                                            radius: 12 * root.u
                                            pixelScale: root.pixelScale
                                            crop: false
                                        }
                                    }

                                    FallbackPill {
                                        objectName: "eclipseResumeFallbackPill"
                                        maxWidth: stageWell.width - 48 * root.u
                                        pillText: Words.resumeNoPreview
                                    }
                                }
                            }

                            // Tier 2 fallback: unscraped ROM surface plate + crescent emblem + state pill.
                            Rectangle {
                                id: tier2Stage
                                objectName: "eclipseResumeTier2Plate"
                                anchors.fill: parent
                                radius: stageWell.radius
                                visible: root.stageShowTier2
                                gradient: Gradient {
                                    GradientStop { position: 0.0; color: ThemeInk.surfaceRaised }
                                    GradientStop { position: 1.0; color: ThemeInk.surface }
                                }

                                Shape {
                                    anchors.fill: parent
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        strokeColor: "transparent"
                                        fillGradient: RadialGradient {
                                            centerX: tier2Stage.width * 0.5
                                            centerY: tier2Stage.height * 0.42
                                            centerRadius: tier2Stage.width * 0.45
                                            focalX: tier2Stage.width * 0.5
                                            focalY: tier2Stage.height * 0.42
                                            GradientStop { position: 0.0; color: ThemeInk.coronaGlowSoft }
                                            GradientStop { position: 1.0; color: "transparent" }
                                        }
                                        PathRectangle {
                                            width: tier2Stage.width
                                            height: tier2Stage.height
                                            radius: stageWell.radius
                                        }
                                    }
                                }

                                Column {
                                    anchors.centerIn: parent
                                    spacing: 20 * root.u

                                    Rectangle {
                                        objectName: "eclipseResumeTier2Crescent"
                                        anchors.horizontalCenter: parent.horizontalCenter
                                        width: 88 * root.u
                                        height: 88 * root.u
                                        radius: width / 2
                                        color: ThemeInk.voidFill
                                        border.width: Math.max(1.5, 1.5 * root.u)
                                        border.color: ThemeInk.coronaSoft

                                        Kit.Glow {
                                            anchors.fill: parent
                                            radius: parent.radius
                                            color: ThemeInk.coronaGlowSoft
                                            blur: 28 * root.u
                                            pixelScale: root.pixelScale
                                        }

                                        Shape {
                                            width: 34
                                            height: 34
                                            x: (parent.width - 40 * root.u) / 2
                                            y: (parent.height - 40 * root.u) / 2
                                            scale: (40 * root.u) / 34
                                            transformOrigin: Item.TopLeft
                                            preferredRendererType: Shape.CurveRenderer
                                            ShapePath {
                                                fillColor: ThemeInk.corona
                                                strokeColor: "transparent"
                                                fillRule: ShapePath.OddEvenFill
                                                PathSvg { path: Marks.menuCrescent }
                                            }
                                        }
                                    }

                                    FallbackPill {
                                        objectName: "eclipseResumeTier2Pill"
                                        maxWidth: stageWell.width - 48 * root.u
                                        pillText: Words.resumeFootLine(root.session.resumeWhen)
                                    }
                                }
                            }
                        }

                        ShaderEffectSource {
                            id: stageTex
                            sourceItem: GraphicsInfo.api !== GraphicsInfo.Software ? stageLayers : null
                            hideSource: GraphicsInfo.api !== GraphicsInfo.Software
                            live: true
                            visible: false
                            textureSize: Qt.size(Math.max(1, Math.ceil(stageWell.width * root.pixelScale * Screen.devicePixelRatio)), Math.max(1, Math.ceil(stageWell.height * root.pixelScale * Screen.devicePixelRatio)))
                        }

                        Shape {
                            anchors.fill: parent
                            visible: GraphicsInfo.api !== GraphicsInfo.Software
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                strokeColor: "transparent"
                                fillColor: "transparent"
                                fillItem: stageTex
                                fillTransform: PlanarTransform.fromScale(stageWell.width / Math.max(1, stageTex.textureSize.width), stageWell.height / Math.max(1, stageTex.textureSize.height))
                                PathRectangle {
                                    width: stageWell.width
                                    height: stageWell.height
                                    radius: stageWell.radius
                                }
                            }
                        }

                        Rectangle {
                            id: stageBorderOverlay
                            objectName: "eclipseResumeStageBorder"
                            anchors.fill: parent
                            z: 10
                            radius: stageWell.radius
                            color: "transparent"
                            border.width: Math.max(1, 2 * root.u)
                            border.color: Qt.alpha(ThemeInk.corona, 0.28)
                        }
                    }
                }

                Item {
                    id: stageFoot
                    objectName: "eclipseResumeStageFoot"
                    anchors.top: stageWellWrap.bottom
                    anchors.topMargin: (root.desktopMode ? 10 : 18) * root.u
                    anchors.left: parent.left
                    anchors.right: parent.right
                    height: Math.max(44 * root.u, 32 / root.canvasUnit)

                    CardLine {
                        objectName: "eclipseResumeStageCaption"
                        anchors.left: parent.left
                        anchors.leftMargin: 4 * root.u
                        anchors.right: stageMetaPill.left
                        anchors.rightMargin: 16 * root.u
                        anchors.verticalCenter: parent.verticalCenter
                        unit: root.u
                        text: Words.resumeFootLine(root.session.resumeWhen)
                        size: Math.max(18, Math.ceil(13 / root.screenPerBoard))
                        color: ThemeInk.body
                        elide: Text.ElideRight
                    }

                    Rectangle {
                        id: stageMetaPill
                        objectName: "eclipseResumeStageMetaPill"
                        anchors.right: parent.right
                        anchors.verticalCenter: parent.verticalCenter
                        visible: !root.stageWaitingPicture && !(root.cover.length > 0 && root.stageCoverWaiting && !root.stageHasPicture)
                        width: metaText.width + 28 * root.u
                        height: Math.max(34 * root.u, 26 / root.canvasUnit)
                        radius: height / 2
                        color: Kit.Tokens.surface
                        border.width: Math.max(1, 1 * root.u)
                        border.color: ThemeInk.hairline

                        CardLine {
                            id: metaText
                            objectName: "eclipseResumeStageMetaText"
                            anchors.centerIn: parent
                            unit: root.u
                            text: Words.resumeStageMeta(root.stageHasPicture ? stageImage.implicitWidth : 0, root.stageHasPicture ? stageImage.implicitHeight : 0)
                            size: Math.max(14, Math.ceil(12 / root.screenPerBoard))
                            family: Kit.Tokens.faceCode
                            color: ThemeInk.muted
                        }
                    }
                }
            }

            // Grounded full-width bottom bar.
            Rectangle {
                id: footer
                objectName: "eclipseResumeBottomBar"
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                height: Math.max(76 * root.u, 48 / root.canvasUnit)
                color: ThemeInk.surface

                Rectangle {
                    anchors.top: parent.top
                    anchors.left: parent.left
                    anchors.right: parent.right
                    height: Math.max(1, 2 * root.u)
                    color: Kit.Tokens.rule
                }

                HelpModel {
                    id: resumeHelp
                    objectName: "resumeHelp"
                    screen: HelpModel.QuestionBack
                }

                Kit.HelpBar {
                    id: helpBar
                    objectName: "eclipseResumeHelpBar"
                    anchors.left: parent.left
                    anchors.leftMargin: (root.desktopMode ? 28 : 56) * root.u
                    anchors.verticalCenter: parent.verticalCenter
                    controller: root.controller
                    model: resumeHelp
                    family: root.padFamily
                    fitted: true
                    ruled: false
                    color: "transparent"
                    labelColor: ThemeInk.body
                    iconColor: Kit.Tokens.muted
                    leadStyle: Kit.GlyphStyles.Filled
                    leadColor: Kit.Tokens.accent
                    entrySpacing: 34 * root.u
                    glyphGap: 10 * root.u
                    glyphSize: 34 * root.u
                    snapped: false
                    label: Component {
                        CardLine {
                            unit: root.u
                            boardLine: true
                            size: Math.max(21, Math.ceil(13 / root.screenPerBoard))
                        }
                    }
                    glyph: Component {
                        CardGlyph {
                            unit: root.u
                            side: 34
                            ink: ThemeInk.body
                        }
                    }
                }

                CardLine {
                    objectName: "eclipseResumeCounter"
                    anchors.right: parent.right
                    anchors.rightMargin: (root.desktopMode ? 28 : 56) * root.u
                    anchors.verticalCenter: parent.verticalCenter
                    unit: root.u
                    text: Words.positionLine(actionList.currentIndex, root.question.choices.length)
                    size: Math.max(21, Math.ceil(13 / root.screenPerBoard))
                    family: Kit.Tokens.faceCode
                    color: ThemeInk.muted
                }
            }
        }
    }
}
