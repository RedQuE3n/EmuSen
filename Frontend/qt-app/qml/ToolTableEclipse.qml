pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

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
    readonly property var view: JSON.parse(root.tool.eclipseCores)
    readonly property var aside: root.view.aside ?? ({})
    readonly property var colSpecs: root.view.columns ?? []
    readonly property int focusedZone: root.view.focusedZone ?? 1
    readonly property int focusedIndex: root.view.focusedIndex ?? -1
    readonly property int focusedFilterIndex: root.view.focusedFilterIndex ?? 0
    readonly property alias controller: frame.controller
    readonly property bool sheetOpen: frame.controller.sheetOpen

    onFocusedIndexChanged: {
        if (root.focusedIndex < 0 || root.focusedIndex >= rowRep.count) return
        const row = rowRep.itemAt(root.focusedIndex)
        if (!row) return
        const top = row.y
        const bottom = top + row.height
        if (top < tableFlick.contentY) tableFlick.contentY = Math.max(0, top - 8 * root.unit)
        else if (bottom > tableFlick.contentY + tableFlick.height) tableFlick.contentY = Math.max(0, bottom - tableFlick.height + 8 * root.unit)
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
            Kit.BackdropField { x: 1240 * root.unit; y: -420 * root.unit; width: 900 * root.unit; height: 2 * 360 * root.unit; color: "#1FC9A8"; blur: 110 * root.unit; opacity: 0.5 },
            Kit.BackdropField { x: 1500 * root.unit; y: -200 * root.unit; width: 700 * root.unit; height: 600 * root.unit; color: "#6A4CF0"; blur: 120 * root.unit; opacity: 0.55 }
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
        controller.actionLines: root.view.actionLines ?? ""
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
                    model: root.view.trail ?? []
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
                            color: crumbItem.modelData.current ? Kit.Tokens.textStrong : "#B9BED0"
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
                            color: "#8E94AB"
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
            color: "#C707080F"
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
                    text: root.battery + (root.view.percentSuffix ?? "")
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
            text: root.view.title ?? ""
            size: 56
            weight: Font.DemiBold
            spacing: -0.56
            color: Kit.Tokens.textStrong
        }
        CardLine {
            objectName: "toolSubtitle"
            visible: (root.view.subtitle ?? "").length > 0
            width: 1100 * root.unit
            unit: root.unit
            boardLine: true
            boardLineHeight: 22 * 1.45
            text: root.view.subtitle ?? ""
            size: 22
            weight: Font.Light
            color: "#C9CDDB"
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
                    model: root.view.filters ?? []
                    Item {
                        id: pill
                        required property int index
                        required property var modelData
                        objectName: pill.modelData.objectName ?? ""
                        height: 44 * root.unit
                        width: pillLabel.implicitWidth + 40 * root.unit
                        readonly property bool focused: root.focusedZone === 0 && root.focusedFilterIndex === pill.index
                        Kit.FocusRing {
                            visible: pill.focused
                            anchors.fill: pillBox
                            radius: 22 * root.unit
                            gap: 4 * root.unit
                            thickness: 3 * root.unit
                            bandRadius: 29 * root.unit
                            glowColor: "#80F0B4C4"
                            glowBlur: 40 * root.unit
                            glowSpread: 10 * root.unit
                            pixelScale: root.pixelScale
                        }
                        Rectangle {
                            id: pillBox
                            anchors.fill: parent
                            radius: 22 * root.unit
                            color: pill.modelData.active ? Kit.Tokens.accent : "transparent"
                            border.width: pill.modelData.active ? 0 : 2 * root.unit
                            border.color: "#5A6076"
                        }
                        CardLine {
                            id: pillLabel
                            objectName: pill.modelData.labelObjectName ?? ""
                            x: 20 * root.unit
                            y: (parent.height - height) / 2
                            unit: root.unit
                            boardLine: true
                            text: pill.modelData.label ?? ""
                            size: 20
                            weight: pill.modelData.active ? Font.DemiBold : Font.Normal
                            color: pill.modelData.active ? Kit.Tokens.surface : "#D8DBE6"
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
                                color: "#8E94AB"
                            }
                        }
                    }
                }
            }
            Flickable {
                id: tableFlick
                x: 0
                y: 48 * root.unit
                width: tableBlock.width
                height: tableBlock.height - 48 * root.unit
                contentWidth: width
                contentHeight: rowsCol.implicitHeight
                clip: contentHeight > height
                Column {
                    id: rowsCol
                    width: tableFlick.width
                    spacing: 8 * root.unit
                    Repeater {
                        id: rowRep
                        model: root.view.rows ?? []
                        Item {
                            id: rowItem
                            required property int index
                            required property var modelData
                            readonly property var cellNames: rowItem.modelData.cellObjectNames ?? []
                            objectName: rowItem.modelData.objectName ?? ""
                            width: rowsCol.width
                            height: Math.max(66 * root.unit, cellsRow.implicitHeight + 20 * root.unit)
                            Kit.FocusRing {
                                objectName: rowItem.modelData.ringObjectName ?? ""
                                visible: rowItem.modelData.focused === true
                                anchors.fill: rowBg
                                radius: 16 * root.unit
                                gap: 4 * root.unit
                                thickness: 3 * root.unit
                                bandRadius: 23 * root.unit
                                glowColor: "#80F0B4C4"
                                glowBlur: 40 * root.unit
                                glowSpread: 10 * root.unit
                                pixelScale: root.pixelScale
                            }
                            Rectangle {
                                id: rowBg
                                objectName: rowItem.modelData.bgObjectName ?? ""
                                anchors.fill: parent
                                radius: 16 * root.unit
                                color: rowItem.modelData.focused ? "#151826" : "#11131C"
                                border.width: rowItem.modelData.focused ? 0 : 1 * root.unit
                                border.color: "#262A3A"
                            }
                            Row {
                                id: cellsRow
                                x: 20 * root.unit
                                y: (parent.height - height) / 2
                                spacing: 16 * root.unit
                                Repeater {
                                    model: rowItem.modelData.cells ?? []
                                    CardLine {
                                        id: cellText
                                        required property int index
                                        required property string modelData
                                        objectName: rowItem.cellNames[cellText.index] ?? ""
                                        y: (cellsRow.height - height) / 2
                                        width: (root.colSpecs[cellText.index]?.width ?? 100) * root.unit
                                        unit: root.unit
                                        boardLine: true
                                        boardLineHeight: 26
                                        lineHeight: 26
                                        lineHeightMode: Text.FixedHeight
                                        wrapMode: Text.WordWrap
                                        text: cellText.modelData
                                        size: 20
                                        weight: Font.Normal
                                        color: Kit.Tokens.textStrong
                                    }
                                }
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: root.tool.eclipseActCode(3, rowItem.modelData.key ?? "")
                                onDoubleClicked: root.tool.eclipseNav(4)
                            }
                        }
                    }
                    CardLine {
                        objectName: "tableEmpty"
                        visible: rowRep.count === 0
                        width: rowsCol.width
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 29
                        text: root.view.emptyMessage ?? ""
                        size: 20
                        weight: Font.Normal
                        color: "#B9BED0"
                    }
                }
            }
        }

        Rectangle {
            objectName: root.view.asideObjectName ?? ""
            x: 1300 * root.unit
            y: 276 * root.unit
            width: 524 * root.unit
            height: asideCol.implicitHeight + 64 * root.unit
            radius: 28 * root.unit
            color: "#11131C"
            border.width: 1 * root.unit
            border.color: "#262A3A"
            Column {
                id: asideCol
                x: 32 * root.unit
                y: 32 * root.unit
                width: 460 * root.unit
                spacing: 16 * root.unit
                CardLine {
                    objectName: root.view.asideTitleObjectName ?? ""
                    width: asideCol.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 36
                    lineHeight: 36
                    lineHeightMode: Text.FixedHeight
                    wrapMode: Text.WordWrap
                    text: root.aside.title ?? ""
                    size: 30
                    weight: Font.DemiBold
                    color: Kit.Tokens.textStrong
                }
                CardLine {
                    objectName: root.view.asideSubObjectName ?? ""
                    visible: (root.aside.subtitle ?? "").length > 0
                    width: asideCol.width
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 29
                    lineHeight: 29
                    lineHeightMode: Text.FixedHeight
                    wrapMode: Text.WordWrap
                    text: root.aside.subtitle ?? ""
                    size: 20
                    weight: Font.Normal
                    color: root.aside.subtitleMuted ? "#B9BED0" : "#D8DBE6"
                }
                Loader {
                    id: asideMediaLoader
                    active: root.asideMediaSlot !== null
                    visible: active
                    width: asideCol.width
                    sourceComponent: root.asideMediaSlot
                }
                Repeater {
                    model: root.aside.lines ?? []
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
                        color: asideLine.modelData.muted ? "#B9BED0" : "#D8DBE6"
                    }
                }
            }
        }

        Rectangle {
            objectName: root.view.markBarObjectName ?? ""
            x: 96 * root.unit
            y: 1032 * root.unit
            width: root.width - 192 * root.unit
            height: 64 * root.unit
            radius: 20 * root.unit
            color: "#151826"
            border.width: 1 * root.unit
            border.color: "#262A3A"
            CardLine {
                objectName: root.view.markBarTextObjectName ?? ""
                x: 24 * root.unit
                y: (parent.height - height) / 2
                unit: root.unit
                boardLine: true
                text: root.view.markBarText ?? ""
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
        objectName: root.view.footerObjectName ?? ""
        x: 96 * root.unit
        y: 1122 * root.unit
        width: root.width - 192 * root.unit
        height: 38 * root.unit
        Kit.HelpBar {
            id: footerHelpBar
            objectName: root.view.helpObjectName ?? ""
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter
            controller: frame.controller
            family: root.padFamily
            fitted: true
            ruled: false
            color: "transparent"
            labelColor: "#D8DBE6"
            iconColor: Kit.Tokens.muted
            leadStyle: Kit.GlyphStyles.Filled
            leadColor: Kit.Tokens.accent
            entrySpacing: 34 * root.unit
            glyphGap: 10 * root.unit
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
                    size: 21
                }
            }
            glyph: Component {
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
                    ink: "#D8DBE6"
                    ring: "#B9BED0"
                }
            }
        }
        CardLine {
            objectName: root.view.counterObjectName ?? ""
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            unit: root.unit
            boardLine: true
            text: root.view.counter ?? ""
            size: 21
            weight: Font.Normal
            color: "#B9BED0"
        }
    }
}
