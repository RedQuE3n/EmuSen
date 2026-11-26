pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Eclipse Controller Bindings (#12) and Keyboard Keys (#75), program-drawn in Eclipse's look (R-20261113-5).
FocusScope {
    id: outer
    required property SettingsModel settings
    property int padFamily: Kit.Families.Generic
    property int battery: -1
    property string clockText: ""
    property bool inputs: true
    focus: true
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "controllersEclipseCanvas"
        readonly property SettingsModel settings: outer.settings
        readonly property int padFamily: outer.padFamily
        readonly property int battery: outer.battery
        readonly property string clockText: outer.clockText
        readonly property bool inputs: outer.inputs
        readonly property var pad: outer.settings.padMeta
        EclipseListModel {
            id: padLeftModel
            rows: outer.settings.padLeftRows
        }
        EclipseListModel {
            id: padRightModel
            rows: outer.settings.padRightRows
        }
    component LeaderPass: Repeater {
        required property bool leftSide
        required property bool focusPass
        required property real px
        required property real offsetX
        required property real scrollPx
        required property real diagramWidth
        required property int focusedControl
        required property int headerFocus
        required property bool isKeys
        id: leaderRep
        Shape {
            id: leaderShape
            required property int index
            required property var model
            readonly property var itemData: leaderShape.model.item
            readonly property bool focusedCard: (leaderRep.isKeys || leaderRep.headerFocus < 0) && leaderShape.itemData.index === leaderRep.focusedControl
            readonly property real cardMid: (leaderShape.itemData.y ?? 0) + 28 - leaderRep.scrollPx
            readonly property bool inView: leaderShape.cardMid >= 24 && leaderShape.cardMid <= 496
            objectName: leaderRep.focusPass ? (leaderShape.itemData.leaderFocusedName ?? "") : (leaderShape.itemData.leaderName ?? "")
            visible: leaderShape.inView && leaderShape.focusedCard === leaderRep.focusPass
            anchors.fill: parent
            ShapePath {
                fillColor: "transparent"
                strokeColor: leaderRep.focusPass ? Kit.Tokens.accent : ThemeInk.pillBorder
                strokeWidth: (leaderRep.focusPass ? 2.5 : 1.5) * leaderRep.px
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                startX: ((leaderShape.itemData.ax ?? 0) + leaderRep.offsetX) * leaderRep.px
                startY: (leaderShape.itemData.ay ?? 0) * leaderRep.px
                PathLine { x: ((leaderShape.itemData.elbowX ?? 0) + leaderRep.offsetX) * leaderRep.px; y: (leaderShape.itemData.ay ?? 0) * leaderRep.px }
                PathLine { x: ((leaderShape.itemData.elbowX ?? 0) + leaderRep.offsetX) * leaderRep.px; y: leaderShape.cardMid * leaderRep.px }
                PathLine { x: leaderRep.leftSide ? 0 : leaderRep.diagramWidth; y: leaderShape.cardMid * leaderRep.px }
            }
        }
    }
    component CalloutCard: Item {
        id: card
        required property int rowIndex
        required property var itemData
        required property bool leftSide
        required property real px
        required property int focusedControl
        required property int headerFocus
        required property bool isKeys
        required property SettingsModel modelSettings
        readonly property bool focusedCard: (card.isKeys || card.headerFocus < 0) && card.itemData.index === card.focusedControl
        objectName: card.itemData.cardName ?? ""
        width: 344 * card.px
        height: 56 * card.px
        Kit.FocusRing {
            objectName: card.itemData.ringName ?? ""
            visible: card.focusedCard
            anchors.fill: parent
            radius: 14 * card.px
            gap: 3 * card.px
            thickness: 3 * card.px
            bandRadius: 20 * card.px
            glowColor: Kit.Tokens.glowColor
            glowBlur: 34 * card.px
            glowSpread: 8 * card.px
            pixelScale: canvas.unit
        }
        Rectangle {
            objectName: card.itemData.fillName ?? ""
            anchors.fill: parent
            radius: 14 * card.px
            color: card.focusedCard ? Kit.Tokens.surfaceRaised : ThemeInk.rowSurface
            border.width: card.focusedCard ? 0 : 1 * card.px
            border.color: ThemeInk.hairline
        }
        CardLine {
            id: leftText
            objectName: card.itemData.leftName ?? ""
            x: 16 * card.px
            y: (card.height - leftText.height) / 2
            width: card.leftSide ? Math.min(implicitWidth, Math.max(0, card.width - 32 * card.px - 12 * card.px - rightText.width)) : implicitWidth
            unit: card.px
            boardLine: true
            size: card.leftSide ? 16 : 19
            weight: card.leftSide ? Font.Normal : Font.DemiBold
            color: card.leftSide ? Kit.Tokens.muted : Kit.Tokens.text
            elide: card.leftSide ? Text.ElideRight : Text.ElideNone
            text: card.itemData.left ?? ""
        }
        CardLine {
            id: rightText
            objectName: card.itemData.rightName ?? ""
            x: card.width - rightText.width - 16 * card.px
            y: (card.height - rightText.height) / 2
            width: card.leftSide ? implicitWidth : Math.min(implicitWidth, Math.max(0, card.width - 32 * card.px - 12 * card.px - leftText.width))
            horizontalAlignment: Text.AlignRight
            unit: card.px
            boardLine: true
            size: card.leftSide ? 19 : 16
            weight: card.leftSide ? Font.DemiBold : Font.Normal
            color: card.leftSide ? Kit.Tokens.text : Kit.Tokens.muted
            elide: card.leftSide ? Text.ElideNone : Text.ElideRight
            text: card.itemData.right ?? ""
        }
        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.PointingHandCursor
            onClicked: card.modelSettings.bindingFocus(card.itemData.index ?? card.rowIndex)
        }
    }
    component CalloutColumn: Item {
        id: col
        required property bool leftSide
        required property real px
        required property real reach
        required property bool scrolls
        required property real scrollTarget
        required property var cards
        required property int focusedControl
        required property int headerFocus
        required property bool isKeys
        required property SettingsModel modelSettings
        readonly property real scrollY: col.scrolls ? (colList.contentY + 4 * col.px) : 0
        clip: col.scrolls
        Item {
            anchors.fill: parent
            visible: !col.scrolls
            Repeater {
                model: col.scrolls ? null : col.cards
                CalloutCard {
                    id: flatCard
                    required property int index
                    required property var model
                    rowIndex: flatCard.index
                    itemData: flatCard.model.item
                    leftSide: col.leftSide
                    px: col.px
                    focusedControl: col.focusedControl
                    headerFocus: col.headerFocus
                    isKeys: col.isKeys
                    modelSettings: col.modelSettings
                    x: col.reach
                    y: ((flatCard.itemData.y ?? 0) + 48) * col.px
                }
            }
        }
        ListView {
            id: colList
            anchors.fill: parent
            visible: col.scrolls
            interactive: col.scrolls
            clip: true
            reuseItems: true
            boundsBehavior: Flickable.StopAtBounds
            topMargin: 4 * col.px
            bottomMargin: 4 * col.px
            spacing: 6 * col.px
            model: col.scrolls ? col.cards : null
            contentY: col.scrollTarget * col.px - 4 * col.px
            Behavior on contentY {
                enabled: col.scrolls && !Kit.Tokens.reducedMotion
                NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
            }
            delegate: Item {
                id: scrollSlot
                required property int index
                required property var model
                width: colList.width
                height: 56 * col.px
                CalloutCard {
                    rowIndex: scrollSlot.index
                    itemData: scrollSlot.model.item
                    leftSide: col.leftSide
                    px: col.px
                    focusedControl: col.focusedControl
                    headerFocus: col.headerFocus
                    isKeys: col.isKeys
                    modelSettings: col.modelSettings
                    x: col.reach
                    y: 0
                }
            }
        }
    }
    FocusScope {
        id: root
        objectName: "controllersEclipse"
        anchors.fill: parent
        focus: true
        readonly property real boardHeight: 1200
        readonly property real px: canvas.designHeight / root.boardHeight
        readonly property real reach: 48 * root.px
        readonly property var model: canvas.pad
        readonly property bool isKeys: root.model?.keys === true
        readonly property bool fromCard: root.model?.fromCard === true
        readonly property int focusedControl: canvas.settings.padFocus
        readonly property int headerFocus: root.isKeys ? -1 : canvas.settings.padHeaderFocus
        TapHandler {}
        HelpModel {
            id: helpModel
            screen: canvas.settings.listening
                ? HelpModel.SettingsListening
                : (root.isKeys
                    ? HelpModel.SettingsKeys
                    : (root.fromCard
                        ? HelpModel.SettingsEclipseButtonsCard
                        : HelpModel.SettingsEclipseButtons))
        }
        FocusScope {
            id: navScope
            objectName: "bindingsNavScope"
            anchors.fill: parent
            focus: canvas.inputs && !canvas.settings.listening
            Keys.onLeftPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Left, event.isAutoRepeat)
            Keys.onRightPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Right, event.isAutoRepeat)
            Keys.onUpPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Up, event.isAutoRepeat)
            Keys.onDownPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Down, event.isAutoRepeat)
            Keys.onReturnPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Accept, event.isAutoRepeat)
            Keys.onEnterPressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Accept, event.isAutoRepeat)
            Keys.onEscapePressed: (event) => event.accepted = canvas.settings.bindingPadButton(Kit.Pad.Back, event.isAutoRepeat)
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_PageUp || event.key === Qt.Key_PageDown)
                    event.accepted = canvas.settings.bindingPadButton(event.key === Qt.Key_PageDown ? Kit.Pad.PageDown : Kit.Pad.PageUp, event.isAutoRepeat)
            }
            Connections {
                target: Kit.PadRouter
                enabled: navScope.activeFocus
                function onPressed(button, repeat, jump) { canvas.settings.bindingPadButton(button, repeat) }
            }
            Timer {
                interval: 8
                repeat: true
                running: navScope.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
                onTriggered: Kit.PadRouter.pump()
            }
        }
        FocusScope {
            id: listenScope
            objectName: "bindingsListener"
            anchors.fill: parent
            focus: canvas.inputs && canvas.settings.listening
            Keys.onPressed: (event) => {
                if (canvas.settings.listeningPad) {
                    if (event.key === Qt.Key_Escape) {
                        canvas.settings.back()
                        event.accepted = true
                    }
                } else {
                    event.accepted = canvas.settings.heardKey(event.key, event.modifiers)
                }
            }
            Keys.onReleased: (event) => {
                if (!canvas.settings.listeningPad)
                    event.accepted = !event.isAutoRepeat && canvas.settings.releasedKey(event.key, event.modifiers)
            }
            Connections {
                target: Kit.PadRouter
                enabled: listenScope.activeFocus && !canvas.settings.listeningPad
                function onPressed(button, repeat, jump) { canvas.settings.padPressed(button) }
            }
            Timer {
                interval: 8
                repeat: true
                running: listenScope.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
                onTriggered: Kit.PadRouter.pump()
            }
        }
        // Five-second timeout leaves pad binding unchanged (Settings.qml:318-322).
        Timer {
            id: padTimeout
            objectName: "padListenTimeout"
            interval: 5000
            running: canvas.settings.listeningPad
            onTriggered: canvas.settings.back()
        }
        Kit.Backdrop {
            objectName: "padBackdrop"
            anchors.fill: parent
            ground: Kit.Tokens.surface
            pixelScale: canvas.unit
            fields: [
                Kit.BackdropField {
                    x: (root.width - 800 * root.px) / 2
                    y: 180 * root.px
                    width: 800 * root.px
                    height: 640 * root.px
                    color: ThemeInk.backdropHalo
                    blur: 170 * root.px
                    opacity: 0.28
                }
            ]
        }
        // Top header bar (left 96, right 96, top 36, height 60, center y 66).
        Item {
            id: headerBar
            objectName: "padHeader"
            x: 96 * root.px
            y: 36 * root.px
            width: root.width - 192 * root.px
            height: 60 * root.px
            Shape {
                id: crescent
                objectName: "mark"
                x: 0
                y: 13 * root.px
                readonly property real box: 24
                width: crescent.box
                height: crescent.box
                scale: (34 * root.px) / crescent.box
                transformOrigin: Item.TopLeft
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: Kit.Tokens.coronaColor
                    strokeColor: "transparent"
                    fillRule: ShapePath.OddEvenFill
                    PathSvg { path: Marks.crescent }
                }
            }
            Item {
                objectName: "padCrumbs"
                x: (34 + 28) * root.px
                y: 0
                height: headerBar.height
                width: crumbCtrl.x + crumbCtrl.width
                CardLine {
                    id: crumbSet
                    objectName: "crumbSettings"
                    x: 0
                    y: (headerBar.height - crumbSet.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 24
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                    text: root.model?.crumbSettings ?? ""
                    MouseArea {
                        anchors.fill: parent
                        enabled: !root.fromCard
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: canvas.settings.bindingSettings()
                    }
                }
                CardLine {
                    id: crumbSl
                    objectName: "crumbSlash"
                    x: crumbSet.width + 14 * root.px
                    y: (headerBar.height - crumbSl.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 24
                    weight: Font.Normal
                    color: ThemeInk.dim
                    text: root.model?.crumbSlash ?? ""
                }
                CardLine {
                    id: crumbCtrl
                    objectName: "crumbControllers"
                    x: crumbSl.x + crumbSl.width + 14 * root.px
                    y: (headerBar.height - crumbCtrl.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 24
                    weight: Font.DemiBold
                    color: Kit.Tokens.text
                    text: root.model?.crumbControllers ?? ""
                }
            }
            Item {
                objectName: "padStatus"
                x: 0
                y: 0
                width: headerBar.width
                height: headerBar.height
                CardLine {
                    id: battLine
                    objectName: "battery"
                    visible: canvas.battery >= 0
                    x: clkLine.x - battLine.width - 22 * root.px
                    y: (headerBar.height - battLine.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 22
                    weight: Font.Normal
                    color: Kit.Tokens.text
                    text: canvas.battery >= 0 ? Words.percent(canvas.battery) : ""
                }
                CardLine {
                    id: clkLine
                    objectName: "clock"
                    x: headerBar.width - clkLine.width
                    y: (headerBar.height - clkLine.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 22
                    weight: Font.DemiBold
                    color: Kit.Tokens.text
                    text: canvas.clockText
                }
            }
        }
        // Five active sub-navigation tabs (top 116, left 96, gap 36; #76 Test omitted).
        Row {
            id: nav2
            objectName: "padNav"
            x: 96 * root.px
            y: 116 * root.px
            height: 42 * root.px
            spacing: 36 * root.px
            Repeater {
                model: root.model?.navTabs ?? null
                Item {
                    id: tabItem
                    required property int index
                    required property string modelData
                    readonly property bool active: tabItem.index === (root.model?.navActive ?? 1)
                    objectName: "navTab." + tabItem.index
                    width: tabLabel.width
                    height: 42 * root.px
                    opacity: root.fromCard && !tabItem.active ? 0.45 : 1
                    CardLine {
                        id: tabLabel
                        objectName: "navTabWords." + tabItem.index
                        y: 0
                        unit: root.px
                        boardLine: true
                        size: 23
                        weight: tabItem.active ? Font.DemiBold : Font.Normal
                        color: tabItem.active ? Kit.Tokens.textStrong : Kit.Tokens.muted
                        text: tabItem.modelData
                    }
                    Rectangle {
                        objectName: "navTabLine." + tabItem.index
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: 3 * root.px
                        color: tabItem.active ? Kit.Tokens.accent : "transparent"
                    }
                    MouseArea {
                        anchors.fill: parent
                        enabled: !root.fromCard
                        cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        onClicked: canvas.settings.bindingNav(tabItem.index)
                    }
                }
            }
        }
        // Control strip under sub-navigation (top 186, height 48).
        Item {
            id: stripRow
            objectName: "padStrip"
            x: 96 * root.px
            y: 186 * root.px
            width: root.width - 192 * root.px
            height: 48 * root.px
            Row {
                visible: !root.isKeys
                anchors.left: parent.left
                height: parent.height
                spacing: 12 * root.px
                Repeater {
                    model: root.model?.players ?? null
                    Rectangle {
                        id: playerPill
                        required property int index
                        required property var modelData
                        readonly property bool pillActive: playerPill.modelData.active === true
                        readonly property bool pillFocused: root.headerFocus === playerPill.index
                        objectName: "playerPill." + playerPill.index
                        height: 48 * root.px
                        width: playerWords.width + 40 * root.px
                        radius: 24 * root.px
                        color: playerPill.pillActive ? Kit.Tokens.accent : "transparent"
                        border.width: playerPill.pillActive ? 0 : 2 * root.px
                        border.color: ThemeInk.pillBorder
                        Kit.FocusRing {
                            objectName: "playerPillRing." + playerPill.index
                            visible: playerPill.pillFocused
                            anchors.fill: parent
                            radius: 24 * root.px
                            gap: 3 * root.px
                            thickness: 3 * root.px
                            bandRadius: 30 * root.px
                            glowColor: Kit.Tokens.glowColor
                            glowBlur: 34 * root.px
                            glowSpread: 8 * root.px
                            pixelScale: canvas.unit
                        }
                        CardLine {
                            id: playerWords
                            objectName: "playerWords." + playerPill.index
                            x: (playerPill.width - playerWords.width) / 2
                            y: (playerPill.height - playerWords.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 20
                            weight: playerPill.pillActive ? Font.DemiBold : Font.Normal
                            color: playerPill.pillActive ? Kit.Tokens.overAccent : Kit.Tokens.text
                            text: playerPill.modelData.label ?? ""
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.bindingPlayer(playerPill.modelData.player ?? (playerPill.index + 1))
                        }
                    }
                }
            }
            Item {
                visible: !root.isKeys
                x: 0
                y: 0
                width: stripRow.width
                height: stripRow.height
                CardLine {
                    id: forWordsItem
                    objectName: "forWords"
                    x: consoleDrop.x - forWordsItem.width - 12 * root.px
                    y: (stripRow.height - forWordsItem.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 20
                    weight: Font.Normal
                    color: ThemeInk.body
                    text: root.model?.forWord ?? ""
                }
                Rectangle {
                    id: consoleDrop
                    readonly property int myIndex: root.model?.players ? root.model?.players.length : 0
                    readonly property bool pillFocused: root.headerFocus === consoleDrop.myIndex
                    objectName: "consolePill"
                    x: profilePreItem.x - consoleDrop.width - 12 * root.px
                    y: 0
                    height: 48 * root.px
                    width: consoleDropWords.width + 10 * root.px + consoleChev.width + 40 * root.px
                    radius: 24 * root.px
                    color: "transparent"
                    border.width: 2 * root.px
                    border.color: Kit.Tokens.accent
                    Kit.FocusRing {
                        objectName: "consolePillRing"
                        visible: consoleDrop.pillFocused
                        anchors.fill: parent
                        radius: 24 * root.px
                        gap: 3 * root.px
                        thickness: 3 * root.px
                        bandRadius: 30 * root.px
                        glowColor: Kit.Tokens.glowColor
                        glowBlur: 34 * root.px
                        glowSpread: 8 * root.px
                        pixelScale: canvas.unit
                    }
                    CardLine {
                        id: consoleDropWords
                        objectName: "consolePillWords"
                        x: 20 * root.px
                        y: (consoleDrop.height - consoleDropWords.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 20
                        weight: Font.Normal
                        color: Kit.Tokens.text
                        text: root.model?.console ?? ""
                    }
                    Shape {
                        id: consoleChev
                        objectName: "consoleChevron"
                        x: consoleDropWords.x + consoleDropWords.width + 10 * root.px
                        y: (consoleDrop.height - consoleChev.height) / 2
                        width: 18 * root.px
                        height: 18 * root.px
                        ShapePath {
                            fillColor: "transparent"
                            strokeColor: Kit.Tokens.text
                            strokeWidth: 2.4 * root.px * (18 / 24)
                            capStyle: ShapePath.RoundCap
                            joinStyle: ShapePath.RoundJoin
                            PathMove { x: 4.5 * root.px; y: 6.75 * root.px }
                            PathLine { x: 9 * root.px; y: 11.25 * root.px }
                            PathLine { x: 13.5 * root.px; y: 6.75 * root.px }
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: canvas.settings.shoulder(1)
                    }
                }
                CardLine {
                    id: profilePreItem
                    objectName: "profilePre"
                    x: profileNameItem.x - profilePreItem.width - 12 * root.px
                    y: (stripRow.height - profilePreItem.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 20
                    weight: Font.Normal
                    color: ThemeInk.body
                    text: root.model?.profilePre ?? ""
                }
                CardLine {
                    id: profileNameItem
                    objectName: "profileName"
                    x: stripRow.width - profileNameItem.width
                    y: (stripRow.height - profileNameItem.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 20
                    weight: Font.DemiBold
                    color: Kit.Tokens.text
                    text: root.model?.profile ?? ""
                }
            }
            Item {
                visible: root.isKeys
                anchors.fill: parent
                Row {
                    id: keysPills
                    anchors.left: parent.left
                    height: parent.height
                    spacing: 12 * root.px
                    Rectangle {
                        id: keyboardPill
                        objectName: "keyboardPill"
                        height: 48 * root.px
                        width: keyboardWords.width + 40 * root.px
                        radius: 24 * root.px
                        color: Kit.Tokens.accent
                        border.width: 0
                        CardLine {
                            id: keyboardWords
                            objectName: "keyboardPillWords"
                            x: (keyboardPill.width - keyboardWords.width) / 2
                            y: (keyboardPill.height - keyboardWords.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 20
                            weight: Font.DemiBold
                            color: Kit.Tokens.overAccent
                            text: root.model?.keyboardLabel ?? ""
                        }
                    }
                    Repeater {
                        model: root.model?.consolePills ?? null
                        Rectangle {
                            id: keyConsolePill
                            required property int index
                            required property var modelData
                            readonly property real pillWidth: keyConsoleWords.width + 40 * root.px
                            readonly property bool fits: x + pillWidth <= stripRow.width - keysCaption.width - 28 * root.px
                            objectName: "keyConsolePill." + keyConsolePill.index
                            height: 48 * root.px
                            width: fits ? pillWidth : 0
                            radius: 24 * root.px
                            color: "transparent"
                            border.width: 2 * root.px
                            border.color: ThemeInk.pillBorder
                            visible: fits
                            enabled: fits
                            CardLine {
                                id: keyConsoleWords
                                objectName: "keyConsoleWords." + keyConsolePill.index
                                x: (keyConsolePill.width - keyConsoleWords.width) / 2
                                y: (keyConsolePill.height - keyConsoleWords.height) / 2
                                unit: root.px
                                boardLine: true
                                size: 20
                                weight: Font.Normal
                                color: Kit.Tokens.text
                                text: keyConsolePill.modelData.name ?? ""
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: canvas.settings.bindingConsole(keyConsolePill.modelData.key ?? "")
                            }
                        }
                    }
                }
                CardLine {
                    id: keysCaption
                    objectName: "keysCaption"
                    x: keysPills.width + 28 * root.px
                    y: (stripRow.height - keysCaption.height) / 2
                    unit: root.px
                    boardLine: true
                    size: 20
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                    text: root.model?.caption ?? ""
                }
            }
        }
        // Center diagram box (left 440, top 270, width 1040, height 520 at 1920x1200).
        Item {
            id: diagram
            objectName: "padDiagram"
            x: (96 + 344) * root.px
            y: 270 * root.px
            width: root.width - 2 * (96 + 344) * root.px
            height: 520 * root.px
            readonly property real padOffsetX: (diagram.width / root.px - 1040) / 2
            readonly property real leftScrollPx: leftFlick.scrollY / root.px
            readonly property real rightScrollPx: rightFlick.scrollY / root.px
            LeaderPass {
                model: padLeftModel
                leftSide: true
                focusPass: false
                px: root.px
                offsetX: diagram.padOffsetX
                scrollPx: diagram.leftScrollPx
                diagramWidth: diagram.width
                focusedControl: root.focusedControl
                headerFocus: root.headerFocus
                isKeys: root.isKeys
            }
            LeaderPass {
                model: padRightModel
                leftSide: false
                focusPass: false
                px: root.px
                offsetX: diagram.padOffsetX
                scrollPx: diagram.rightScrollPx
                diagramWidth: diagram.width
                focusedControl: root.focusedControl
                headerFocus: root.headerFocus
                isKeys: root.isKeys
            }
            LeaderPass {
                model: padLeftModel
                leftSide: true
                focusPass: true
                px: root.px
                offsetX: diagram.padOffsetX
                scrollPx: diagram.leftScrollPx
                diagramWidth: diagram.width
                focusedControl: root.focusedControl
                headerFocus: root.headerFocus
                isKeys: root.isKeys
            }
            LeaderPass {
                model: padRightModel
                leftSide: false
                focusPass: true
                px: root.px
                offsetX: diagram.padOffsetX
                scrollPx: diagram.rightScrollPx
                diagramWidth: diagram.width
                focusedControl: root.focusedControl
                headerFocus: root.headerFocus
                isKeys: root.isKeys
            }
            Image {
                id: padPicture
                objectName: "padPicture"
                x: ((root.model?.padX ?? 200) + diagram.padOffsetX) * root.px
                y: (root.model?.padY ?? 40) * root.px
                width: (root.model?.padW ?? 640) * root.px
                height: (root.model?.padH ?? 440) * root.px
                source: canvas.settings.padSvg
                fillMode: Image.PreserveAspectFit
                sourceSize: drawnPixels.pixels
                Kit.DrawnPixels {
                    id: drawnPixels
                    anchors.fill: parent
                }
            }
        }
        CalloutColumn {
            id: leftFlick
            objectName: "leftCallouts"
            leftSide: true
            px: root.px
            reach: root.reach
            scrolls: root.model?.leftScroll === true
            scrollTarget: canvas.settings.padLeftScrollY
            cards: padLeftModel
            focusedControl: root.focusedControl
            headerFocus: root.headerFocus
            isKeys: root.isKeys
            modelSettings: canvas.settings
            x: 96 * root.px - root.reach
            y: 270 * root.px - (leftFlick.scrolls ? 0 : root.reach)
            width: 344 * root.px + 2 * root.reach
            height: 520 * root.px + (leftFlick.scrolls ? 0 : 2 * root.reach)
        }
        CalloutColumn {
            id: rightFlick
            objectName: "rightCallouts"
            leftSide: false
            px: root.px
            reach: root.reach
            scrolls: root.model?.rightScroll === true
            scrollTarget: canvas.settings.padRightScrollY
            cards: padRightModel
            focusedControl: root.focusedControl
            headerFocus: root.headerFocus
            isKeys: root.isKeys
            modelSettings: canvas.settings
            x: root.width - 96 * root.px - 344 * root.px - root.reach
            y: 270 * root.px - (rightFlick.scrolls ? 0 : root.reach)
            width: 344 * root.px + 2 * root.reach
            height: 520 * root.px + (rightFlick.scrolls ? 0 : 2 * root.reach)
        }
        // Detail card (top 840, width 1040 centered, padding 22 28, radius 22).
        Item {
            id: detailBox
            objectName: "detailBox"
            x: (root.width - detailBox.width) / 2
            y: 840 * root.px
            width: Math.min(1040 * root.px, root.width - 192 * root.px)
            height: root.isKeys ? keysDetailCol.height + 44 * root.px : Math.max(100 * root.px, buttonsDetailCol.height + 44 * root.px)
            Kit.Glow {
                objectName: "detailGlow"
                anchors.fill: parent
                radius: 22 * root.px
                color: ThemeInk.glowWeak
                blur: 40 * root.px
                spread: 6 * root.px
                pixelScale: canvas.unit
            }
            Rectangle {
                objectName: "detailRim"
                x: -2 * root.px
                y: -2 * root.px
                width: parent.width + 4 * root.px
                height: parent.height + 4 * root.px
                radius: 24 * root.px
                color: "transparent"
                border.width: 2 * root.px
                border.color: ThemeInk.coronaSoft
            }
            Rectangle {
                objectName: "detailCard"
                anchors.fill: parent
                radius: 22 * root.px
                color: Kit.Tokens.surfaceRaised
            }
            Column {
                id: buttonsDetailCol
                visible: !root.isKeys
                x: 28 * root.px
                y: 22 * root.px
                width: parent.width - 56 * root.px - (counterLine.visible ? counterLine.width + 24 * root.px : 0)
                spacing: 4 * root.px
                CardLine {
                    objectName: "detailHeading"
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    size: 24
                    weight: Font.DemiBold
                    color: Kit.Tokens.text
                    elide: Text.ElideRight
                    text: root.model?.detailHeading ?? ""
                }
                CardLine {
                    objectName: "detailSubline"
                    visible: (root.model?.detailSubline ?? "").length > 0
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    size: 18
                    weight: Font.Normal
                    color: ThemeInk.body
                    wrapMode: Text.WordWrap
                    text: root.model?.detailSubline ?? ""
                }
            }
            CardLine {
                id: counterLine
                objectName: "detailCounter"
                visible: !root.isKeys && (root.model?.detailCounter ?? "").length > 0
                x: detailBox.width - counterLine.width - 28 * root.px
                y: (detailBox.height - counterLine.height) / 2
                unit: root.px
                boardLine: true
                size: 18
                weight: Font.Normal
                color: Kit.Tokens.muted
                text: root.model?.detailCounter ?? ""
            }
            Column {
                id: keysDetailCol
                visible: root.isKeys
                x: 28 * root.px
                y: 22 * root.px
                width: parent.width - 56 * root.px
                spacing: 6 * root.px
                CardLine {
                    objectName: "keysHeading"
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    size: 26
                    weight: Font.DemiBold
                    color: Kit.Tokens.text
                    text: root.model?.detailHeading ?? ""
                }
                CardLine {
                    objectName: "keysLine1"
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    boardLineHeight: 29.4
                    size: 21
                    weight: Font.Normal
                    color: ThemeInk.body
                    text: root.model?.detailLine1 ?? ""
                }
                CardLine {
                    objectName: "keysLine2"
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    boardLineHeight: 26.6
                    size: 19
                    weight: Font.Normal
                    color: canvas.settings.listening ? Kit.Tokens.accent : ThemeInk.body
                    text: root.model?.detailLine2 ?? ""
                }
                CardLine {
                    objectName: "keysMuted"
                    visible: (root.model?.detailMuted ?? "").length > 0
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    boardLineHeight: 26.6
                    size: 19
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                    wrapMode: Text.WordWrap
                    text: root.model?.detailMuted ?? ""
                }
                CardLine {
                    objectName: "keysConflict"
                    visible: (root.model?.detailConflict ?? "").length > 0
                    width: parent.width
                    unit: root.px
                    boardLine: true
                    boardLineHeight: 26.6
                    size: 19
                    weight: Font.Normal
                    color: ThemeInk.rose
                    wrapMode: Text.WordWrap
                    text: root.model?.detailConflict ?? ""
                }
            }
        }
        // Bottom whole-pad summary cards (#12 only: top 980, 4 columns, gap 16, height 84, radius 16).
        Row {
            id: wholeRow
            objectName: "wholeStrip"
            visible: !root.isKeys
            x: 96 * root.px
            y: 980 * root.px
            width: root.width - 192 * root.px
            height: 84 * root.px
            spacing: 16 * root.px
            Repeater {
                model: root.model?.whole ?? null
                Rectangle {
                    id: wholeCard
                    required property int index
                    required property var modelData
                    objectName: "wholeCard." + wholeCard.index
                    width: (wholeRow.width - 48 * root.px) / 4
                    height: 84 * root.px
                    radius: 16 * root.px
                    color: ThemeInk.rowSurface
                    border.width: 1 * root.px
                    border.color: ThemeInk.hairline
                    CardLine {
                        id: wholeNameLine
                        objectName: "wholeCardName." + wholeCard.index
                        x: 20 * root.px
                        y: (wholeCard.height - wholeNameLine.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 19
                        weight: Font.Normal
                        color: Kit.Tokens.text
                        text: wholeCard.modelData.name ?? ""
                    }
                    CardLine {
                        id: wholeValueLine
                        objectName: "wholeCardValue." + wholeCard.index
                        x: wholeCard.width - wholeValueLine.width - 20 * root.px
                        y: (wholeCard.height - wholeValueLine.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 19
                        weight: Font.DemiBold
                        color: Kit.Tokens.accent
                        text: wholeCard.modelData.value ?? ""
                    }
                }
            }
        }
        // Footer help bar (bottom 30, middle at 1151, gap 30, glyph gap 10, 20 px words).
        Kit.HelpBar {
            id: helpBar
            objectName: "help"
            x: 96 * root.px
            y: root.height - 49 * root.px - helpBar.height / 2
            model: helpModel
            family: canvas.padFamily
            fitted: true
            ruled: false
            color: "transparent"
            labelColor: ThemeInk.body
            iconColor: Kit.Tokens.muted
            leadStyle: Kit.GlyphStyles.Filled
            leadColor: Kit.Tokens.accent
            entrySpacing: 30 * root.px
            glyphGap: 10 * root.px
            glyphSize: 34 * root.px
            snapped: false
            onPressed: (button) => canvas.settings.bindingPadButton(button, false)
            label: Component {
                CardLine {
                    id: helpWordLine
                    objectName: "helpWords-" + helpWordLine.text
                    unit: root.px
                    boardLine: true
                    size: 20
                }
            }
            glyph: Component {
                CardGlyph {
                    unit: root.px
                    side: 34
                    keyPairs: true
                    ink: ThemeInk.body
                    ring: Kit.Tokens.muted
                    leadFill: Kit.Tokens.accent
                    leadInk: Kit.Tokens.overAccent
                }
            }
        }
    }
    }
}
