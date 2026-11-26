pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Eclipse Cores board (#10): console filter strip, 3-column console grid, right aside, and dynamic help footer.
FocusScope {
    id: root
    required property ToolModel tool
    required property int padFamily
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    signal backed()
    readonly property real unit: Kit.CanvasUnits.designHeight / 1200
    readonly property var view: JSON.parse(root.tool.eclipseCores)
    readonly property var labels: root.view.labels ?? ({})
    readonly property var aside: root.view.aside ?? ({})
    readonly property var statusPill: root.view.statusPill ?? ({})
    readonly property int focusZone: root.view.focusZoneCode ?? 1
    readonly property int focusedTopIndex: root.view.focusedTopIndex ?? 0
    readonly property int focusedConsoleIndex: root.view.focusedConsoleIndex ?? -1
    readonly property int focusedAsideIndex: root.view.focusedAsideIndex ?? -1
    readonly property alias controller: frame.controller
    readonly property bool sheetOpen: frame.controller.sheetOpen
    objectName: "coresEclipse"
    property bool backdrop: root.tool.cardOpen || root.tool.asking

    component LibretroMark: Item {
        id: mark
        required property real side
        required property color light
        required property color dark
        width: mark.side * root.unit
        height: mark.side * root.unit
        Shape {
            width: 11
            height: 11
            scale: (mark.side * root.unit) / 11
            transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: mark.light
                strokeColor: "transparent"
                PathSvg { path: Marks.badge }
            }
            ShapePath {
                fillColor: mark.dark
                strokeColor: "transparent"
                PathSvg { path: Marks.alien }
            }
        }
    }

    component CoreHelpGlyph: Item {
        id: cg
        property int button: Kit.Pad.Accept
        property int family: root.padFamily
        property bool lead: false
        property int pair: -1
        readonly property bool drawsPair: cg.button === Kit.Pad.Left || cg.button === Kit.Pad.PageUp
        readonly property bool pairedShoulders: cg.button === Kit.Pad.PageUp && cg.pair >= 0
        readonly property bool named: primaryGlyph.named
        readonly property string word: primaryGlyph.word
        readonly property string letter: primaryGlyph.letter
        readonly property bool filled: primaryGlyph.filled
        readonly property real wordSize: primaryGlyph.wordSize
        readonly property real letterSize: primaryGlyph.letterSize
        readonly property real leadLetterSize: primaryGlyph.leadLetterSize
        readonly property int leadWeight: primaryGlyph.leadWeight
        readonly property real corner: primaryGlyph.corner
        readonly property real inset: primaryGlyph.inset
        implicitWidth: pairRow.implicitWidth
        implicitHeight: pairRow.implicitHeight
        width: implicitWidth
        height: implicitHeight
        Row {
            id: pairRow
            spacing: 5 * root.unit
            CardGlyph {
                id: primaryGlyph
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                button: cg.button
                family: cg.family
                lead: cg.lead
                pair: cg.button === Kit.Pad.Left ? cg.pair : -1
                side: 34
                boxSide: 30
                boxRadius: 9
                boxPadding: 10
                wordSize: 14
                letterSize: 16
                leadLetterSize: 17
                leadWeight: Font.Bold
                padding: 10
                boxMenu: true
                boxOptions: true
                ink: ThemeInk.body
                ring: Kit.Tokens.muted
            }
            CardGlyph {
                anchors.verticalCenter: parent.verticalCenter
                visible: cg.pairedShoulders
                unit: root.unit
                button: cg.pair >= 0 ? cg.pair : Kit.Pad.PageDown
                family: cg.family
                lead: false
                side: 34
                boxSide: 30
                boxRadius: 9
                boxPadding: 10
                wordSize: 14
                letterSize: 16
                padding: 10
                boxMenu: true
                boxOptions: true
                ink: ThemeInk.body
                ring: Kit.Tokens.muted
            }
        }
    }

    onFocusedAsideIndexChanged: {
        if (root.focusedAsideIndex < 0) {
            asideFlick.contentY = 0
            return
        }
        const item = root.focusedAsideIndex < instRep.count ? instRep.itemAt(root.focusedAsideIndex) : availRep.itemAt(root.focusedAsideIndex - instRep.count)
        if (!item) return
        const top = item.mapToItem(asideCol, 0, 0).y
        const bottom = top + item.height
        const viewH = asideFlick.height - 48 * root.unit
        if (top < asideFlick.contentY) asideFlick.contentY = top
        else if (bottom > asideFlick.contentY + viewH) asideFlick.contentY = Math.max(0, bottom - viewH)
    }

    Kit.Backdrop {
        objectName: "coresBackdrop"
        anchors.fill: parent
        ground: Kit.Tokens.surface
        pixelScale: root.pixelScale
        fields: [
            Kit.BackdropField {
                x: 1250 * root.unit
                y: 80 * root.unit
                width: 900 * root.unit
                height: 900 * root.unit
                color: ThemeInk.backdropGlow
                blur: 150 * root.unit
                opacity: 0.36
            }
        ]
    }

    Kit.Frame {
        id: frame
        objectName: "coresEclipseFrame"
        anchors.fill: parent
        focus: true
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        controller.navigationShown: false
        controller.actionLines: root.view.actionLines ?? ""
        Connections {
            target: frame.controller
            function onBackDeclined() { root.backed() }
            function onActionInvoked(key) { root.tool.eclipseActionKey(key) }
        }
        FocusScope {
            id: keyScope
            anchors.fill: parent
            focus: true
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Up) { root.tool.eclipseNav(0); event.accepted = true }
                else if (event.key === Qt.Key_Down) { root.tool.eclipseNav(1); event.accepted = true }
                else if (event.key === Qt.Key_Left) { root.tool.eclipseNav(2); event.accepted = true }
                else if (event.key === Qt.Key_Right) { root.tool.eclipseNav(3); event.accepted = true }
                else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) { root.tool.eclipseNav(4); event.accepted = true }
            }
        }
    }

    Item {
        objectName: "coresHeader"
        x: 96 * root.unit
        y: 48 * root.unit
        width: root.width - 192 * root.unit
        height: 64 * root.unit
        Item {
            objectName: "coresMark"
            x: 0
            y: (parent.height - height) / 2
            width: 34 * root.unit
            height: 34 * root.unit
            Shape {
                width: 34
                height: 34
                scale: root.unit
                transformOrigin: Item.TopLeft
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: Kit.Tokens.accent
                    strokeColor: "transparent"
                    fillRule: ShapePath.OddEvenFill
                    PathSvg { path: Marks.menuCrescent }
                }
            }
        }
        Row {
            objectName: "coresTrail"
            x: 62 * root.unit
            y: (parent.height - height) / 2
            spacing: 14 * root.unit
            CardLine {
                objectName: "trailWords.0"
                unit: root.unit
                boardLine: true
                text: root.labels.tools ?? ""
                size: 24
                weight: Font.Normal
                color: Kit.Tokens.muted
                MouseArea {
                    anchors.fill: parent
                    onClicked: root.backed()
                }
            }
            CardLine {
                objectName: "trailSlash.0"
                unit: root.unit
                boardLine: true
                text: root.labels.slash ?? ""
                size: 24
                weight: Font.Normal
                color: ThemeInk.dim
            }
            CardLine {
                objectName: "trailWords.1"
                unit: root.unit
                boardLine: true
                text: root.labels.cores ?? ""
                size: 24
                weight: Font.DemiBold
                color: Kit.Tokens.textStrong
            }
        }
        Row {
            anchors.right: parent.right
            y: (parent.height - height) / 2
            spacing: 22 * root.unit
            Rectangle {
                id: statusPillBox
                objectName: "coresStatusPill"
                visible: root.statusPill.visible === true
                y: (parent.height - height) / 2
                height: 44 * root.unit
                width: statusLine.implicitWidth + 32 * root.unit
                radius: 22 * root.unit
                color: root.statusPill.error ? ThemeInk.errorFill : Kit.Tokens.surfaceRaised
                border.width: 2 * root.unit
                border.color: root.statusPill.error ? Kit.Tokens.error : ThemeInk.pillBorder
                CardLine {
                    id: statusLine
                    objectName: "coresStatusText"
                    x: (parent.width - width) / 2
                    y: (parent.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.statusPill.text ?? ""
                    size: 17
                    weight: Font.Normal
                    color: root.statusPill.error ? Kit.Tokens.error : Kit.Tokens.textStrong
                }
            }
            CardLine {
                objectName: "battery"
                y: (parent.height - height) / 2
                visible: root.battery >= 0
                unit: root.unit
                boardLine: true
                text: root.battery >= 0 ? Words.percent(root.battery) : ""
                size: 22
                weight: Font.Normal
                color: Kit.Tokens.textStrong
            }
            CardLine {
                objectName: "clock"
                y: (parent.height - height) / 2
                visible: root.clockText.length > 0
                unit: root.unit
                boardLine: true
                text: root.clockText
                size: 22
                weight: Font.DemiBold
                color: Kit.Tokens.textStrong
            }
        }
    }

    Item {
        id: coresTopStrip
        objectName: "coresTopStrip"
        x: 96 * root.unit
        y: 146 * root.unit
        width: 1124 * root.unit
        height: 52 * root.unit
        Row {
            objectName: "coresFilters"
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            spacing: 12 * root.unit
            Repeater {
                id: filterRep
                model: root.view.filters ?? []
                Item {
                    id: filterPill
                    required property int index
                    required property var modelData
                    objectName: "filterPill." + filterPill.modelData.key
                    TextMetrics {
                        id: filterMetrics
                        font.family: Kit.Tokens.faceText
                        font.pixelSize: 21
                        font.weight: filterPill.modelData.active ? Font.DemiBold : Font.Normal
                        text: filterPill.modelData.label ?? ""
                    }
                    readonly property real naturalWidth: filterMetrics.advanceWidth * root.unit + 44 * root.unit
                    readonly property real maxWidth: Math.max(120 * root.unit, (coresTopStrip.width - viewButton.width - 12 * root.unit - Math.max(0, filterRep.count - 1) * 12 * root.unit) / Math.max(1, filterRep.count))
                    height: 52 * root.unit
                    width: Math.min(filterPill.naturalWidth, filterPill.maxWidth)
                    readonly property bool focused: root.focusZone === 0 && root.focusedTopIndex === filterPill.index
                    Kit.FocusRing {
                        visible: filterPill.focused
                        anchors.fill: parent
                        gap: 3 * root.unit
                        thickness: 3 * root.unit
                        bandRadius: 29 * root.unit
                        glowBlur: 36 * root.unit
                        glowSpread: 6 * root.unit
                        glowColor: ThemeInk.glowMid
                        pixelScale: root.pixelScale
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: 26 * root.unit
                        color: filterPill.modelData.active ? Kit.Tokens.accent : "transparent"
                        border.width: filterPill.modelData.active ? 0 : 2 * root.unit
                        border.color: filterPill.modelData.attention ? Kit.Tokens.error : ThemeInk.pillBorder
                    }
                    CardLine {
                        id: filterWords
                        objectName: "filterWords." + filterPill.modelData.key
                        anchors.centerIn: parent
                        width: Math.max(1, filterPill.width - 24 * root.unit)
                        unit: root.unit
                        boardLine: true
                        text: filterPill.modelData.label ?? ""
                        size: 21
                        weight: filterPill.modelData.active ? Font.DemiBold : Font.Normal
                        color: filterPill.modelData.active ? Kit.Tokens.overAccent : (filterPill.modelData.attention ? Kit.Tokens.textStrong : ThemeInk.body)
                        wrapMode: Text.WordWrap
                        maximumLineCount: 2
                        horizontalAlignment: Text.AlignHCenter
                        lineHeight: 0.95
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            root.tool.eclipseFocusZoneCode(0)
                            root.tool.eclipseChooseFilter(filterPill.modelData.key)
                        }
                    }
                }
            }
        }
        Item {
            id: viewButton
            objectName: "coresViewButton"
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            TextMetrics {
                id: viewMetrics
                font.family: Kit.Tokens.faceText
                font.pixelSize: 21
                font.weight: viewButton.active ? Font.DemiBold : Font.Normal
                text: root.view.viewLabel ?? ""
            }
            readonly property real maxLabelWidth: Math.max(120 * root.unit, 424 * root.unit - viewBadge.width - 42 * root.unit)
            height: 52 * root.unit
            width: viewBadge.width + viewLabel.width + 42 * root.unit
            readonly property bool focused: root.focusZone === 0 && root.focusedTopIndex === (root.view.filters?.length ?? 1)
            readonly property bool active: root.view.showingRemaining === true
            Kit.FocusRing {
                visible: viewButton.focused
                anchors.fill: parent
                gap: 3 * root.unit
                thickness: 3 * root.unit
                bandRadius: 29 * root.unit
                glowBlur: 36 * root.unit
                glowSpread: 6 * root.unit
                glowColor: ThemeInk.glowMid
                pixelScale: root.pixelScale
            }
            Rectangle {
                anchors.fill: parent
                radius: 26 * root.unit
                color: viewButton.active ? Kit.Tokens.accent : "transparent"
                border.width: viewButton.active ? 0 : 2 * root.unit
                border.color: ThemeInk.pillBorder
            }
            Row {
                id: viewContent
                x: 10 * root.unit
                anchors.verticalCenter: parent.verticalCenter
                spacing: 10 * root.unit
                CardGlyph {
                    id: viewBadge
                    objectName: "viewBadge"
                    anchors.verticalCenter: parent.verticalCenter
                    unit: root.unit
                    family: root.padFamily
                    button: Kit.Pad.Options
                    side: 28
                    boxSide: 28
                    boxRadius: 8
                    boxPadding: 8
                    wordSize: 13
                    padding: 10
                    boxMenu: false
                    boxOptions: true
                    ink: viewButton.active ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                    ring: viewButton.active ? Kit.Tokens.overAccent : Kit.Tokens.muted
                }
                CardLine {
                    id: viewLabel
                    objectName: "viewLabel"
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(viewMetrics.advanceWidth * root.unit, viewButton.maxLabelWidth)
                    unit: root.unit
                    boardLine: true
                    text: root.view.viewLabel ?? ""
                    size: 21
                    weight: viewButton.active ? Font.DemiBold : Font.Normal
                    color: viewButton.active ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                    wrapMode: Text.WordWrap
                    maximumLineCount: 2
                    horizontalAlignment: Text.AlignHCenter
                    lineHeight: 0.95
                }
            }
            MouseArea {
                anchors.fill: parent
                onClicked: root.tool.eclipseToggleView()
            }
        }
    }

    GridView {
        id: grid
        objectName: "coresGrid"
        x: (96 - 64) * root.unit
        y: (232 - 28) * root.unit
        width: (1124 + 128) * root.unit
        height: (814 + 72) * root.unit
        topMargin: 28 * root.unit
        bottomMargin: 22 * root.unit
        leftMargin: 64 * root.unit
        rightMargin: 42 * root.unit
        cellWidth: ((1124 + 20) / 3) * root.unit
        cellHeight: (258 + 20) * root.unit
        clip: (root.view.cards?.length ?? 0) > 9
        interactive: (root.view.cards?.length ?? 0) > 9
        boundsBehavior: Flickable.StopAtBounds
        highlightMoveDuration: 0
        model: root.view.cards ?? []
        Binding {
            target: grid
            property: "currentIndex"
            value: Math.max(0, root.focusedConsoleIndex)
        }
        delegate: Item {
            id: cardCell
            required property int index
            required property var modelData
            width: grid.cellWidth
            height: grid.cellHeight
            readonly property bool cardFocused: root.focusZone === 1 && root.focusedConsoleIndex === cardCell.index
            readonly property bool cardSelected: root.focusZone !== 1 && root.focusedConsoleIndex === cardCell.index
            Item {
                id: cardBox
                objectName: "consoleCard." + cardCell.index
                x: 0
                y: 0
                width: 361.3333 * root.unit
                height: 258 * root.unit
                Kit.FocusRing {
                    objectName: "consoleRing." + cardCell.index
                    visible: cardCell.cardFocused
                    anchors.fill: parent
                    gap: 4 * root.unit
                    thickness: 3 * root.unit
                    bandRadius: 29 * root.unit
                    glowBlur: 44 * root.unit
                    glowSpread: 10 * root.unit
                    glowColor: ThemeInk.glowMid
                    pixelScale: root.pixelScale
                }
                Rectangle {
                    objectName: "consoleBg." + cardCell.index
                    anchors.fill: parent
                    radius: 22 * root.unit
                    color: cardCell.cardSelected ? Kit.Tokens.surfaceRaised : Kit.Tokens.inputSurface
                    border.width: (cardCell.cardSelected ? 1.5 : 1) * root.unit
                    border.color: cardCell.cardSelected ? ThemeInk.accentSoft : ThemeInk.hairline
                }
                Item {
                    anchors.fill: parent
                    anchors.leftMargin: 24 * root.unit
                    anchors.rightMargin: 24 * root.unit
                    anchors.topMargin: 22 * root.unit
                    anchors.bottomMargin: 22 * root.unit
                    Item {
                        id: topBlock
                        width: parent.width
                        height: Math.max(topTitles.implicitHeight, cardFlag.visible ? cardFlag.height : 0)
                        Column {
                            id: topTitles
                            width: cardFlag.visible ? parent.width - cardFlag.width - 12 * root.unit : parent.width
                            spacing: 4 * root.unit
                            CardLine {
                                objectName: "consoleName." + cardCell.index
                                unit: root.unit
                                boardLine: true
                                boardLineHeight: 23 * 1.15
                                width: parent.width
                                text: cardCell.modelData.name ?? ""
                                size: 23
                                weight: Font.DemiBold
                                color: Kit.Tokens.textStrong
                                wrapMode: Text.WordWrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                            }
                            CardLine {
                                objectName: "consoleMaker." + cardCell.index
                                unit: root.unit
                                boardLine: true
                                visible: text.length > 0
                                width: parent.width
                                text: cardCell.modelData.makerLine ?? ""
                                size: 16
                                weight: Font.Normal
                                color: Kit.Tokens.muted
                                elide: Text.ElideRight
                            }
                        }
                        Rectangle {
                            id: cardFlag
                            objectName: "consoleFlag." + cardCell.index
                            visible: (cardCell.modelData.flag ?? "").length > 0
                            anchors.right: parent.right
                            anchors.top: parent.top
                            height: 32 * root.unit
                            width: flagText.implicitWidth + 24 * root.unit
                            radius: 16 * root.unit
                            color: cardCell.modelData.flagRose ? ThemeInk.rose : "transparent"
                            border.width: cardCell.modelData.flagRose ? 0 : 2 * root.unit
                            border.color: Kit.Tokens.muted
                            CardLine {
                                id: flagText
                                objectName: "consoleFlagWords." + cardCell.index
                                x: (parent.width - width) / 2
                                y: (parent.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                text: cardCell.modelData.flag ?? ""
                                size: 15
                                weight: Font.Bold
                                color: cardCell.modelData.flagRose ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                            }
                        }
                    }
                    Item {
                        id: bottomBlock
                        anchors.bottom: parent.bottom
                        width: parent.width
                        height: Math.max(20 * root.unit, otherLineText.implicitHeight)
                        Row {
                            y: (parent.height - height) / 2
                            width: parent.width
                            spacing: 10 * root.unit
                            LibretroMark {
                                objectName: "consoleOtherMark." + cardCell.index
                                y: (parent.height - height) / 2
                                visible: cardCell.modelData.otherCheckerboard === true
                                side: 18
                                light: Kit.Tokens.muted
                                dark: Kit.Tokens.surface
                            }
                            CardLine {
                                id: otherLineText
                                objectName: "consoleOther." + cardCell.index
                                y: (parent.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                boardLineHeight: 20
                                width: parent.width - (cardCell.modelData.otherCheckerboard ? 28 * root.unit : 0)
                                text: cardCell.modelData.otherLine ?? ""
                                size: 16
                                weight: Font.Normal
                                color: Kit.Tokens.muted
                                wrapMode: Text.WordWrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                            }
                        }
                    }
                    Item {
                        anchors.top: topBlock.bottom
                        anchors.bottom: bottomBlock.top
                        width: parent.width
                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width
                            spacing: 16 * root.unit
                            Item {
                                objectName: "consoleMark." + cardCell.index
                                anchors.verticalCenter: parent.verticalCenter
                                width: 54 * root.unit
                                height: 54 * root.unit
                                CardPlanet {
                                    anchors.fill: parent
                                    visible: (cardCell.modelData.planet ?? -2) >= 0
                                    unit: root.unit
                                    planet: cardCell.modelData.planet ?? 0
                                    side: 54
                                    glowBlur: 18
                                    glowSpread: 3
                                    glowAlpha: 0.533
                                    pixelScale: root.pixelScale
                                }
                                LibretroMark {
                                    anchors.fill: parent
                                    visible: cardCell.modelData.planet === -1
                                    side: 54
                                    light: Kit.Tokens.accent
                                    dark: Kit.Tokens.overAccent
                                }
                                Rectangle {
                                    anchors.fill: parent
                                    visible: (cardCell.modelData.planet ?? 0) <= -2
                                    radius: width / 2
                                    color: "transparent"
                                    border.width: 2 * root.unit
                                    border.color: cardCell.modelData.planet === -3 ? Kit.Tokens.error : ThemeInk.pillBorder
                                }
                            }
                            Column {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 70 * root.unit
                                spacing: 2 * root.unit
                                CardLine {
                                    objectName: "consoleCore." + cardCell.index
                                    unit: root.unit
                                    boardLine: true
                                    width: parent.width
                                    text: cardCell.modelData.coreName ?? ""
                                    size: 30
                                    weight: Font.DemiBold
                                    color: Kit.Tokens.textStrong
                                    elide: Text.ElideRight
                                }
                                CardLine {
                                    objectName: "consoleCoreKind." + cardCell.index
                                    unit: root.unit
                                    boardLine: true
                                    width: parent.width
                                    text: cardCell.modelData.coreSubtitle ?? ""
                                    size: 16
                                    weight: Font.Normal
                                    color: Kit.Tokens.muted
                                    elide: Text.ElideRight
                                }
                            }
                        }
                    }
                }
                MouseArea {
                    anchors.fill: parent
                    onClicked: {
                        root.tool.eclipseFocusZoneCode(1)
                        root.tool.eclipseFocusConsole(cardCell.modelData.console)
                    }
                }
            }
        }
    }

    Item {
        objectName: "coresAside"
        x: 1280 * root.unit
        y: 146 * root.unit
        width: 544 * root.unit
        height: 940 * root.unit
        Column {
            id: asideHeader
            width: parent.width
            spacing: 6 * root.unit
            CardLine {
                objectName: "asideEyebrow"
                unit: root.unit
                boardLine: true
                boardLineHeight: 22
                width: parent.width
                visible: text.length > 0
                text: root.aside.eyebrow ?? ""
                size: 18
                weight: Font.DemiBold
                spacing: 18 * 0.2
                capitalization: Font.AllUppercase
                color: ThemeInk.rose
                elide: Text.ElideRight
            }
            CardLine {
                objectName: "asideHeading"
                unit: root.unit
                boardLine: true
                width: parent.width
                text: root.aside.heading ?? ""
                size: 46
                weight: Font.DemiBold
                color: Kit.Tokens.textStrong
                elide: Text.ElideRight
            }
        }
        Flickable {
            id: asideFlick
            x: -48 * root.unit
            y: asideHeader.height
            width: 544 * root.unit + 96 * root.unit
            height: parent.height - asideHeader.height + 24 * root.unit
            leftMargin: 48 * root.unit
            rightMargin: 48 * root.unit
            topMargin: 24 * root.unit
            bottomMargin: 24 * root.unit
            contentWidth: 544 * root.unit
            contentHeight: asideCol.implicitHeight
            clip: asideCol.implicitHeight > height - 48 * root.unit
            interactive: asideCol.implicitHeight > height - 48 * root.unit
            boundsBehavior: Flickable.StopAtBounds
            Column {
                id: asideCol
                width: 544 * root.unit
                spacing: 16 * root.unit
                CardLine {
                    objectName: "asideInstalledHeading"
                    unit: root.unit
                    boardLine: true
                    width: parent.width
                    text: root.aside.installedHeading ?? root.labels.installedHeading ?? ""
                    size: 14
                    weight: Font.DemiBold
                    spacing: 2.1
                    capitalization: Font.AllUppercase
                    color: ThemeInk.dim
                }
                CardLine {
                    unit: root.unit
                    boardLine: true
                    width: parent.width
                    visible: (root.aside.installed?.length ?? 0) === 0
                    text: root.labels.emptyInstalled ?? ""
                    size: 17
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                }
                Repeater {
                    id: instRep
                    model: root.aside.installed ?? []
                    Item {
                        id: instCard
                        required property int index
                        required property var modelData
                        objectName: "asideCore." + instCard.index
                        readonly property bool cardFocused: root.focusZone === 2 && root.focusedAsideIndex === instCard.index
                        width: parent.width
                        height: instInner.implicitHeight + 36 * root.unit
                        Kit.FocusRing {
                            objectName: "asideCoreRing." + instCard.index
                            visible: instCard.cardFocused
                            anchors.fill: parent
                            gap: 3 * root.unit
                            thickness: 3 * root.unit
                            bandRadius: 24 * root.unit
                            glowBlur: 40 * root.unit
                            glowSpread: 8 * root.unit
                            glowColor: ThemeInk.glowRing
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            objectName: "asideCoreBg." + instCard.index
                            anchors.fill: parent
                            radius: 18 * root.unit
                            color: instCard.cardFocused ? Kit.Tokens.surfaceRaised : ThemeInk.rowSurface
                            border.width: instCard.cardFocused ? 0 : 1 * root.unit
                            border.color: ThemeInk.hairline
                        }
                        Column {
                            id: instInner
                            x: 20 * root.unit
                            y: 18 * root.unit
                            width: parent.width - 40 * root.unit
                            spacing: 12 * root.unit
                            Item {
                                width: parent.width
                                height: Math.max(40 * root.unit, instTitles.implicitHeight)
                                Row {
                                    anchors.left: parent.left
                                    y: (parent.height - height) / 2
                                    width: instBadge.visible ? parent.width - instBadge.width - 14 * root.unit : parent.width
                                    spacing: 14 * root.unit
                                    Item {
                                        objectName: "asideCoreMark." + instCard.index
                                        y: (parent.height - height) / 2
                                        width: 40 * root.unit
                                        height: 40 * root.unit
                                        CardPlanet {
                                            anchors.fill: parent
                                            visible: (instCard.modelData.planet ?? -2) >= 0
                                            unit: root.unit
                                            planet: instCard.modelData.planet ?? 0
                                            side: 40
                                            glowBlur: 18
                                            glowSpread: 3
                                            glowAlpha: 0.533
                                            pixelScale: root.pixelScale
                                        }
                                        LibretroMark {
                                            anchors.fill: parent
                                            visible: instCard.modelData.planet === -1
                                            side: 40
                                            light: Kit.Tokens.accent
                                            dark: Kit.Tokens.overAccent
                                        }
                                        Rectangle {
                                            anchors.fill: parent
                                            visible: (instCard.modelData.planet ?? 0) <= -2
                                            radius: width / 2
                                            color: "transparent"
                                            border.width: 2 * root.unit
                                            border.color: Kit.Tokens.error
                                        }
                                    }
                                    Column {
                                        id: instTitles
                                        y: (parent.height - height) / 2
                                        width: parent.width - 54 * root.unit
                                        spacing: 2 * root.unit
                                        CardLine {
                                            objectName: "asideCoreName." + instCard.index
                                            unit: root.unit
                                            boardLine: true
                                            width: parent.width
                                            text: instCard.modelData.name ?? ""
                                            size: 25
                                            weight: Font.DemiBold
                                            color: Kit.Tokens.textStrong
                                            elide: Text.ElideRight
                                        }
                                        CardLine {
                                            objectName: "asideCoreKind." + instCard.index
                                            unit: root.unit
                                            boardLine: true
                                            width: parent.width
                                            text: instCard.modelData.subtitle ?? ""
                                            size: 16
                                            weight: Font.Normal
                                            color: Kit.Tokens.muted
                                            elide: Text.ElideRight
                                        }
                                    }
                                }
                                Rectangle {
                                    id: instBadge
                                    objectName: "asideCoreBadge." + instCard.index
                                    visible: (instCard.modelData.badge ?? "").length > 0
                                    anchors.right: parent.right
                                    y: (parent.height - height) / 2
                                    height: 32 * root.unit
                                    width: instBadgeText.implicitWidth + 24 * root.unit
                                    radius: 16 * root.unit
                                    color: instCard.modelData.badgeRose ? ThemeInk.rose : "transparent"
                                    border.width: instCard.modelData.badgeRose ? 0 : 2 * root.unit
                                    border.color: Kit.Tokens.accent
                                    CardLine {
                                        id: instBadgeText
                                        x: (parent.width - width) / 2
                                        y: (parent.height - height) / 2
                                        unit: root.unit
                                        boardLine: true
                                        text: instCard.modelData.badge ?? ""
                                        size: 15
                                        weight: Font.DemiBold
                                        color: instCard.modelData.badgeRose ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
                                    }
                                }
                                MouseArea {
                                    anchors.fill: parent
                                    onClicked: {
                                        root.tool.eclipseFocusZoneCode(2)
                                        root.tool.eclipseFocusCore(instCard.modelData.key)
                                        root.tool.eclipseActCode(0, instCard.modelData.key)
                                    }
                                }
                            }
                            Column {
                                visible: instCard.modelData.hasUpdate === true
                                width: parent.width
                                spacing: 12 * root.unit
                                Rectangle {
                                    objectName: "asideTimelineBox." + instCard.index
                                    width: parent.width
                                    height: 40 * root.unit
                                    radius: 12 * root.unit
                                    color: Kit.Tokens.surface
                                    border.width: 1 * root.unit
                                    border.color: ThemeInk.hairline
                                    Item {
                                        x: 14 * root.unit
                                        width: parent.width - 28 * root.unit
                                        height: parent.height
                                        Row {
                                            id: haveDateRow
                                            anchors.left: parent.left
                                            anchors.verticalCenter: parent.verticalCenter
                                            CardLine {
                                                unit: root.unit
                                                boardLine: true
                                                text: root.labels.youHavePrefix ?? ""
                                                size: 14.5
                                                color: Kit.Tokens.muted
                                            }
                                            CardLine {
                                                objectName: "asideYouHave." + instCard.index
                                                unit: root.unit
                                                boardLine: true
                                                text: instCard.modelData.youHaveDate ?? ""
                                                size: 14.5
                                                family: Kit.Tokens.faceCode
                                                color: Kit.Tokens.textStrong
                                            }
                                        }
                                        Rectangle {
                                            objectName: "asideTimelineRule." + instCard.index
                                            x: haveDateRow.width + 12 * root.unit
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: Math.max(12 * root.unit, parent.width - haveDateRow.width - newerDateRow.width - 24 * root.unit)
                                            height: 2 * root.unit
                                            gradient: Gradient {
                                                orientation: Gradient.Horizontal
                                                GradientStop { position: 0.0; color: ThemeInk.pillBorder }
                                                GradientStop { position: 1.0; color: ThemeInk.rose }
                                            }
                                        }
                                        Row {
                                            id: newerDateRow
                                            anchors.right: parent.right
                                            anchors.verticalCenter: parent.verticalCenter
                                            CardLine {
                                                unit: root.unit
                                                boardLine: true
                                                text: root.labels.newerPrefix ?? ""
                                                size: 14.5
                                                weight: Font.DemiBold
                                                color: ThemeInk.rose
                                            }
                                            CardLine {
                                                objectName: "asideNewer." + instCard.index
                                                unit: root.unit
                                                boardLine: true
                                                text: (instCard.modelData.newerDate ?? "") + (instCard.modelData.newerSize ?? "")
                                                size: 14.5
                                                family: Kit.Tokens.faceCode
                                                color: Kit.Tokens.textStrong
                                            }
                                        }
                                    }
                                }
                                Item {
                                    width: parent.width
                                    height: 36 * root.unit
                                    Row {
                                        anchors.left: parent.left
                                        anchors.verticalCenter: parent.verticalCenter
                                        spacing: 8 * root.unit
                                        Item {
                                            objectName: "asidePillUpdate." + instCard.index
                                            visible: instCard.modelData.canUpdate === true
                                            height: 36 * root.unit
                                            width: updRow.implicitWidth + 18 * root.unit
                                            Rectangle {
                                                anchors.fill: parent
                                                radius: 18 * root.unit
                                                color: Kit.Tokens.accent
                                            }
                                            Row {
                                                id: updRow
                                                x: 6 * root.unit
                                                anchors.verticalCenter: parent.verticalCenter
                                                spacing: 6 * root.unit
                                                CardGlyph {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    family: root.padFamily
                                                    button: Kit.Pad.Accept
                                                    lead: true
                                                    leads: true
                                                    side: 24
                                                    leadLetterSize: 12.5
                                                    leadWeight: Font.Bold
                                                    leadFill: Kit.Tokens.overAccent
                                                    leadInk: Kit.Tokens.accent
                                                }
                                                CardLine {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    boardLine: true
                                                    text: root.labels.updatePill ?? ""
                                                    size: 14
                                                    weight: Font.DemiBold
                                                    color: Kit.Tokens.overAccent
                                                }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: root.tool.eclipseActCode(1, instCard.modelData.key)
                                            }
                                        }
                                        Item {
                                            objectName: "asidePillDefault." + instCard.index
                                            visible: instCard.modelData.canDefault === true
                                            height: 36 * root.unit
                                            width: defRow.implicitWidth + 18 * root.unit
                                            Rectangle {
                                                anchors.fill: parent
                                                radius: 18 * root.unit
                                                color: Kit.Tokens.surface
                                                border.width: 1 * root.unit
                                                border.color: ThemeInk.pillBorder
                                            }
                                            Row {
                                                id: defRow
                                                x: 6 * root.unit
                                                anchors.verticalCenter: parent.verticalCenter
                                                spacing: 6 * root.unit
                                                CardGlyph {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    family: root.padFamily
                                                    button: Kit.Pad.Screensaver
                                                    side: 22
                                                    letterSize: 11.5
                                                    ring: Kit.Tokens.muted
                                                    ink: Kit.Tokens.muted
                                                }
                                                CardLine {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    boardLine: true
                                                    text: root.labels.defaultPill ?? ""
                                                    size: 14
                                                    weight: Font.Normal
                                                    color: Kit.Tokens.textStrong
                                                }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: root.tool.eclipseActCode(2, instCard.modelData.key)
                                            }
                                        }
                                        Item {
                                            objectName: "asidePillRemove." + instCard.index
                                            visible: instCard.modelData.canRemove === true
                                            height: 36 * root.unit
                                            width: remRow.implicitWidth + 18 * root.unit
                                            Rectangle {
                                                anchors.fill: parent
                                                radius: 18 * root.unit
                                                color: Kit.Tokens.surface
                                                border.width: 1 * root.unit
                                                border.color: ThemeInk.pillBorder
                                            }
                                            Row {
                                                id: remRow
                                                x: 6 * root.unit
                                                anchors.verticalCenter: parent.verticalCenter
                                                spacing: 6 * root.unit
                                                CardGlyph {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    family: root.padFamily
                                                    button: Kit.Pad.Search
                                                    side: 22
                                                    letterSize: 11.5
                                                    ring: Kit.Tokens.muted
                                                    ink: Kit.Tokens.muted
                                                }
                                                CardLine {
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    unit: root.unit
                                                    boardLine: true
                                                    text: root.labels.removePill ?? ""
                                                    size: 14
                                                    weight: Font.Normal
                                                    color: Kit.Tokens.textStrong
                                                }
                                            }
                                            MouseArea {
                                                anchors.fill: parent
                                                onClicked: root.tool.eclipseActCode(3, instCard.modelData.key)
                                            }
                                        }
                                    }
                                    Item {
                                        objectName: "asidePillPage." + instCard.index
                                        visible: instCard.modelData.canUpdate === true
                                        anchors.right: parent.right
                                        anchors.verticalCenter: parent.verticalCenter
                                        height: 36 * root.unit
                                        width: pagePillLine.width + 24 * root.unit
                                        Rectangle {
                                            anchors.fill: parent
                                            radius: 18 * root.unit
                                            color: Kit.Tokens.surface
                                            border.width: 1 * root.unit
                                            border.color: ThemeInk.pillBorder
                                        }
                                        CardLine {
                                            id: pagePillLine
                                            anchors.centerIn: parent
                                            unit: root.unit
                                            boardLine: true
                                            text: root.labels.pagePill ?? ""
                                            size: 14
                                            weight: Font.Normal
                                            color: Kit.Tokens.textStrong
                                        }
                                        MouseArea {
                                            anchors.fill: parent
                                            onClicked: root.tool.eclipseActCode(0, instCard.modelData.key)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Item {
                    visible: (root.aside.availableHeading ?? "").length > 0
                    width: parent.width
                    height: availHeadLine.implicitHeight + 8 * root.unit
                    CardLine {
                        id: availHeadLine
                        objectName: "asideAvailableHeading"
                        y: 8 * root.unit
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: root.aside.availableHeading ?? ""
                        size: 14
                        weight: Font.DemiBold
                        spacing: 2.1
                        capitalization: Font.AllUppercase
                        color: ThemeInk.dim
                    }
                }
                Repeater {
                    id: availRep
                    model: root.aside.available ?? []
                    Item {
                        id: availCard
                        required property int index
                        required property var modelData
                        readonly property int asideIdx: instRep.count + availCard.index
                        readonly property bool cardFocused: root.focusZone === 2 && root.focusedAsideIndex === availCard.asideIdx
                        objectName: "asideAvail." + availCard.index
                        width: parent.width
                        height: 84 * root.unit
                        Kit.FocusRing {
                            objectName: "asideAvailRing." + availCard.index
                            visible: availCard.cardFocused
                            anchors.fill: parent
                            gap: 3 * root.unit
                            thickness: 3 * root.unit
                            bandRadius: 24 * root.unit
                            glowBlur: 40 * root.unit
                            glowSpread: 8 * root.unit
                            glowColor: ThemeInk.glowRing
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            objectName: "asideAvailBg." + availCard.index
                            anchors.fill: parent
                            radius: 18 * root.unit
                            color: availCard.cardFocused ? Kit.Tokens.surfaceRaised : ThemeInk.rowSurface
                            border.width: availCard.cardFocused ? 0 : 1 * root.unit
                            border.color: ThemeInk.hairline
                        }
                        Row {
                            anchors.fill: parent
                            anchors.leftMargin: 20 * root.unit
                            anchors.rightMargin: 20 * root.unit
                            spacing: 14 * root.unit
                            LibretroMark {
                                objectName: "asideAvailMark." + availCard.index
                                anchors.verticalCenter: parent.verticalCenter
                                side: 34
                                light: ThemeInk.hairline
                                dark: Kit.Tokens.muted
                            }
                            Column {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width - 34 * root.unit - availRight.width - 28 * root.unit
                                spacing: 2 * root.unit
                                CardLine {
                                    objectName: "asideAvailName." + availCard.index
                                    unit: root.unit
                                    boardLine: true
                                    width: parent.width
                                    text: availCard.modelData.name ?? ""
                                    size: 22
                                    weight: Font.DemiBold
                                    color: Kit.Tokens.textStrong
                                    elide: Text.ElideRight
                                }
                                CardLine {
                                    objectName: "asideAvailDesc." + availCard.index
                                    unit: root.unit
                                    boardLine: true
                                    width: parent.width
                                    text: availCard.modelData.description ?? ""
                                    size: 16
                                    weight: Font.Normal
                                    color: Kit.Tokens.muted
                                    elide: Text.ElideRight
                                }
                            }
                            Row {
                                id: availRight
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 8 * root.unit
                                Item {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 22 * root.unit
                                    height: 22 * root.unit
                                    Shape {
                                        width: 24
                                        height: 24
                                        scale: (22 * root.unit) / 24
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: "transparent"
                                            strokeColor: ThemeInk.body
                                            strokeWidth: 2
                                            capStyle: ShapePath.RoundCap
                                            joinStyle: ShapePath.RoundJoin
                                            PathMove { x: 12; y: 4 }
                                            PathLine { x: 12; y: 15 }
                                            PathMove { x: 7.5; y: 10.5 }
                                            PathLine { x: 12; y: 15 }
                                            PathLine { x: 16.5; y: 10.5 }
                                            PathMove { x: 5; y: 19 }
                                            PathLine { x: 19; y: 19 }
                                        }
                                    }
                                }
                                CardLine {
                                    objectName: "asideAvailSize." + availCard.index
                                    anchors.verticalCenter: parent.verticalCenter
                                    visible: text.length > 0
                                    unit: root.unit
                                    boardLine: true
                                    text: availCard.modelData.size ?? ""
                                    size: 17
                                    weight: Font.Normal
                                    color: ThemeInk.body
                                }
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: {
                                root.tool.eclipseFocusZoneCode(2)
                                root.tool.eclipseFocusCore(availCard.modelData.key)
                                root.tool.eclipseActCode(4, availCard.modelData.key)
                            }
                        }
                    }
                }
                CardLine {
                    objectName: "asideNote"
                    visible: text.length > 0
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 17 * 1.5
                    width: parent.width
                    text: root.aside.note ?? ""
                    size: 17
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                    wrapMode: Text.Wrap
                }
            }
        }
    }

    Item {
        objectName: "coresFooter"
        x: 96 * root.unit
        y: 1122 * root.unit
        width: root.width - 192 * root.unit
        height: 38 * root.unit
        Kit.HelpBar {
            id: helpBar
            objectName: "coresHelp"
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            controller: frame.controller
            family: root.padFamily
            fitted: true
            ruled: false
            color: "transparent"
            labelColor: ThemeInk.body
            iconColor: Kit.Tokens.muted
            leadStyle: Kit.GlyphStyles.Filled
            leadColor: Kit.Tokens.accent
            entrySpacing: 24 * root.unit
            glyphGap: 9 * root.unit
            glyphSize: 34 * root.unit
            snapped: false
            model: RowsModel {
                rows: root.tool.eclipseHelp
            }
            label: Component {
                CardLine {
                    objectName: "helpWords-" + text
                    unit: root.unit
                    boardLine: true
                    size: 19
                }
            }
            glyph: Component {
                CoreHelpGlyph {}
            }
        }
        Rectangle {
            objectName: "coresSummaryPill"
            visible: summaryLine.text.length > 0
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            height: 38 * root.unit
            width: summaryLine.width + 32 * root.unit
            radius: 19 * root.unit
            color: ThemeInk.glassPill
            border.width: 1 * root.unit
            border.color: ThemeInk.hairline
            CardLine {
                id: summaryLine
                objectName: "coresSummary"
                anchors.centerIn: parent
                unit: root.unit
                boardLine: true
                boardLineHeight: 20
                text: root.aside.footerSummary ?? ""
                size: 16
                weight: Font.Normal
                family: Kit.Tokens.faceCode
                color: Kit.Tokens.muted
            }
        }
    }
}
