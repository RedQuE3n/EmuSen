pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Eclipse Screen filter / Shaders browser (#20, EclipseShaders.dc.html) with live before/after Serenity preview.
FocusScope {
    id: outer
    required property SettingsModel settings
    required property SessionModel session
    property double hostFeed: 0
    property int padFamily: Kit.Families.Xbox
    property int battery: -1
    property string clockText: ""
    property bool inputs: true
    property real rightStickX: 0
    focus: true
    Timer {
        interval: 0
        repeat: false
        running: outer.visible && outer.settings.shadersShown
        onTriggered: outer.settings.shadersInit(outer.hostFeed, outer.session.pausedFrameRgba())
    }
    Component.onDestruction: outer.settings.shadersRelease()
    onHostFeedChanged: if (outer.hostFeed !== 0 && outer.settings.shadersShown) outer.settings.shadersInit(outer.hostFeed, outer.session.pausedFrameRgba())
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "shadersEclipseCanvas"
        readonly property SettingsModel settings: outer.settings
        readonly property SessionModel session: outer.session
        readonly property int padFamily: outer.padFamily
        readonly property int battery: outer.battery
        readonly property string clockText: outer.clockText
        readonly property bool inputs: outer.inputs
        readonly property real rightStickX: outer.rightStickX
        readonly property var board: outer.settings.shadersBoard.length > 0 ? JSON.parse(outer.settings.shadersBoard) : ({})
        RowsModel {
            id: helpRows
            rows: outer.settings.shadersHelp
        }
        Timer {
            id: previewDebounce
            objectName: "shadersPreviewTimer"
            interval: 120
            repeat: false
            onTriggered: canvas.settings.commitShaderPreview()
        }
        Connections {
            target: canvas.settings
            function onShadersPreviewDebounce() { previewDebounce.restart() }
        }
        FocusScope {
            id: root
            objectName: "shadersEclipse"
            anchors.fill: parent
            focus: true
            readonly property real boardHeight: 1200
            readonly property real px: canvas.designHeight / root.boardHeight
            readonly property real reach: 48 * root.px
            readonly property var model: canvas.board
            readonly property var labels: root.model.labels ?? ({})
            readonly property var sources: root.model.sources ?? []
            readonly property var categories: root.model.categories ?? []
            readonly property var pack: root.model.pack ?? ({})
            readonly property var search: root.model.search ?? ({})
            readonly property var presets: root.model.presets ?? []
            readonly property var scopes: root.model.scopes ?? []
            readonly property var preview: root.model.preview ?? ({})
            readonly property var sliders: root.model.sliders ?? []
            readonly property var allSettings: root.model.allSettings ?? ({})
            readonly property real splitRatio: root.preview.split ?? 0.5
            readonly property real splitPx: root.splitRatio * 820 * root.px
            readonly property bool hasFiltered: root.preview.hasFiltered === true
            readonly property bool confirmOpen: root.model.confirmOpen === true
            readonly property int focusedPresetIdx: root.model.focusedPreset ?? 0
            TapHandler {}
            FocusScope {
                id: navScope
                objectName: "shadersNavScope"
                anchors.fill: parent
                focus: canvas.inputs && !root.confirmOpen && !canvas.settings.typing
                Timer {
                    interval: 0
                    repeat: false
                    running: navScope.focus && !navScope.activeFocus && !confirmFrame.controller.sheetOpen && canvas.settings.shadersShown
                    onTriggered: navScope.forceActiveFocus()
                }
                Keys.onUpPressed: (event) => event.accepted = canvas.settings.shadersKeyNav(0)
                Keys.onDownPressed: (event) => event.accepted = canvas.settings.shadersKeyNav(1)
                Keys.onLeftPressed: (event) => event.accepted = canvas.settings.shadersKeyNav(2)
                Keys.onRightPressed: (event) => event.accepted = canvas.settings.shadersKeyNav(3)
                Keys.onReturnPressed: (event) => event.accepted = !event.isAutoRepeat && canvas.settings.shadersKeyNav(4)
                Keys.onEnterPressed: (event) => event.accepted = !event.isAutoRepeat && canvas.settings.shadersKeyNav(4)
                Keys.onEscapePressed: (event) => event.accepted = !event.isAutoRepeat && canvas.settings.shadersKeyNav(5)
                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_X && !event.isAutoRepeat) {
                        canvas.settings.shadersHoldCompare(true)
                        event.accepted = true
                    } else if (event.key === Qt.Key_Y && !event.isAutoRepeat) {
                        event.accepted = canvas.settings.shadersKeyNav(6)
                    } else if (event.key === Qt.Key_PageUp) {
                        event.accepted = canvas.settings.shadersKeyNav(7)
                    } else if (event.key === Qt.Key_PageDown) {
                        event.accepted = canvas.settings.shadersKeyNav(8)
                    } else if (event.key === Qt.Key_BracketLeft && !event.isAutoRepeat) {
                        event.accepted = canvas.settings.shadersKeyNav(10)
                    } else if (event.key === Qt.Key_BracketRight && !event.isAutoRepeat) {
                        event.accepted = canvas.settings.shadersKeyNav(11)
                    }
                }
                Keys.onReleased: (event) => {
                    if (event.key === Qt.Key_X && !event.isAutoRepeat) {
                        canvas.settings.shadersHoldCompare(false)
                        event.accepted = true
                    }
                }
                Connections {
                    target: Kit.PadRouter
                    enabled: navScope.activeFocus
                    function onPressed(button, repeat, jump) { canvas.settings.shadersPadButton(button, true, repeat) }
                    function onReleased(button) { canvas.settings.shadersPadButton(button, false, false) }
                }
                Timer {
                    interval: 8
                    repeat: true
                    running: navScope.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
                    onTriggered: Kit.PadRouter.pump()
                }
                Timer {
                    id: splitScrubTimer
                    objectName: "shadersSplitScrubTimer"
                    interval: 16
                    repeat: true
                    running: navScope.activeFocus && Math.abs(canvas.rightStickX) > 0.01
                    onTriggered: canvas.settings.shadersScrubSplit(canvas.rightStickX * 0.018)
                }
            }
            Kit.Backdrop {
                objectName: "shadersBackdrop"
                anchors.fill: parent
                ground: "#07080F"
                pixelScale: canvas.unit
            }
            // Top header bar (left 96, top 40, height 64).
            Item {
                id: headerBar
                objectName: "shadersHeader"
                x: 96 * root.px
                y: 40 * root.px
                width: root.width - 192 * root.px
                height: 64 * root.px
                Shape {
                    id: crescent
                    objectName: "shadersMark"
                    x: 0
                    y: 15 * root.px
                    readonly property real box: 34
                    width: crescent.box
                    height: crescent.box
                    scale: (34 * root.px) / crescent.box
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: "#F6E7D8"
                        strokeColor: "transparent"
                        fillRule: ShapePath.OddEvenFill
                        PathSvg { path: Marks.menuCrescent }
                    }
                }
                Item {
                    id: crumbsRow
                    objectName: "shadersCrumbs"
                    x: (34 + 28) * root.px
                    y: 0
                    width: crumbScreenFilter.x + crumbScreenFilter.width
                    height: headerBar.height
                    CardLine {
                        id: crumbSettings
                        objectName: "crumbSettings"
                        x: 0
                        y: (headerBar.height - crumbSettings.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 24
                        weight: Font.Normal
                        color: "#B9BED0"
                        text: root.labels.settings ?? ""
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersBack()
                        }
                    }
                    CardLine {
                        id: crumbSlash0
                        objectName: "crumbSlash.0"
                        x: crumbSettings.x + crumbSettings.width + 14 * root.px
                        y: (headerBar.height - crumbSlash0.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 24
                        weight: Font.Normal
                        color: "#8E94AB"
                        text: root.labels.slash ?? ""
                    }
                    CardLine {
                        id: crumbPicture
                        objectName: "crumbPicture"
                        x: crumbSlash0.x + crumbSlash0.width + 14 * root.px
                        y: (headerBar.height - crumbPicture.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 24
                        weight: Font.Normal
                        color: "#B9BED0"
                        text: root.labels.picture ?? ""
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersBack()
                        }
                    }
                    CardLine {
                        id: crumbSlash1
                        objectName: "crumbSlash.1"
                        x: crumbPicture.x + crumbPicture.width + 14 * root.px
                        y: (headerBar.height - crumbSlash1.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 24
                        weight: Font.Normal
                        color: "#8E94AB"
                        text: root.labels.slash ?? ""
                    }
                    CardLine {
                        id: crumbScreenFilter
                        objectName: "crumbScreenFilter"
                        x: crumbSlash1.x + crumbSlash1.width + 14 * root.px
                        y: (headerBar.height - crumbScreenFilter.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 24
                        weight: Font.DemiBold
                        color: "#F4F1EC"
                        text: root.labels.filter ?? ""
                    }
                }
                Item {
                    objectName: "shadersStatus"
                    anchors.fill: parent
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
                        color: "#F4F1EC"
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
                        color: "#F4F1EC"
                        text: canvas.clockText
                    }
                }
            }
            // Left column: source switch (y 130, w 400, h 64).
            Rectangle {
                id: sourceSwitch
                objectName: "sourceSwitch"
                x: 96 * root.px
                y: 130 * root.px
                width: 400 * root.px
                height: 64 * root.px
                radius: 32 * root.px
                color: "#11131C"
                border.width: 1 * root.px
                border.color: "#262A3A"
                Repeater {
                    model: root.sources
                    Rectangle {
                        id: srcTab
                        required property int index
                        required property var modelData
                        readonly property bool isActive: srcTab.modelData.active === true
                        readonly property bool isFocused: srcTab.modelData.focused === true
                        objectName: srcTab.modelData.pillName ?? ""
                        x: (6 + srcTab.index * 194) * root.px
                        y: 6 * root.px
                        width: 194 * root.px
                        height: 52 * root.px
                        radius: 26 * root.px
                        color: srcTab.isActive ? "#F6E7D8" : "transparent"
                        Kit.FocusRing {
                            objectName: srcTab.modelData.ringName ?? ""
                            visible: srcTab.isFocused
                            anchors.fill: parent
                            radius: 26 * root.px
                            gap: 3 * root.px
                            thickness: 3 * root.px
                            bandRadius: 32 * root.px
                            glowColor: "#66F0B4C4"
                            glowBlur: 34 * root.px
                            glowSpread: 8 * root.px
                            pixelScale: canvas.unit
                        }
                        CardLine {
                            id: srcWords
                            objectName: srcTab.modelData.textName ?? ""
                            x: (srcTab.width - srcWords.width) / 2
                            y: (srcTab.height - srcWords.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 19
                            weight: srcTab.isActive ? Font.DemiBold : Font.Normal
                            color: srcTab.isActive ? "#07080F" : "#D8DBE6"
                            text: srcTab.modelData.text ?? ""
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersSource(srcTab.modelData.index ?? srcTab.index)
                        }
                    }
                }
            }
            // Left column: categories (x 96, y 208, w 400, 8 x 60 + 4 gap).
            Column {
                id: catCol
                objectName: "categoryList"
                x: 96 * root.px
                y: 208 * root.px
                width: 400 * root.px
                spacing: 4 * root.px
                Repeater {
                    model: root.categories
                    Rectangle {
                        id: catRow
                        required property int index
                        required property var modelData
                        readonly property bool isActive: catRow.modelData.active === true
                        readonly property bool isFocused: catRow.modelData.focused === true
                        objectName: catRow.modelData.rowName ?? ""
                        width: 400 * root.px
                        height: 60 * root.px
                        radius: 14 * root.px
                        color: catRow.isActive ? "#F6E7D8" : (catRow.isFocused ? "#151826" : "transparent")
                        Kit.FocusRing {
                            objectName: catRow.modelData.ringName ?? ""
                            visible: catRow.isFocused
                            anchors.fill: parent
                            radius: 14 * root.px
                            gap: 3 * root.px
                            thickness: 3 * root.px
                            bandRadius: 20 * root.px
                            glowColor: "#66F0B4C4"
                            glowBlur: 34 * root.px
                            glowSpread: 8 * root.px
                            pixelScale: canvas.unit
                        }
                        CardLine {
                            id: catName
                            objectName: catRow.modelData.nameName ?? ""
                            x: 18 * root.px
                            y: (catRow.height - catName.height) / 2
                            width: catRow.width - 54 * root.px - catCount.width
                            unit: root.px
                            boardLine: true
                            size: 21
                            weight: catRow.isActive ? Font.DemiBold : Font.Normal
                            color: catRow.isActive ? "#07080F" : "#D8DBE6"
                            elide: Text.ElideRight
                            text: catRow.modelData.name ?? ""
                        }
                        CardLine {
                            id: catCount
                            objectName: catRow.modelData.countName ?? ""
                            x: catRow.width - 18 * root.px - catCount.width
                            y: (catRow.height - catCount.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 17
                            weight: Font.Normal
                            color: catRow.isActive ? "#07080F" : "#D8DBE6"
                            text: catRow.modelData.count ?? ""
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersCategory(catRow.modelData.index ?? catRow.index)
                        }
                    }
                }
            }
            // Left column: local shader pack card (x 96, y 736, w 400).
            Rectangle {
                id: packCard
                objectName: "packCard"
                x: 96 * root.px
                y: 736 * root.px
                width: 400 * root.px
                height: packCol.height + 40 * root.px
                radius: 18 * root.px
                color: "#11131C"
                border.width: 1 * root.px
                border.color: "#262A3A"
                Column {
                    id: packCol
                    x: 20 * root.px
                    y: 20 * root.px
                    width: packCard.width - 40 * root.px
                    spacing: 12 * root.px
                    CardLine {
                        objectName: "packTitle"
                        width: parent.width
                        unit: root.px
                        boardLine: true
                        size: 20
                        weight: Font.DemiBold
                        color: "#F4F1EC"
                        text: root.labels.packTitle ?? ""
                    }
                    CardLine {
                        objectName: "packStatus"
                        width: parent.width
                        unit: root.px
                        boardLine: true
                        boardLineHeight: 24.65
                        size: 17
                        weight: Font.Normal
                        color: "#C9CDDB"
                        wrapMode: Text.WordWrap
                        text: root.pack.status ?? ""
                    }
                    Rectangle {
                        id: removeBtn
                        objectName: "removePackButton"
                        visible: root.pack.installed === true
                        width: removeText.width + 32 * root.px
                        height: 46 * root.px
                        radius: 23 * root.px
                        color: "transparent"
                        border.width: 2 * root.px
                        border.color: removeBtn.isFocused ? "#F6E7D8" : "#4A5068"
                        readonly property bool isFocused: root.pack.removeFocused === true
                        Kit.FocusRing {
                            objectName: "removePackRing"
                            visible: removeBtn.isFocused
                            anchors.fill: parent
                            radius: 23 * root.px
                            gap: 3 * root.px
                            thickness: 3 * root.px
                            bandRadius: 29 * root.px
                            glowColor: "#66F0B4C4"
                            glowBlur: 34 * root.px
                            glowSpread: 8 * root.px
                            pixelScale: canvas.unit
                        }
                        CardLine {
                            id: removeText
                            objectName: "removePackLabel"
                            x: 16 * root.px
                            y: (removeBtn.height - removeText.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 17
                            weight: Font.Normal
                            color: "#D8DBE6"
                            text: root.labels.removePack ?? ""
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersAskRemove()
                        }
                    }
                }
            }
            // Middle column: search pill (x 536, y 130, w 430, h 56).
            Rectangle {
                id: searchPill
                objectName: "searchBar"
                x: 536 * root.px
                y: 130 * root.px
                width: 430 * root.px
                height: 56 * root.px
                radius: 28 * root.px
                color: "#11131C"
                border.width: 1 * root.px
                border.color: "#2A2E40"
                readonly property bool isFocused: root.search.focused === true
                Kit.FocusRing {
                    objectName: "searchRing"
                    visible: searchPill.isFocused
                    anchors.fill: parent
                    radius: 28 * root.px
                    gap: 3 * root.px
                    thickness: 3 * root.px
                    bandRadius: 34 * root.px
                    glowColor: "#66F0B4C4"
                    glowBlur: 34 * root.px
                    glowSpread: 8 * root.px
                    pixelScale: canvas.unit
                }
                Shape {
                    id: searchIcon
                    x: 20 * root.px
                    y: 18 * root.px
                    readonly property real box: 24
                    width: searchIcon.box
                    height: searchIcon.box
                    scale: (20 * root.px) / searchIcon.box
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: "transparent"
                        strokeColor: "#B9BED0"
                        strokeWidth: 2
                        capStyle: ShapePath.RoundCap
                        PathAngleArc { centerX: 11; centerY: 11; radiusX: 7; radiusY: 7; startAngle: 0; sweepAngle: 360 }
                        PathMove { x: 16.5; y: 16.5 }
                        PathLine { x: 21; y: 21 }
                    }
                }
                CardLine {
                    id: searchWord
                    objectName: "searchText"
                    x: 52 * root.px
                    y: (searchPill.height - searchWord.height) / 2
                    width: searchPill.width - 72 * root.px
                    unit: root.px
                    boardLine: true
                    size: 19
                    weight: Font.Normal
                    color: (root.search.hasText === true) ? "#F4F1EC" : "#B9BED0"
                    elide: Text.ElideRight
                    text: (root.search.hasText === true) ? (root.search.text ?? "") : (root.search.placeholder ?? "")
                }
                MouseArea {
                    anchors.fill: parent
                    cursorShape: Qt.PointingHandCursor
                    onClicked: canvas.settings.shadersSearch()
                }
            }
            // Middle column: preset list (x 536, y 202, w 430, row h 88, gap 8).
            Flickable {
                id: presetFlick
                objectName: "presetList"
                x: 536 * root.px - root.reach
                y: 202 * root.px - root.reach
                width: 430 * root.px + 2 * root.reach
                height: 860 * root.px + 2 * root.reach
                clip: true
                interactive: presetCol.height + 2 * root.reach > presetFlick.height
                contentWidth: presetFlick.width
                contentHeight: Math.max(presetFlick.height, presetCol.height + 2 * root.reach)
                contentY: Math.max(0, Math.min(Math.max(0, presetFlick.contentHeight - presetFlick.height), (root.focusedPresetIdx - 8) * 96 * root.px))
                Behavior on contentY {
                    enabled: !Kit.Tokens.reducedMotion
                    NumberAnimation { duration: 120; easing.type: Easing.OutCubic }
                }
                Column {
                    id: presetCol
                    x: root.reach
                    y: root.reach
                    width: 430 * root.px
                    spacing: 8 * root.px
                    Repeater {
                        model: root.presets
                        Rectangle {
                            id: presetRow
                            required property int index
                            required property var modelData
                            readonly property bool isFocused: presetRow.modelData.focused === true
                            readonly property bool isSelected: presetRow.modelData.selected === true
                            readonly property bool isFav: presetRow.modelData.favourite === true
                            readonly property bool drawScan: presetRow.modelData.scanlines === true
                            objectName: presetRow.modelData.rowName ?? ""
                            width: 430 * root.px
                            height: 88 * root.px
                            radius: 16 * root.px
                            color: presetRow.isFocused ? "#151826" : "transparent"
                            border.width: (!presetRow.isFocused && presetRow.isSelected) ? 1 * root.px : 0
                            border.color: "#4A5068"
                            Kit.FocusRing {
                                objectName: presetRow.modelData.ringName ?? ""
                                visible: presetRow.isFocused
                                anchors.fill: parent
                                radius: 16 * root.px
                                gap: 3 * root.px
                                thickness: 3 * root.px
                                bandRadius: 22 * root.px
                                glowColor: "#66F0B4C4"
                                glowBlur: 34 * root.px
                                glowSpread: 8 * root.px
                                pixelScale: canvas.unit
                            }
                            Rectangle {
                                id: swatchBox
                                x: 14 * root.px
                                y: 13 * root.px
                                width: 84 * root.px
                                height: 62 * root.px
                                radius: 8 * root.px
                                color: "#17306E"
                                clip: true
                                Rectangle {
                                    x: 52 * root.px
                                    y: 8 * root.px
                                    width: 18 * root.px
                                    height: 18 * root.px
                                    color: "#B8FFE9"
                                }
                                Rectangle {
                                    x: 0
                                    y: 38 * root.px
                                    width: swatchBox.width
                                    height: 24 * root.px
                                    color: "#0E5A50"
                                }
                                Repeater {
                                    model: presetRow.drawScan ? 10 : 0
                                    Rectangle {
                                        id: scanBar
                                        required property int index
                                        x: 0
                                        y: scanBar.index * 6 * root.px
                                        width: swatchBox.width
                                        height: 2 * root.px
                                        color: "#4D000000"
                                    }
                                }
                            }
                            CardLine {
                                id: pName
                                objectName: presetRow.modelData.nameName ?? ""
                                x: 114 * root.px
                                y: 18.5 * root.px
                                width: (presetRow.isFav ? 262 : 296) * root.px
                                unit: root.px
                                boardLine: true
                                size: 21
                                weight: Font.DemiBold
                                color: "#F4F1EC"
                                elide: Text.ElideRight
                                text: presetRow.modelData.name ?? ""
                            }
                            CardLine {
                                id: pMeta
                                objectName: presetRow.modelData.metaName ?? ""
                                x: 114 * root.px
                                y: 48.5 * root.px
                                width: (presetRow.isFav ? 262 : 296) * root.px
                                unit: root.px
                                boardLine: true
                                size: 16
                                weight: Font.Normal
                                color: "#B9BED0"
                                elide: Text.ElideRight
                                text: presetRow.modelData.meta ?? ""
                            }
                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: canvas.settings.shadersFocusPreset(presetRow.modelData.index ?? presetRow.index, true)
                            }
                            Item {
                                id: starBtn
                                objectName: presetRow.modelData.starName ?? ""
                                visible: presetRow.isFav || presetRow.isFocused
                                x: 392 * root.px
                                y: 32 * root.px
                                width: 24 * root.px
                                height: 24 * root.px
                                Shape {
                                    id: starShape
                                    readonly property real box: 24
                                    width: starShape.box
                                    height: starShape.box
                                    scale: (24 * root.px) / starShape.box
                                    transformOrigin: Item.TopLeft
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        fillColor: presetRow.isFav ? "#F3C9D2" : "transparent"
                                        strokeColor: presetRow.isFav ? "#F3C9D2" : "#5A6076"
                                        strokeWidth: 1.4
                                        joinStyle: ShapePath.RoundJoin
                                        startX: 12; startY: 3
                                        PathLine { x: 14.6; y: 8.6 }
                                        PathLine { x: 20.6; y: 9.3 }
                                        PathLine { x: 16.1; y: 13.4 }
                                        PathLine { x: 17.4; y: 19.4 }
                                        PathLine { x: 12; y: 16.4 }
                                        PathLine { x: 6.6; y: 19.4 }
                                        PathLine { x: 7.9; y: 13.4 }
                                        PathLine { x: 3.4; y: 9.3 }
                                        PathLine { x: 9.4; y: 8.6 }
                                        PathLine { x: 12; y: 3 }
                                    }
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: canvas.settings.shadersToggleFavourite(presetRow.modelData.index ?? presetRow.index)
                                }
                            }
                        }
                    }
                }
                CardLine {
                    objectName: "presetEmptyText"
                    visible: root.presets.length === 0
                    x: root.reach + 16 * root.px
                    y: root.reach + 24 * root.px
                    width: 398 * root.px
                    unit: root.px
                    boardLine: true
                    size: 18
                    weight: Font.Normal
                    color: "#B9BED0"
                    wrapMode: Text.WordWrap
                    text: root.model.emptyListText ?? ""
                }
            }
            // Right column: scope switch (x 1004, y 130, w 820, h 60).
            Rectangle {
                id: scopeSwitch
                objectName: "scopeSwitch"
                x: 1004 * root.px
                y: 130 * root.px
                width: 820 * root.px
                height: 60 * root.px
                radius: 32 * root.px
                color: "#11131C"
                border.width: 1 * root.px
                border.color: "#262A3A"
                Repeater {
                    model: root.scopes
                    Rectangle {
                        id: scpTab
                        required property int index
                        required property var modelData
                        readonly property bool isActive: scpTab.modelData.active === true
                        readonly property bool isFocused: scpTab.modelData.focused === true
                        objectName: scpTab.modelData.pillName ?? ""
                        x: (6 + scpTab.index * 404) * root.px
                        y: 6 * root.px
                        width: 404 * root.px
                        height: 48 * root.px
                        radius: 24 * root.px
                        color: scpTab.isActive ? "#F6E7D8" : "transparent"
                        Kit.FocusRing {
                            objectName: scpTab.modelData.ringName ?? ""
                            visible: scpTab.isFocused
                            anchors.fill: parent
                            radius: 24 * root.px
                            gap: 3 * root.px
                            thickness: 3 * root.px
                            bandRadius: 30 * root.px
                            glowColor: "#66F0B4C4"
                            glowBlur: 34 * root.px
                            glowSpread: 8 * root.px
                            pixelScale: canvas.unit
                        }
                        CardLine {
                            id: scpWords
                            objectName: scpTab.modelData.textName ?? ""
                            x: (scpTab.width - scpWords.width) / 2
                            y: (scpTab.height - scpWords.height) / 2
                            unit: root.px
                            boardLine: true
                            size: 19
                            weight: scpTab.isActive ? Font.DemiBold : Font.Normal
                            color: scpTab.isActive ? "#07080F" : "#D8DBE6"
                            text: scpTab.modelData.text ?? ""
                        }
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: canvas.settings.shadersScope(scpTab.modelData.index ?? scpTab.index)
                        }
                    }
                }
            }
            // Right column: live Serenity Vulkan before/after split preview (x 1004, y 208, 820x560).
            Item {
                id: previewBox
                objectName: "previewBox"
                x: 1004 * root.px
                y: 208 * root.px
                width: 820 * root.px
                height: 560 * root.px
                Kit.Glow {
                    objectName: "previewGlow"
                    anchors.fill: parent
                    radius: 20 * root.px
                    color: "#38788CFF"
                    blur: 70 * root.px
                    spread: 12 * root.px
                    pixelScale: canvas.unit
                }
                Rectangle {
                    objectName: "previewRim"
                    x: -2 * root.px
                    y: -2 * root.px
                    width: previewBox.width + 4 * root.px
                    height: previewBox.height + 4 * root.px
                    radius: 22 * root.px
                    color: "transparent"
                    border.width: 2 * root.px
                    border.color: "#8CF6E7D8"
                }
                Item {
                    id: previewLayers
                    objectName: "previewLayers"
                    anchors.fill: parent
                    Rectangle {
                        anchors.fill: parent
                        color: "#17306E"
                    }
                    Image {
                        id: unfilteredPic
                        objectName: "previewUnfiltered"
                        anchors.fill: parent
                        source: canvas.settings.shaderPreviewSource
                        fillMode: Image.PreserveAspectCrop
                        smooth: false
                        cache: false
                    }
                    Item {
                        id: filteredClip
                        objectName: "previewFilteredClip"
                        x: root.splitPx
                        y: 0
                        width: Math.max(0, previewBox.width - root.splitPx)
                        height: previewBox.height
                        clip: true
                        visible: root.hasFiltered && canvas.settings.shaderPreviewFeed !== 0
                        Kit.GamePicture {
                            id: filteredPic
                            objectName: "previewFiltered"
                            x: -root.splitPx
                            y: 0
                            width: previewBox.width
                            height: previewBox.height
                            feed: canvas.settings.shadersShown ? canvas.settings.shaderPreviewFeed : 0
                            live: false
                        }
                    }
                    Connections {
                        target: canvas.settings
                        function onShaderPreviewReady() { filteredPic.update() }
                    }
                }
                ShaderEffectSource {
                    id: previewTex
                    sourceItem: previewLayers
                    hideSource: true
                    live: true
                    visible: false
                    textureSize: Qt.size(Math.max(1, Math.ceil(previewBox.width)), Math.max(1, Math.ceil(previewBox.height)))
                }
                Shape {
                    objectName: "previewClipped"
                    anchors.fill: parent
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        strokeColor: "transparent"
                        fillColor: "transparent"
                        fillItem: previewTex
                        fillTransform: PlanarTransform.fromScale(previewBox.width / Math.max(1, previewTex.textureSize.width), previewBox.height / Math.max(1, previewTex.textureSize.height))
                        PathRectangle {
                            width: previewBox.width
                            height: previewBox.height
                            radius: 20 * root.px
                        }
                    }
                }
                Rectangle {
                    objectName: "previewDivider"
                    x: root.splitPx - 2 * root.px
                    y: 0
                    width: 4 * root.px
                    height: previewBox.height
                    color: "#F6E7D8"
                    Kit.Glow {
                        anchors.fill: parent
                        color: "#B2F6D6C8"
                        blur: 22 * root.px
                        spread: 4 * root.px
                        pixelScale: canvas.unit
                    }
                }
                Rectangle {
                    objectName: "previewHandle"
                    x: root.splitPx - 30 * root.px
                    y: (previewBox.height - 60 * root.px) / 2
                    width: 60 * root.px
                    height: 60 * root.px
                    radius: 30 * root.px
                    color: "#07080F"
                    border.width: 2 * root.px
                    border.color: "#F6E7D8"
                    Kit.Glow {
                        anchors.fill: parent
                        radius: 30 * root.px
                        color: "#99F6D6C8"
                        blur: 26 * root.px
                        spread: 6 * root.px
                        pixelScale: canvas.unit
                    }
                    Shape {
                        id: handleShape
                        anchors.centerIn: parent
                        readonly property real box: 24
                        width: handleShape.box
                        height: handleShape.box
                        scale: (30 * root.px) / handleShape.box
                        transformOrigin: Item.Center
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            fillColor: "transparent"
                            strokeColor: "#F6E7D8"
                            strokeWidth: 2
                            capStyle: ShapePath.RoundCap
                            joinStyle: ShapePath.RoundJoin
                            startX: 9; startY: 7
                            PathLine { x: 4; y: 12 }
                            PathLine { x: 9; y: 17 }
                            PathMove { x: 15; y: 7 }
                            PathLine { x: 20; y: 12 }
                            PathLine { x: 15; y: 17 }
                        }
                    }
                }
                Rectangle {
                    id: beforeBadge
                    objectName: "previewBeforePill"
                    x: 16 * root.px
                    y: 16 * root.px
                    width: beforeWord.width + 28 * root.px
                    height: 38 * root.px
                    radius: 19 * root.px
                    color: "#D107080F"
                    CardLine {
                        id: beforeWord
                        objectName: "previewBeforeWords"
                        x: 14 * root.px
                        y: (beforeBadge.height - beforeWord.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 17
                        weight: Font.Normal
                        color: "#F4F1EC"
                        text: root.labels.noFilter ?? ""
                    }
                }
                Rectangle {
                    id: afterBadge
                    objectName: "previewAfterPill"
                    x: previewBox.width - 16 * root.px - afterBadge.width
                    y: 16 * root.px
                    width: afterWord.width + 28 * root.px
                    height: 38 * root.px
                    radius: 19 * root.px
                    color: "#F6E7D8"
                    CardLine {
                        id: afterWord
                        objectName: "previewAfterWords"
                        x: 14 * root.px
                        y: (afterBadge.height - afterWord.height) / 2
                        unit: root.px
                        boardLine: true
                        size: 17
                        weight: Font.DemiBold
                        color: "#07080F"
                        text: root.preview.afterLabel ?? ""
                    }
                }
                MouseArea {
                    id: splitMouse
                    anchors.fill: parent
                    cursorShape: Qt.SplitHCursor
                    onPressed: (mouse) => canvas.settings.shadersSetSplit(mouse.x / previewBox.width)
                    onPositionChanged: (mouse) => { if (splitMouse.pressed) canvas.settings.shadersSetSplit(mouse.x / previewBox.width) }
                }
            }
            // Right column bottom: preset heading, metadata, 3 sliders, "All N settings" link (x 1004, y 794, w 820).
            Item {
                id: asideArea
                objectName: "shadersAside"
                x: 1004 * root.px
                y: 794 * root.px
                width: 820 * root.px
                height: 240 * root.px
                CardLine {
                    id: headingLine
                    objectName: "presetHeading"
                    x: 0
                    y: 0
                    width: asideArea.width
                    unit: root.px
                    boardLine: true
                    size: 36
                    weight: Font.DemiBold
                    color: "#F4F1EC"
                    elide: Text.ElideRight
                    text: root.preview.heading ?? ""
                }
                CardLine {
                    id: subtitleLine
                    objectName: "presetSubtitle"
                    x: 0
                    y: 51 * root.px
                    width: asideArea.width
                    unit: root.px
                    boardLine: true
                    size: 18
                    weight: Font.Normal
                    color: "#C9CDDB"
                    elide: Text.ElideRight
                    text: root.preview.subtitle ?? ""
                }
                Row {
                    id: slidersRow
                    objectName: "slidersRow"
                    visible: root.sliders.length > 0
                    x: 0
                    y: 91 * root.px
                    width: asideArea.width
                    spacing: 26 * root.px
                    Repeater {
                        model: root.sliders
                        Item {
                            id: slCol
                            required property int index
                            required property var modelData
                            readonly property bool isFocused: slCol.modelData.focused === true
                            readonly property real fillNorm: slCol.modelData.ratio ?? 0
                            objectName: slCol.modelData.boxName ?? ""
                            width: 256 * root.px
                            height: 50 * root.px
                            Kit.FocusRing {
                                objectName: slCol.modelData.ringName ?? ""
                                visible: slCol.isFocused
                                x: -8 * root.px
                                y: -6 * root.px
                                width: slCol.width + 16 * root.px
                                height: slCol.height + 12 * root.px
                                radius: 10 * root.px
                                gap: 3 * root.px
                                thickness: 3 * root.px
                                bandRadius: 16 * root.px
                                glowColor: "#66F0B4C4"
                                glowBlur: 34 * root.px
                                glowSpread: 8 * root.px
                                pixelScale: canvas.unit
                            }
                            CardLine {
                                id: slLabel
                                objectName: slCol.modelData.labelName ?? ""
                                x: 0
                                y: 0
                                width: slCol.width - slVal.width - 10 * root.px
                                unit: root.px
                                boardLine: true
                                size: 18
                                weight: Font.Normal
                                color: "#F4F1EC"
                                elide: Text.ElideRight
                                text: slCol.modelData.label ?? ""
                            }
                            CardLine {
                                id: slVal
                                objectName: slCol.modelData.valName ?? ""
                                x: slCol.width - slVal.width
                                y: 0
                                unit: root.px
                                boardLine: true
                                size: 18
                                weight: Font.Normal
                                color: "#D8DBE6"
                                text: slCol.modelData.value ?? ""
                            }
                            Rectangle {
                                id: slTrack
                                x: 0
                                y: 32 * root.px
                                width: slCol.width
                                height: 8 * root.px
                                radius: 4 * root.px
                                color: "#262A3A"
                                Rectangle {
                                    x: 0
                                    y: 0
                                    width: slTrack.width * slCol.fillNorm
                                    height: slTrack.height
                                    radius: 4 * root.px
                                    color: "#F6E7D8"
                                }
                                Rectangle {
                                    objectName: slCol.modelData.knobName ?? ""
                                    x: slTrack.width * slCol.fillNorm - 13 * root.px
                                    y: -9 * root.px
                                    width: 26 * root.px
                                    height: 26 * root.px
                                    radius: 13 * root.px
                                    color: "#07080F"
                                    border.width: 3 * root.px
                                    border.color: "#F6E7D8"
                                }
                            }
                            MouseArea {
                                id: sliderMouse
                                x: 0
                                y: 14 * root.px
                                width: slCol.width
                                height: 38 * root.px
                                cursorShape: Qt.PointingHandCursor
                                onPressed: (mouse) => canvas.settings.shadersSlideSlider(slCol.modelData.slot ?? slCol.index, mouse.x / slTrack.width)
                                onPositionChanged: (mouse) => { if (sliderMouse.pressed) canvas.settings.shadersSlideSlider(slCol.modelData.slot ?? slCol.index, mouse.x / slTrack.width) }
                            }
                        }
                    }
                }
                Item {
                    id: allLink
                    objectName: "allSettingsLink"
                    visible: root.allSettings.visible === true
                    readonly property bool isFocused: root.allSettings.focused === true
                    x: 0
                    y: (root.sliders.length > 0 ? 149 : 91) * root.px
                    width: allWords.width
                    height: allWords.height
                    Kit.FocusRing {
                        objectName: "allSettingsRing"
                        visible: allLink.isFocused
                        x: -8 * root.px
                        y: -4 * root.px
                        width: allLink.width + 16 * root.px
                        height: allLink.height + 8 * root.px
                        radius: 8 * root.px
                        gap: 3 * root.px
                        thickness: 3 * root.px
                        bandRadius: 14 * root.px
                        glowColor: "#66F0B4C4"
                        glowBlur: 34 * root.px
                        glowSpread: 8 * root.px
                        pixelScale: canvas.unit
                    }
                    CardLine {
                        id: allWords
                        objectName: "allSettingsWords"
                        unit: root.px
                        boardLine: true
                        size: 18
                        weight: Font.Normal
                        color: "#F6E7D8"
                        text: root.allSettings.text ?? ""
                    }
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: canvas.settings.shadersAllSettings()
                    }
                }
            }
            // Footer help bar (left 96, bottom 34, middle 1147).
            Kit.HelpBar {
                id: helpBar
                objectName: "help"
                x: 96 * root.px
                y: root.height - 53 * root.px - helpBar.height / 2
                model: helpRows
                family: canvas.padFamily
                fitted: true
                ruled: false
                color: "transparent"
                labelColor: "#D8DBE6"
                iconColor: "#B9BED0"
                leadStyle: Kit.GlyphStyles.Filled
                leadColor: Kit.Tokens.accent
                entrySpacing: 30 * root.px
                glyphGap: 10 * root.px
                glyphSize: 34 * root.px
                snapped: false
                onPressed: (button) => canvas.settings.shadersPadButton(button, true, false)
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
                        ink: "#D8DBE6"
                        ring: "#B9BED0"
                        leadFill: Kit.Tokens.accent
                        leadInk: Kit.Tokens.overAccent
                    }
                }
            }
            // Pack removal confirm (#14) over the board.
            Kit.Frame {
                id: confirmFrame
                anchors.fill: parent
                form: Kit.Forms.BigScreen
                surfaceDrawn: false
                helpShown: false
                focus: root.confirmOpen
                controller.navigationShown: false
                QuestionHost {
                    id: removeConfirm
                    objectName: "removeConfirm"
                    controller: confirmFrame.controller
                    title: root.labels.confirmTitle ?? ""
                    message: root.labels.confirmMessage ?? ""
                    choices: root.labels.confirmChoices ?? []
                    backIndex: 0
                    padFamily: canvas.padFamily
                    titleFont: Qt.font({ family: Kit.Tokens.faceLabel, pixelSize: Kit.Tokens.fontTitle, weight: Kit.Tokens.weightTitle })
                    bodyFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
                    captionFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontCaption })
                    rowFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontRow })
                    helpFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: 20 })
                    help: HelpModel { screen: HelpModel.QuestionBack }
                    showing: root.confirmOpen
                    onChosen: (index) => canvas.settings.shadersConfirmRemove(index === 1)
                }
            }
        }
    }
}
