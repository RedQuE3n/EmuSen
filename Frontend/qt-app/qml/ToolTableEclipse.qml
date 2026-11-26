pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Shared Eclipse scaffold for Scraper (#66), Scrape Status (#67), and Library Manager (#68).
FocusScope {
    id: root
    required property ToolModel tool
    required property int padFamily
    property real pixelScale: 1
    property int battery: -1
    property string clockText: ""
    property bool tableShown: true
    property int screenCode: 0
    property Component asideMediaSlot: null
    property Component markBarSlot: null
    property Component bodySlot: null
    signal backed()
    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    readonly property bool screenShown: root.screenCode === 0 || root.tool.eclipseScreen === root.screenCode
    readonly property var view: root.screenShown ? root.tool.eclipseState : ({})
    readonly property var aside: root.view?.aside ?? ({})
    readonly property var colSpecs: root.view?.columns ?? null
    readonly property int focusedZone: root.screenShown ? root.tool.eclipseFocusedZone : 1
    readonly property int focusedIndex: root.screenShown ? root.tool.eclipseFocusedIndex : -1
    readonly property int focusedFilterIndex: root.screenShown ? root.tool.eclipseFilterIndex : 0
    readonly property alias controller: frame.controller
    readonly property bool sheetOpen: frame.controller.sheetOpen
    readonly property bool isLibrary: root.screenCode === 4
    readonly property bool isStructuredAside: root.aside?.structured === true

    EclipseListModel {
        id: rowsModel
        rows: root.screenShown ? root.tool.eclipseRowsJson : ""
    }

    component TableHelpGlyph: Item {
        id: tg
        property int button: Kit.Pad.Accept
        property int family: root.padFamily
        property bool lead: false
        property int pair: -1
        readonly property bool drawsPair: tg.button === Kit.Pad.Left || tg.button === Kit.Pad.PageUp
        readonly property bool pairedShoulders: tg.button === Kit.Pad.PageUp && tg.pair >= 0
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
        implicitWidth: tgRow.implicitWidth
        implicitHeight: tgRow.implicitHeight
        width: implicitWidth
        height: implicitHeight
        Row {
            id: tgRow
            spacing: 5 * root.unit
            CardGlyph {
                id: primaryGlyph
                anchors.verticalCenter: parent.verticalCenter
                unit: root.unit
                button: tg.button
                family: tg.family
                lead: tg.lead
                pair: tg.button === Kit.Pad.Left ? tg.pair : -1
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
                visible: tg.pairedShoulders
                unit: root.unit
                button: tg.pair >= 0 ? tg.pair : Kit.Pad.PageDown
                family: tg.family
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

    Component {
        id: defaultHelpGlyph
        CardGlyph {
            unit: root.unit
            side: 34
            boxSide: 30
            boxRadius: 8
            boxPadding: 10
            wordSize: 15
            padding: 12
            boxMenu: true
            boxOptions: true
            keyPairs: true
            ink: ThemeInk.body
            ring: Kit.Tokens.muted
        }
    }

    Component {
        id: libraryHelpGlyph
        TableHelpGlyph {}
    }

    onFocusedIndexChanged: {
        if (root.focusedIndex >= 0 && root.focusedIndex < rowsModel.count)
            tableFlick.positionViewAtIndex(root.focusedIndex, ListView.Contain)
    }

    onFocusedFilterIndexChanged: {
        if (root.focusedFilterIndex < 0 || root.focusedFilterIndex >= filterRep.count) return
        const pill = filterRep.itemAt(root.focusedFilterIndex)
        if (!pill) return
        const left = pill.x
        const right = left + pill.width
        if (left < filterFlick.contentX) filterFlick.contentX = left
        else if (right > filterFlick.contentX + filterFlick.width) filterFlick.contentX = right - filterFlick.width
    }

    Binding {
        target: frame.controller
        property: "sheetOpen"
        value: false
        when: root.screenCode > 0 && root.tool.eclipseScreen !== root.screenCode
        restoreMode: Binding.RestoreNone
    }

    Kit.Backdrop {
        objectName: "toolBackdrop"
        anchors.fill: parent
        ground: Kit.Tokens.surface
        pixelScale: root.pixelScale
        fields: [
            Kit.BackdropField { x: 1240 * root.unit; y: -420 * root.unit; width: 900 * root.unit; height: 2 * 360 * root.unit; color: Kit.Tokens.success; blur: 110 * root.unit; opacity: 0.5 },
            Kit.BackdropField { x: 1500 * root.unit; y: -200 * root.unit; width: 700 * root.unit; height: 600 * root.unit; color: ThemeInk.backdropHalo; blur: 120 * root.unit; opacity: 0.55 }
        ]
    }

    Kit.Frame {
        id: frame
        objectName: "toolEclipseFrame"
        anchors.fill: parent
        focus: true
        form: Kit.Forms.BigScreen
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        controller.navigationShown: false
        controller.actionLines: root.view?.actionLines ?? ""
        Connections {
            target: frame.controller
            function onBackDeclined() { root.backed() }
            function onActionInvoked(key) { root.tool.eclipseActionKey(key) }
        }
        FocusScope {
            anchors.fill: parent
            focus: true
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Up) { root.tool.eclipseNav(0); event.accepted = true }
                else if (event.key === Qt.Key_Down) { root.tool.eclipseNav(1); event.accepted = true }
                else if (event.key === Qt.Key_Left) { root.tool.eclipseNav(2); event.accepted = true }
                else if (event.key === Qt.Key_Right) { root.tool.eclipseNav(3); event.accepted = true }
                else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) { root.tool.eclipseNav(4); event.accepted = true }
                else if (event.key === Qt.Key_PageUp) { root.tool.eclipseNav(5); event.accepted = true }
                else if (event.key === Qt.Key_PageDown) { root.tool.eclipseNav(6); event.accepted = true }
            }
        }
    }

    Item {
        objectName: "toolHeader"
        x: 96 * root.unit
        y: 48 * root.unit
        width: root.width - 192 * root.unit
        height: 64 * root.unit
        Row {
            id: leftHdrRow
            x: 0
            height: 34 * root.unit
            y: (parent.height - height) / 2
            spacing: 28 * root.unit
            Item {
                objectName: "toolMark"
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
                objectName: "toolTrail"
                height: parent.height
                y: (parent.height - height) / 2
                spacing: 14 * root.unit
                Repeater {
                    model: root.view?.trail ?? null
                    Row {
                        id: crumbItem
                        required property var modelData
                        height: leftHdrRow.height
                        spacing: 14 * root.unit
                        CardLine {
                            objectName: crumbItem.modelData.objectName ?? ""
                            y: (crumbItem.height - height) / 2
                            unit: root.unit
                            boardLine: true
                            text: crumbItem.modelData.label ?? ""
                            size: 24
                            weight: crumbItem.modelData.current ? Font.DemiBold : Font.Normal
                            color: crumbItem.modelData.current ? Kit.Tokens.textStrong : Kit.Tokens.muted
                            MouseArea {
                                visible: !crumbItem.modelData.current
                                anchors.fill: parent
                                onClicked: root.backed()
                            }
                        }
                        CardLine {
                            objectName: crumbItem.modelData.slashObjectName ?? ""
                            visible: (crumbItem.modelData.slashObjectName ?? "").length > 0
                            y: (crumbItem.height - height) / 2
                            unit: root.unit
                            boardLine: true
                            text: crumbItem.modelData.slash ?? ""
                            size: 24
                            weight: Font.Normal
                            color: ThemeInk.dim
                        }
                    }
                }
            }
        }
        Rectangle {
            objectName: "statusPill"
            x: parent.width - width
            y: (parent.height - height) / 2
            visible: root.battery >= 0 || root.clockText.length > 0
            height: 56 * root.unit
            width: statusPillRow.implicitWidth + 44 * root.unit
            radius: 28 * root.unit
            color: ThemeInk.glass
            Row {
                id: statusPillRow
                x: 22 * root.unit
                y: 0
                height: parent.height
                spacing: 22 * root.unit
                CardLine {
                    objectName: "battery"
                    visible: root.battery >= 0
                    y: (statusPillRow.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.battery + (root.view?.percentSuffix ?? "")
                    size: 22
                    weight: Font.Normal
                    color: Kit.Tokens.textStrong
                }
                CardLine {
                    objectName: "clock"
                    visible: root.clockText.length > 0
                    y: (statusPillRow.height - height) / 2
                    unit: root.unit
                    boardLine: true
                    text: root.clockText
                    size: 22
                    weight: Font.DemiBold
                    color: Kit.Tokens.textStrong
                }
            }
        }
    }

    Column {
        objectName: "toolHeading"
        x: 96 * root.unit
        y: 150 * root.unit
        spacing: 10 * root.unit
        CardLine {
            objectName: "toolTitle"
            unit: root.unit
            boardLine: true
            boardLineHeight: 56 * 1.06
            text: root.view?.title ?? ""
            size: 56
            weight: Font.DemiBold
            spacing: -0.56
            color: Kit.Tokens.textStrong
        }
        CardLine {
            objectName: "toolSubtitle"
            visible: (root.view?.subtitle ?? "").length > 0
            width: 1100 * root.unit
            unit: root.unit
            boardLine: true
            boardLineHeight: 22 * 1.45
            text: root.view?.subtitle ?? ""
            size: 22
            weight: Font.Light
            color: ThemeInk.body
            elide: Text.ElideRight
        }
    }

    Item {
        visible: root.tableShown
        anchors.fill: parent
        Flickable {
            id: filterFlick
            objectName: "toolFilters"
            x: 96 * root.unit
            y: 276 * root.unit
            width: 1160 * root.unit
            height: 44 * root.unit
            contentWidth: filterRow.implicitWidth
            contentHeight: height
            clip: contentWidth > width
            Row {
                id: filterRow
                spacing: 10 * root.unit
                Repeater {
                    id: filterRep
                    model: root.view?.filters ?? null
                    Item {
                        id: pill
                        required property int index
                        required property var modelData
                        readonly property bool hasCount: (pill.modelData.countText ?? "").length > 0
                        objectName: pill.modelData.objectName ?? ""
                        height: 44 * root.unit
                        width: pillRow.implicitWidth + (pill.hasCount ? 34 : 40) * root.unit
                        readonly property bool focused: root.focusedZone === 0 && root.focusedFilterIndex === pill.index
                        Kit.FocusRing {
                            visible: pill.focused
                            anchors.fill: pillBox
                            radius: 22 * root.unit
                            gap: 4 * root.unit
                            thickness: 3 * root.unit
                            bandRadius: 29 * root.unit
                            glowColor: ThemeInk.glowStrong
                            glowBlur: 40 * root.unit
                            glowSpread: 10 * root.unit
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            id: pillBox
                            anchors.fill: parent
                            radius: 22 * root.unit
                            color: pill.modelData.active ? Kit.Tokens.accent : "transparent"
                            border.width: pill.modelData.active ? 0 : (pill.hasCount ? 1.5 : 2) * root.unit
                            border.color: ThemeInk.pillBorder
                        }
                        Row {
                            id: pillRow
                            x: (pill.hasCount ? 17 : 20) * root.unit
                            y: (pill.height - pillRow.height) / 2
                            spacing: 8 * root.unit
                            CardLine {
                                id: pillLabel
                                objectName: pill.modelData.labelObjectName ?? ""
                                y: (pillRow.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                boardLineHeight: pill.hasCount ? 22 : 0
                                text: pill.modelData.label ?? ""
                                size: pill.hasCount ? 17 : 20
                                weight: pill.modelData.active ? Font.DemiBold : Font.Normal
                                color: pill.modelData.active ? Kit.Tokens.overAccent : ThemeInk.body
                            }
                            CardLine {
                                objectName: pill.modelData.countObjectName ?? ""
                                visible: pill.hasCount
                                y: (pillRow.height - height) / 2
                                unit: root.unit
                                boardLine: true
                                boardLineHeight: 18
                                text: pill.modelData.countText ?? ""
                                size: 14
                                weight: Font.Normal
                                family: Kit.Tokens.faceCode
                                color: pill.modelData.active ? ThemeInk.overAccentMuted : ThemeInk.dim
                            }
                        }
                        MouseArea {
                            anchors.fill: parent
                            onClicked: root.tool.eclipseChooseFilter(pill.modelData.key ?? "")
                        }
                    }
                }
            }
        }

        Item {
            id: tableBlock
            objectName: "toolTable"
            x: 96 * root.unit
            y: 340 * root.unit
            width: 1160 * root.unit
            height: 672 * root.unit
            Item {
                objectName: "tableHeader"
                x: 0
                y: 0
                width: tableBlock.width
                height: 40 * root.unit
                Row {
                    x: 20 * root.unit
                    height: parent.height
                    spacing: 16 * root.unit
                    Repeater {
                        model: root.colSpecs
                        Item {
                            id: hdrCol
                            required property var modelData
                            width: (hdrCol.modelData.width ?? 100) * root.unit
                            height: parent.height
                            CardLine {
                                objectName: hdrCol.modelData.objectName ?? ""
                                visible: (hdrCol.modelData.title ?? "").length > 0
                                y: (parent.height - height) / 2
                                width: parent.width
                                unit: root.unit
                                boardLine: true
                                text: hdrCol.modelData.title ?? ""
                                size: 16
                                weight: Font.Normal
                                spacing: 2.24
                                capitalization: Font.AllUppercase
                                color: ThemeInk.dim
                            }
                        }
                    }
                }
            }
            ListView {
                id: tableFlick
                x: 0
                y: 48 * root.unit
                width: tableBlock.width
                height: tableBlock.height - 48 * root.unit
                spacing: 8 * root.unit
                reuseItems: true
                clip: contentHeight > height
                boundsBehavior: Flickable.StopAtBounds
                model: rowsModel
                delegate: Item {
                    id: rowItem
                    required property int index
                    required property var model
                    readonly property var itemData: rowItem.model.item
                    readonly property bool rowFocused: root.focusedZone === 1 && rowItem.index === root.focusedIndex
                    readonly property var cellNames: rowItem.itemData.cellObjectNames ?? null
                    objectName: rowItem.itemData.objectName ?? ""
                    width: tableFlick.width
                    height: Math.max(66 * root.unit, cellsRow.implicitHeight + 20 * root.unit)
                    Kit.FocusRing {
                        objectName: rowItem.itemData.ringObjectName ?? ""
                        visible: rowItem.rowFocused
                        anchors.fill: rowBg
                        radius: 16 * root.unit
                        gap: 4 * root.unit
                        thickness: 3 * root.unit
                        bandRadius: 23 * root.unit
                        glowColor: ThemeInk.glowStrong
                        glowBlur: 40 * root.unit
                        glowSpread: 10 * root.unit
                        pixelScale: root.pixelScale
                    }
                    Rectangle {
                        id: rowBg
                        objectName: rowItem.itemData.bgObjectName ?? ""
                        anchors.fill: parent
                        radius: 16 * root.unit
                        color: rowItem.rowFocused ? Kit.Tokens.surfaceRaised : Kit.Tokens.inputSurface
                        border.width: rowItem.rowFocused ? 0 : 1 * root.unit
                        border.color: ThemeInk.hairline
                    }
                    Row {
                        id: cellsRow
                        x: 20 * root.unit
                        y: (parent.height - height) / 2
                        spacing: 16 * root.unit
                        Repeater {
                            model: rowItem.itemData.cells ?? null
                            Item {
                                id: cellWrap
                                required property int index
                                required property string modelData
                                readonly property real colW: (root.colSpecs && root.colSpecs[cellWrap.index] ? (root.colSpecs[cellWrap.index].width ?? 100) : 100) * root.unit
                                readonly property bool isMono: root.isLibrary && (rowItem.itemData.cellMono ? rowItem.itemData.cellMono[cellWrap.index] === true : false)
                                readonly property string subLine: root.isLibrary && rowItem.itemData.cellSubs ? (rowItem.itemData.cellSubs[cellWrap.index] ?? "") : ""
                                readonly property var chipData: rowItem.itemData.statusChip ?? null
                                readonly property bool hasChip: root.isLibrary && cellWrap.index === 5 && ((cellWrap.chipData?.text ?? "").length > 0)
                                width: cellWrap.colW
                                height: cellWrap.hasChip ? 26 * root.unit : cellCol.height
                                y: (cellsRow.height - height) / 2
                                Column {
                                    id: cellCol
                                    visible: !cellWrap.hasChip
                                    width: parent.width
                                    spacing: 2 * root.unit
                                    CardLine {
                                        id: cellText
                                        objectName: rowItem.cellNames ? (rowItem.cellNames[cellWrap.index] ?? "") : ""
                                        width: parent.width
                                        unit: root.unit
                                        boardLine: true
                                        boardLineHeight: root.isLibrary ? 22 : 26
                                        lineHeight: root.isLibrary ? 22 : 26
                                        lineHeightMode: Text.FixedHeight
                                        wrapMode: cellWrap.isMono ? Text.NoWrap : Text.WordWrap
                                        elide: cellWrap.isMono ? Text.ElideRight : Text.ElideNone
                                        text: cellWrap.modelData
                                        size: root.isLibrary ? (cellWrap.isMono ? 16 : 17) : 20
                                        weight: root.isLibrary && cellWrap.isMono && cellWrap.index === 0 ? Font.DemiBold : Font.Normal
                                        family: cellWrap.isMono ? Kit.Tokens.faceCode : Kit.Tokens.faceText
                                        color: root.isLibrary && cellWrap.index === 3 ? Kit.Tokens.accent : (root.isLibrary && cellWrap.index === 1 ? Kit.Tokens.muted : Kit.Tokens.textStrong)
                                        horizontalAlignment: root.isLibrary && cellWrap.index === 4 ? Text.AlignRight : Text.AlignLeft
                                    }
                                    CardLine {
                                        visible: cellWrap.subLine.length > 0
                                        width: parent.width
                                        unit: root.unit
                                        boardLine: true
                                        text: cellWrap.subLine
                                        size: 14
                                        weight: Font.Normal
                                        color: Kit.Tokens.muted
                                        elide: Text.ElideRight
                                    }
                                }
                                Rectangle {
                                    visible: cellWrap.hasChip
                                    anchors.right: parent.right
                                    anchors.verticalCenter: parent.verticalCenter
                                    height: 26 * root.unit
                                    width: chipLine.width + 20 * root.unit
                                    radius: 13 * root.unit
                                    readonly property int toneCode: cellWrap.chipData?.toneCode ?? 0
                                    color: toneCode === 1 ? ThemeInk.errorFill : (toneCode === 2 ? ThemeInk.attentionFill : Kit.Tokens.surface)
                                    border.width: 1 * root.unit
                                    border.color: toneCode === 1 ? ThemeInk.rose : (toneCode === 2 ? ThemeInk.accentSoft : ThemeInk.pillBorder)
                                    CardLine {
                                        id: chipLine
                                        objectName: "tableChip." + rowItem.index
                                        anchors.centerIn: parent
                                        unit: root.unit
                                        boardLine: true
                                        text: cellWrap.chipData?.text ?? ""
                                        size: 13
                                        weight: Font.DemiBold
                                        color: (cellWrap.chipData?.toneCode ?? 0) === 1 ? ThemeInk.rose : ((cellWrap.chipData?.toneCode ?? 0) === 2 ? Kit.Tokens.accent : Kit.Tokens.muted)
                                    }
                                }
                            }
                        }
                    }
                    MouseArea {
                        anchors.fill: parent
                        onClicked: root.tool.eclipseActCode(3, rowItem.itemData.key ?? "")
                        onDoubleClicked: root.tool.eclipseNav(4)
                    }
                }
                CardLine {
                    objectName: "tableEmpty"
                    visible: rowsModel.count === 0
                    width: tableFlick.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 29
                    text: root.view?.emptyMessage ?? ""
                    size: 20
                    weight: Font.Normal
                    color: Kit.Tokens.muted
                }
            }
        }

        Rectangle {
            visible: !root.isStructuredAside
            objectName: !root.isStructuredAside ? (root.view?.asideObjectName ?? "") : ""
            x: 1300 * root.unit
            y: 276 * root.unit
            width: 524 * root.unit
            height: asideCol.implicitHeight + 64 * root.unit
            radius: 28 * root.unit
            color: Kit.Tokens.inputSurface
            border.width: 1 * root.unit
            border.color: ThemeInk.hairline
            Column {
                id: asideCol
                x: 32 * root.unit
                y: 32 * root.unit
                width: 460 * root.unit
                spacing: 16 * root.unit
                CardLine {
                    objectName: !root.isStructuredAside ? (root.view?.asideTitleObjectName ?? "") : ""
                    width: asideCol.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 36
                    lineHeight: 36
                    lineHeightMode: Text.FixedHeight
                    wrapMode: Text.WordWrap
                    text: root.aside?.title ?? ""
                    size: 30
                    weight: Font.DemiBold
                    color: Kit.Tokens.textStrong
                }
                CardLine {
                    objectName: !root.isStructuredAside ? (root.view?.asideSubObjectName ?? "") : ""
                    visible: (root.aside?.subtitle ?? "").length > 0
                    width: asideCol.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 29
                    lineHeight: 29
                    lineHeightMode: Text.FixedHeight
                    wrapMode: Text.WordWrap
                    text: root.aside?.subtitle ?? ""
                    size: 20
                    weight: Font.Normal
                    color: root.aside?.subtitleMuted ? Kit.Tokens.muted : ThemeInk.body
                }
                Loader {
                    id: asideMediaLoader
                    active: root.asideMediaSlot !== null
                    visible: active
                    width: asideCol.width
                    sourceComponent: root.asideMediaSlot
                }
                Repeater {
                    model: root.isStructuredAside ? null : (root.aside?.lines ?? null)
                    CardLine {
                        id: asideLine
                        required property var modelData
                        objectName: asideLine.modelData.objectName ?? ""
                        width: asideCol.width
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 29
                        lineHeight: 29
                        lineHeightMode: Text.FixedHeight
                        wrapMode: Text.WordWrap
                        text: asideLine.modelData.text ?? ""
                        size: 20
                        weight: Font.Normal
                        color: asideLine.modelData.muted ? Kit.Tokens.muted : ThemeInk.body
                    }
                }
            }
        }

        Column {
            visible: root.isStructuredAside
            x: 1300 * root.unit
            y: 276 * root.unit
            width: 524 * root.unit
            spacing: 16 * root.unit
            Rectangle {
                objectName: root.isStructuredAside ? (root.view?.asideObjectName ?? "") : ""
                width: parent.width
                height: diagCol.implicitHeight + 52 * root.unit
                radius: 22 * root.unit
                color: Kit.Tokens.inputSurface
                border.width: 1 * root.unit
                border.color: ThemeInk.hairline
                Column {
                    id: diagCol
                    x: 28 * root.unit
                    y: 26 * root.unit
                    width: parent.width - 56 * root.unit
                    spacing: 14 * root.unit
                    Column {
                        width: parent.width
                        spacing: 4 * root.unit
                        CardLine {
                            objectName: "asideEyebrow"
                            width: parent.width
                            unit: root.unit
                            boardLine: true
                            text: root.aside?.eyebrow ?? ""
                            size: 13
                            weight: Font.DemiBold
                            spacing: 2.08
                            capitalization: Font.AllUppercase
                            color: (root.aside?.eyebrowToneCode ?? 0) === 1 ? ThemeInk.rose : ((root.aside?.eyebrowToneCode ?? 0) === 2 ? Kit.Tokens.accent : Kit.Tokens.muted)
                        }
                        CardLine {
                            objectName: root.isStructuredAside ? (root.view?.asideTitleObjectName ?? "") : ""
                            width: parent.width
                            unit: root.unit
                            boardLine: true
                            boardLineHeight: 28
                            lineHeight: 28
                            lineHeightMode: Text.FixedHeight
                            wrapMode: Text.WordWrap
                            text: root.aside?.title ?? ""
                            size: 23
                            weight: Font.Bold
                            family: Kit.Tokens.faceCode
                            color: Kit.Tokens.textStrong
                        }
                    }
                    Rectangle {
                        objectName: "asidePathBox"
                        width: parent.width
                        height: diagPath.implicitHeight + 22 * root.unit
                        radius: 12 * root.unit
                        color: Kit.Tokens.surface
                        border.width: 1 * root.unit
                        border.color: ThemeInk.hairline
                        CardLine {
                            id: diagPath
                            objectName: root.isStructuredAside ? (root.view?.asideSubObjectName ?? "") : ""
                            x: 15 * root.unit
                            y: 11 * root.unit
                            width: parent.width - 30 * root.unit
                            unit: root.unit
                            boardLine: true
                            boardLineHeight: 19
                            lineHeight: 19
                            lineHeightMode: Text.FixedHeight
                            wrapMode: Text.WrapAnywhere
                            text: root.aside?.subtitle ?? ""
                            size: 14
                            weight: Font.Normal
                            family: Kit.Tokens.faceCode
                            color: Kit.Tokens.accent
                        }
                    }
                    Rectangle {
                        objectName: "asideDiagBanner"
                        width: parent.width
                        height: diagWords.implicitHeight + 26 * root.unit
                        radius: 14 * root.unit
                        readonly property int toneCode: root.aside?.eyebrowToneCode ?? 0
                        color: toneCode === 1 ? ThemeInk.errorFill : (toneCode === 2 ? ThemeInk.attentionFill : Kit.Tokens.surfaceRaised)
                        border.width: 1 * root.unit
                        border.color: toneCode === 1 ? ThemeInk.rose : (toneCode === 2 ? ThemeInk.accentSoft : ThemeInk.hairline)
                        CardLine {
                            id: diagWords
                            objectName: root.isStructuredAside ? (root.aside?.diagObjectName ?? "") : ""
                            x: 16 * root.unit
                            y: 13 * root.unit
                            width: parent.width - 32 * root.unit
                            unit: root.unit
                            boardLine: true
                            boardLineHeight: 22
                            lineHeight: 22
                            lineHeightMode: Text.FixedHeight
                            wrapMode: Text.WordWrap
                            text: root.aside?.diagnostic ?? ""
                            size: 15
                            weight: Font.Normal
                            color: Kit.Tokens.textStrong
                        }
                    }
                    Rectangle {
                        width: parent.width
                        height: 1 * root.unit
                        color: ThemeInk.hairline
                    }
                    Column {
                        objectName: "asideFactsList"
                        width: parent.width
                        spacing: 10 * root.unit
                        Repeater {
                            model: root.aside?.facts ?? null
                            Row {
                                id: factRow
                                required property var modelData
                                width: parent.width
                                spacing: 10 * root.unit
                                CardLine {
                                    width: 110 * root.unit
                                    y: (factRow.height - height) / 2
                                    unit: root.unit
                                    boardLine: true
                                    text: factRow.modelData.label ?? ""
                                    size: 15
                                    color: ThemeInk.dim
                                }
                                Row {
                                    id: factValRow
                                    width: parent.width - 120 * root.unit
                                    y: (factRow.height - height) / 2
                                    spacing: 8 * root.unit
                                    CardPlanet {
                                        visible: (factRow.modelData.planet ?? -3) >= 0
                                        y: (factValRow.height - height) / 2
                                        unit: root.unit
                                        planet: factRow.modelData.planet ?? 0
                                        side: 14
                                        glowBlur: 6
                                        glowSpread: 1
                                        pixelScale: root.pixelScale
                                    }
                                    CardLine {
                                        width: parent.width - ((factRow.modelData.planet ?? -3) >= 0 ? 22 * root.unit : 0)
                                        y: (factValRow.height - height) / 2
                                        unit: root.unit
                                        boardLine: true
                                        text: factRow.modelData.value ?? ""
                                        size: 15
                                        family: factRow.modelData.mono ? Kit.Tokens.faceCode : Kit.Tokens.faceText
                                        color: Kit.Tokens.textStrong
                                        elide: Text.ElideRight
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Rectangle {
                objectName: "asideContractCard"
                width: parent.width
                height: contractCol.implicitHeight + 40 * root.unit
                radius: 20 * root.unit
                color: Kit.Tokens.surfaceRaised
                border.width: 1 * root.unit
                border.color: ThemeInk.hairline
                Column {
                    id: contractCol
                    x: 24 * root.unit
                    y: 20 * root.unit
                    width: parent.width - 48 * root.unit
                    spacing: 6 * root.unit
                    CardLine {
                        objectName: "asideContractHeading"
                        width: parent.width
                        unit: root.unit
                        boardLine: true
                        text: root.aside?.contractHeading ?? ""
                        size: 16
                        weight: Font.DemiBold
                        color: Kit.Tokens.textStrong
                    }
                    CardLine {
                        objectName: "asideContractBody"
                        width: parent.width
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 21
                        lineHeight: 21
                        lineHeightMode: Text.FixedHeight
                        wrapMode: Text.WordWrap
                        text: root.aside?.contractBody ?? ""
                        size: 14
                        weight: Font.Normal
                        color: ThemeInk.body
                    }
                }
            }
        }

        Rectangle {
            objectName: root.view?.markBarObjectName ?? ""
            x: 96 * root.unit
            y: 1032 * root.unit
            width: root.width - 192 * root.unit
            height: 64 * root.unit
            radius: 20 * root.unit
            color: Kit.Tokens.surfaceRaised
            border.width: 1 * root.unit
            border.color: ThemeInk.hairline
            CardLine {
                objectName: root.view?.markBarTextObjectName ?? ""
                x: 24 * root.unit
                y: (parent.height - height) / 2
                unit: root.unit
                boardLine: true
                text: root.view?.markBarText ?? ""
                size: 21
                weight: Font.Normal
                color: Kit.Tokens.textStrong
            }
            Loader {
                id: markBarLoader
                x: parent.width - width - 24 * root.unit
                y: (parent.height - height) / 2
                active: root.markBarSlot !== null
                visible: active
                sourceComponent: root.markBarSlot
            }
        }
    }

    Loader {
        anchors.fill: parent
        active: !root.tableShown && root.bodySlot !== null
        visible: active
        sourceComponent: root.bodySlot
    }

    Item {
        objectName: root.view?.footerObjectName ?? ""
        x: 96 * root.unit
        y: 1122 * root.unit
        width: root.width - 192 * root.unit
        height: 38 * root.unit
        Kit.HelpBar {
            id: footerHelpBar
            objectName: root.view?.helpObjectName ?? ""
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
            entrySpacing: (root.isLibrary ? 24 : 34) * root.unit
            glyphGap: (root.isLibrary ? 9 : 10) * root.unit
            glyphSize: (root.isLibrary ? 34 : 34) * root.unit
            snapped: false
            model: RowsModel {
                rows: root.screenShown ? root.tool.eclipseHelp : ""
            }
            label: Component {
                CardLine {
                    objectName: "helpWords-" + text
                    unit: root.unit
                    boardLine: true
                    size: root.isLibrary ? 19 : 21
                }
            }
            glyph: root.isLibrary ? libraryHelpGlyph : defaultHelpGlyph
        }
        CardLine {
            objectName: root.view?.counterObjectName ?? ""
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            unit: root.unit
            boardLine: true
            text: root.screenShown ? root.tool.eclipseCounter : ""
            size: 21
            weight: Font.Normal
            color: Kit.Tokens.muted
        }
    }
}
