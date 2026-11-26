pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Reference Desktop Mode Library Table screen: sortable high-density catalog rows and multi-select batch action bar.
Item {
    id: root
    objectName: "desktopTable"
    required property DesktopModel model
    required property var chrome
    required property var consoles
    required property var games
    required property real px

    readonly property bool isFiltered: root.model.filterKey !== (root.chrome.allKey ?? "") || root.model.searchQuery.length > 0
    readonly property real rowHeight: Math.min(48 * root.px, Math.max(36 * root.px, Kit.Tokens.rowHeight * root.px))

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Collapsible left views & consoles rail.
        Rectangle {
            visible: root.model.sidebarOpen
            Layout.fillHeight: true
            Layout.preferredWidth: Math.round(236 * root.px)
            color: ThemeInk.glass
            border.width: 1
            border.color: ThemeInk.hairline

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Math.round(14 * root.px)
                spacing: Math.round(10 * root.px)

                Text {
                    text: root.chrome.constellationHeading ?? ""
                    color: ThemeInk.muted
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Kit.Tokens.weightWordmark
                    font.letterSpacing: 1.1
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(3 * root.px)

                    Repeater {
                        model: root.consoles.places ?? []
                        delegate: Rectangle {
                            id: tablePlace
                            required property var modelData
                            readonly property bool current: root.model.consoleKey === (tablePlace.modelData.key ?? "")
                            Layout.fillWidth: true
                            height: Math.round(30 * root.px)
                            radius: Math.round(7 * root.px)
                            color: tablePlace.current ? ThemeInk.glowWeak : "transparent"
                            border.width: tablePlace.current ? 1 : 0
                            border.color: ThemeInk.coronaSoft

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Math.round(10 * root.px)
                                anchors.rightMargin: Math.round(10 * root.px)

                                Text {
                                    Layout.fillWidth: true
                                    text: tablePlace.modelData.label ?? ""
                                    elide: Text.ElideRight
                                    color: tablePlace.current ? ThemeInk.textStrong : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: tablePlace.current ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                                }

                                Text {
                                    text: String(tablePlace.modelData.count ?? 0)
                                    color: ThemeInk.dim
                                    font.family: Kit.Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            TapHandler {
                                onTapped: root.model.selectConsole(tablePlace.modelData.key ?? "")
                            }
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: ThemeInk.hairline
                }

                RowLayout {
                    Layout.fillWidth: true
                    Text {
                        Layout.fillWidth: true
                        text: root.chrome.consolesHeading ?? ""
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Kit.Tokens.weightWordmark
                    }
                    Text {
                        text: root.consoles.countsLine ?? ""
                        color: ThemeInk.dim
                        font.family: Kit.Tokens.faceCode
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    }
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(3 * root.px)
                    model: root.consoles.consoles ?? []

                    delegate: Rectangle {
                        id: tConRow
                        required property var modelData
                        readonly property bool current: root.model.consoleKey === (tConRow.modelData.key ?? "")
                        width: ListView.view.width
                        height: Math.round(30 * root.px)
                        radius: Math.round(7 * root.px)
                        color: tConRow.current ? ThemeInk.glowWeak : "transparent"
                        border.width: tConRow.current ? 1 : 0
                        border.color: ThemeInk.coronaSoft

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(8 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)
                            spacing: Math.round(8 * root.px)

                            CardPlanet {
                                unit: root.px
                                side: 14
                                planet: tConRow.modelData.planet ?? -2
                            }

                            Text {
                                Layout.fillWidth: true
                                text: tConRow.modelData.name ?? ""
                                elide: Text.ElideRight
                                color: tConRow.current ? ThemeInk.textStrong : ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                text: String(tConRow.modelData.count ?? 0)
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        TapHandler {
                            onTapped: root.model.selectConsole(tConRow.modelData.key ?? "")
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    radius: Math.round(8 * root.px)
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.hairline
                    implicitHeight: guaranteeCol.implicitHeight + Math.round(16 * root.px)

                    ColumnLayout {
                        id: guaranteeCol
                        anchors.fill: parent
                        anchors.margins: Math.round(8 * root.px)
                        spacing: Math.round(4 * root.px)

                        Text {
                            text: root.chrome.batchGuaranteeTitle ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        Text {
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                            text: root.chrome.contractNote ?? ""
                            color: ThemeInk.dim
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(9, Math.round(10 * root.px))
                        }
                    }
                }
            }
        }

        // Main table container and floating batch action bar.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Math.round(20 * root.px)
                spacing: Math.round(12 * root.px)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(12 * root.px)

                    Text {
                        text: root.chrome.tableHeading ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(20, Math.round(26 * root.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    Text {
                        text: root.games.summaryLine ?? ""
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(13 * root.px))
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    Rectangle {
                        visible: root.isFiltered
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: height / 2
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.pillBorder
                        implicitWidth: tClearText.implicitWidth + Math.round(20 * root.px)

                        Text {
                            id: tClearText
                            anchors.centerIn: parent
                            text: root.chrome.clearFiltersLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }

                        TapHandler {
                            onTapped: root.model.clearFilters()
                        }
                    }
                }

                // Filter bar above the table headers.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Repeater {
                        model: root.chrome.filters ?? []
                        delegate: Rectangle {
                            id: tFilterChip
                            required property var modelData
                            readonly property string fkey: tFilterChip.modelData.key ?? ""
                            readonly property bool active: root.model.filterKey === fkey
                            readonly property int countVal: root.games.filterCounts ? (root.games.filterCounts[fkey] ?? 0) : 0
                            height: Math.round(26 * root.px)
                            radius: height / 2
                            color: tFilterChip.active ? Kit.Tokens.textStrong : ThemeInk.glass
                            border.width: 1
                            border.color: tFilterChip.active ? Kit.Tokens.textStrong : ThemeInk.hairline
                            implicitWidth: tfRow.implicitWidth + Math.round(18 * root.px)

                            RowLayout {
                                id: tfRow
                                anchors.centerIn: parent
                                spacing: Math.round(5 * root.px)
                                Text {
                                    text: tFilterChip.modelData.label ?? ""
                                    color: tFilterChip.active ? Kit.Tokens.voidFill : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }
                                Text {
                                    text: String(tFilterChip.countVal)
                                    color: tFilterChip.active ? Kit.Tokens.voidFill : ThemeInk.dim
                                    font.family: Kit.Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            TapHandler {
                                onTapped: root.model.filterKey = tFilterChip.fkey
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                    }
                }

                // Column Headers row.
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(32 * root.px)
                    color: ThemeInk.glass
                    border.width: 1
                    border.color: ThemeInk.hairline
                    radius: Math.round(6 * root.px)

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(10 * root.px)
                        anchors.rightMargin: Math.round(10 * root.px)
                        spacing: Math.round(8 * root.px)

                        // Header Select All box.
                        Rectangle {
                            objectName: "desktopTableSelectAll"
                            Layout.preferredWidth: Math.round(18 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            radius: Math.round(4 * root.px)
                            color: root.model.allMarked ? Kit.Tokens.accent : "transparent"
                            border.width: 1
                            border.color: root.model.allMarked ? Kit.Tokens.accent : ThemeInk.pillBorder

                            Item {
                                anchors.centerIn: parent
                                width: Math.round(12 * root.px)
                                height: width
                                visible: root.model.allMarked
                                Shape {
                                    width: 24
                                    height: 24
                                    scale: parent.width / 24
                                    transformOrigin: Item.TopLeft
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        fillColor: "transparent"
                                        strokeColor: Kit.Tokens.overAccent
                                        strokeWidth: 2.2
                                        capStyle: ShapePath.RoundCap
                                        joinStyle: ShapePath.RoundJoin
                                        PathSvg { path: root.chrome.icons?.check ?? "" }
                                    }
                                }
                            }

                            TapHandler {
                                onTapped: root.model.markAll()
                            }
                        }

                        // Sortable column headings.
                        Repeater {
                            model: root.chrome.columns ?? []
                            delegate: Item {
                                id: colHead
                                required property string modelData
                                required property int index
                                readonly property bool sorted: root.model.sortIndex === colHead.index
                                Layout.fillWidth: colHead.index === 0 || colHead.index === 1
                                Layout.preferredWidth: colHead.index === 0 ? Math.round(240 * root.px) : colHead.index === 1 ? Math.round(140 * root.px) : colHead.index === 2 ? Math.round(115 * root.px) : colHead.index === 3 ? Math.round(64 * root.px) : colHead.index === 4 ? Math.round(105 * root.px) : colHead.index === 5 ? Math.round(92 * root.px) : colHead.index === 6 ? Math.round(84 * root.px) : colHead.index === 7 ? Math.round(78 * root.px) : Math.round(96 * root.px)
                                Layout.fillHeight: true

                                Text {
                                    anchors.verticalCenter: parent.verticalCenter
                                    text: colHead.sorted ? colHead.modelData + (root.model.sortAscending ? (root.chrome.sortAsc ?? "") : (root.chrome.sortDesc ?? "")) : colHead.modelData
                                    color: colHead.sorted ? ThemeInk.textStrong : ThemeInk.muted
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    font.weight: Kit.Tokens.weightWordmark
                                    font.letterSpacing: 0.6
                                }

                                TapHandler {
                                    onTapped: root.model.sortByColumn(colHead.index < 7 ? colHead.index : 0)
                                }
                            }
                        }
                    }
                }

                // Rows list.
                ListView {
                    id: tableRows
                    objectName: "desktopTableRows"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(2 * root.px)
                    model: root.games.rows ?? []
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                    }

                    delegate: Rectangle {
                        id: tRow
                        required property var modelData
                        required property int index
                        readonly property string rowPath: tRow.modelData.path ?? ""
                        readonly property bool marked: tRow.modelData.marked ?? false
                        readonly property bool selected: root.model.selectedKey === tRow.rowPath
                        width: tableRows.width
                        height: root.rowHeight
                        radius: Math.round(6 * root.px)
                        color: tRow.marked ? ThemeInk.glowWeak : tRow.selected ? ThemeInk.surfaceRaised : tRowHover.hovered ? ThemeInk.glass : "transparent"
                        border.width: tRow.marked || tRow.selected ? 1 : 0
                        border.color: tRow.marked ? ThemeInk.coronaSoft : ThemeInk.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)
                            spacing: Math.round(8 * root.px)

                            // Row multi-select checkbox.
                            Rectangle {
                                Layout.preferredWidth: Math.round(18 * root.px)
                                Layout.preferredHeight: Layout.preferredWidth
                                radius: Math.round(4 * root.px)
                                color: tRow.marked ? Kit.Tokens.accent : "transparent"
                                border.width: 1
                                border.color: tRow.marked ? Kit.Tokens.accent : ThemeInk.pillBorder

                                Item {
                                    anchors.centerIn: parent
                                    width: Math.round(12 * root.px)
                                    height: width
                                    visible: tRow.marked
                                    Shape {
                                        width: 24
                                        height: 24
                                        scale: parent.width / 24
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: "transparent"
                                            strokeColor: Kit.Tokens.overAccent
                                            strokeWidth: 2.2
                                            capStyle: ShapePath.RoundCap
                                            joinStyle: ShapePath.RoundJoin
                                            PathSvg { path: root.chrome.icons?.check ?? "" }
                                        }
                                    }
                                }

                                TapHandler {
                                    onTapped: root.model.markToggle(tRow.rowPath)
                                }
                            }

                            // Title + miniature thumbnail.
                            RowLayout {
                                Layout.fillWidth: true
                                Layout.preferredWidth: Math.round(240 * root.px)
                                spacing: Math.round(10 * root.px)

                                EclipseArtPlate {
                                    Layout.preferredWidth: Math.round(26 * root.px)
                                    Layout.preferredHeight: Layout.preferredWidth
                                    unit: root.px
                                    radius: 5
                                    cover: tRow.modelData.coverUrl ?? ""
                                    swatch: tRow.modelData.swatch ?? null
                                    slot: tRow.index % 3
                                }

                                Text {
                                    Layout.fillWidth: true
                                    text: tRow.modelData.title ?? ""
                                    elide: Text.ElideRight
                                    color: ThemeInk.textStrong
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }

                                Item {
                                    visible: tRow.modelData.favourite ?? false
                                    Layout.preferredWidth: Math.round(12 * root.px)
                                    Layout.preferredHeight: Layout.preferredWidth
                                    Shape {
                                        width: 24
                                        height: 24
                                        scale: parent.width / 24
                                        transformOrigin: Item.TopLeft
                                        preferredRendererType: Shape.CurveRenderer
                                        ShapePath {
                                            fillColor: ThemeInk.accent
                                            strokeColor: ThemeInk.accent
                                            strokeWidth: 1.6
                                            PathSvg { path: root.chrome.icons?.heart ?? "" }
                                        }
                                    }
                                }
                            }

                            // Console column.
                            Text {
                                Layout.fillWidth: true
                                Layout.preferredWidth: Math.round(140 * root.px)
                                text: tRow.modelData.console ?? ""
                                elide: Text.ElideRight
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            // Core + planet column.
                            RowLayout {
                                Layout.preferredWidth: Math.round(115 * root.px)
                                spacing: Math.round(6 * root.px)

                                CardPlanet {
                                    unit: root.px
                                    side: 12
                                    planet: tRow.modelData.planet ?? -2
                                }

                                Text {
                                    Layout.fillWidth: true
                                    text: tRow.modelData.core ?? ""
                                    elide: Text.ElideRight
                                    color: ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                }
                            }

                            Text {
                                Layout.preferredWidth: Math.round(64 * root.px)
                                text: (tRow.modelData.year ?? "").length > 0 ? tRow.modelData.year : (root.chrome.dash ?? "")
                                color: ThemeInk.muted
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                Layout.preferredWidth: Math.round(105 * root.px)
                                text: (tRow.modelData.genre ?? "").length > 0 ? tRow.modelData.genre : (root.chrome.dash ?? "")
                                elide: Text.ElideRight
                                color: ThemeInk.muted
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                Layout.preferredWidth: Math.round(92 * root.px)
                                text: tRow.modelData.playedWords ?? ""
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                Layout.preferredWidth: Math.round(84 * root.px)
                                text: tRow.modelData.statesLabel ?? ""
                                color: ThemeInk.muted
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }

                            Text {
                                Layout.preferredWidth: Math.round(78 * root.px)
                                text: tRow.modelData.format ?? ""
                                elide: Text.ElideRight
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }

                            Rectangle {
                                Layout.preferredWidth: Math.round(96 * root.px)
                                Layout.preferredHeight: Math.round(24 * root.px)
                                radius: height / 2
                                color: (tRow.modelData.resumeReady ?? false) ? ThemeInk.successFill : (tRow.modelData.favourite ?? false) ? ThemeInk.attentionFill : ThemeInk.surfaceRaised
                                border.width: 1
                                border.color: (tRow.modelData.resumeReady ?? false) ? ThemeInk.success : (tRow.modelData.favourite ?? false) ? ThemeInk.attention : ThemeInk.hairline

                                Text {
                                    anchors.centerIn: parent
                                    text: tRow.modelData.statusLabel ?? ""
                                    color: (tRow.modelData.resumeReady ?? false) ? ThemeInk.success : (tRow.modelData.favourite ?? false) ? ThemeInk.attention : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }
                            }
                        }

                        HoverHandler {
                            id: tRowHover
                        }

                        TapHandler {
                            onTapped: root.model.selectGame(tRow.rowPath)
                            onDoubleTapped: root.model.launchRequested(tRow.rowPath)
                        }
                    }
                }

                // Bottom batch selection toolbar when one or more rows are checked.
                Rectangle {
                    id: batchBar
                    objectName: "desktopTableBatchBar"
                    visible: root.model.markedCount > 0
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(50 * root.px)
                    radius: Math.round(12 * root.px)
                    color: ThemeInk.glassStrong
                    border.width: 1
                    border.color: ThemeInk.corona

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(14 * root.px)
                        anchors.rightMargin: Math.round(14 * root.px)
                        spacing: Math.round(10 * root.px)

                        Rectangle {
                            Layout.preferredHeight: Math.round(30 * root.px)
                            radius: height / 2
                            color: Kit.Tokens.accent
                            implicitWidth: countBadgeRow.implicitWidth + Math.round(20 * root.px)

                            RowLayout {
                                id: countBadgeRow
                                anchors.centerIn: parent
                                spacing: Math.round(4 * root.px)

                                Text {
                                    text: String(root.model.markedCount)
                                    color: Kit.Tokens.overAccent
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }

                                Text {
                                    text: root.chrome.selectedSuffix ?? ""
                                    color: Kit.Tokens.overAccent
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }
                            }
                        }

                        Rectangle {
                            Layout.preferredHeight: Math.round(30 * root.px)
                            radius: height / 2
                            color: "transparent"
                            border.width: 1
                            border.color: ThemeInk.hairline
                            implicitWidth: clearSelLabel.implicitWidth + Math.round(18 * root.px)

                            Text {
                                id: clearSelLabel
                                anchors.centerIn: parent
                                text: root.chrome.clearSelectionLabel ?? ""
                                color: ThemeInk.muted
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            TapHandler {
                                onTapped: root.model.clearMarked()
                            }
                        }

                        Item {
                            Layout.fillWidth: true
                        }

                        Rectangle {
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: height / 2
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.pillBorder
                            implicitWidth: markFavText.implicitWidth + Math.round(20 * root.px)

                            Text {
                                id: markFavText
                                anchors.centerIn: parent
                                text: root.chrome.batchMarkFav ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.batchFavourite(true)
                            }
                        }

                        Rectangle {
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: height / 2
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.pillBorder
                            implicitWidth: unmarkFavText.implicitWidth + Math.round(20 * root.px)

                            Text {
                                id: unmarkFavText
                                anchors.centerIn: parent
                                text: root.chrome.batchUnmarkFav ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.batchFavourite(false)
                            }
                        }

                        Rectangle {
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: height / 2
                            color: Kit.Tokens.accent
                            implicitWidth: playMarkedText.implicitWidth + Math.round(22 * root.px)

                            Text {
                                id: playMarkedText
                                anchors.centerIn: parent
                                text: root.chrome.batchPlay ?? ""
                                color: Kit.Tokens.overAccent
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: if (root.model.markedKeys.length > 0) root.model.launchRequested(root.model.markedKeys[0])
                            }
                        }
                    }
                }
            }
        }
    }

    Shortcut {
        enabled: root.visible
        sequence: "Ctrl+A"
        onActivated: root.model.markAll()
    }
}
