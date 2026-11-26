pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Reference Desktop Mode Library screen: three-pane studio with sidebar, filterable card grid, and live game inspector.
Item {
    id: root
    objectName: "desktopLibrary"
    required property DesktopModel model
    required property var chrome
    required property var consoles
    required property var games
    required property var inspector
    required property real px

    readonly property bool hasSelection: root.inspector.hasGame ?? false
    readonly property string selectedPath: root.inspector.path ?? ""
    readonly property bool isFiltered: root.model.filterKey !== (root.chrome.allKey ?? "") || root.model.searchQuery.length > 0

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Collapsible left Constellation & Consoles rail.
        Rectangle {
            id: sidebar
            objectName: "desktopLibrarySidebar"
            visible: root.model.sidebarOpen
            Layout.fillHeight: true
            Layout.preferredWidth: Math.round(244 * root.px)
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
                            id: placeBtn
                            required property var modelData
                            readonly property bool current: root.model.consoleKey === (placeBtn.modelData.key ?? "")
                            Layout.fillWidth: true
                            height: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: placeBtn.current ? ThemeInk.glowWeak : placeHover.hovered ? ThemeInk.surfaceRaised : "transparent"
                            border.width: placeBtn.current ? 1 : 0
                            border.color: ThemeInk.coronaSoft

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Math.round(10 * root.px)
                                anchors.rightMargin: Math.round(10 * root.px)
                                spacing: Math.round(8 * root.px)

                                Text {
                                    Layout.fillWidth: true
                                    text: placeBtn.modelData.label ?? ""
                                    elide: Text.ElideRight
                                    color: placeBtn.current ? ThemeInk.textStrong : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(13 * root.px))
                                    font.weight: placeBtn.current ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                                }

                                Text {
                                    text: String(placeBtn.modelData.count ?? 0)
                                    color: placeBtn.current ? ThemeInk.textStrong : ThemeInk.muted
                                    font.family: Kit.Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            HoverHandler {
                                id: placeHover
                            }

                            TapHandler {
                                onTapped: root.model.selectConsole(placeBtn.modelData.key ?? "")
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
                    spacing: Math.round(6 * root.px)

                    Text {
                        Layout.fillWidth: true
                        text: root.chrome.consolesHeading ?? ""
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Kit.Tokens.weightWordmark
                        font.letterSpacing: 1.1
                    }

                    Text {
                        text: root.consoles.countsLine ?? ""
                        color: ThemeInk.dim
                        font.family: Kit.Tokens.faceCode
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    }
                }

                // Scope pills: Active vs All.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(6 * root.px)

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(26 * root.px)
                        radius: Math.round(6 * root.px)
                        color: root.model.consoleScope === 1 ? ThemeInk.surfaceRaised : "transparent"
                        border.width: 1
                        border.color: root.model.consoleScope === 1 ? ThemeInk.coronaSoft : ThemeInk.hairline

                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.scopeActive ?? ""
                            color: root.model.consoleScope === 1 ? ThemeInk.textStrong : ThemeInk.muted
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        TapHandler {
                            onTapped: root.model.consoleScope = 1
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(26 * root.px)
                        radius: Math.round(6 * root.px)
                        color: root.model.consoleScope === 0 ? ThemeInk.surfaceRaised : "transparent"
                        border.width: 1
                        border.color: root.model.consoleScope === 0 ? ThemeInk.coronaSoft : ThemeInk.hairline

                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.scopeAll ?? ""
                            color: root.model.consoleScope === 0 ? ThemeInk.textStrong : ThemeInk.muted
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        TapHandler {
                            onTapped: root.model.consoleScope = 0
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(30 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ThemeInk.inputSurface
                    border.width: 1
                    border.color: consoleFilterInput.activeFocus ? ThemeInk.corona : ThemeInk.hairline

                    TextField {
                        id: consoleFilterInput
                        objectName: "desktopConsoleFilterInput"
                        anchors.fill: parent
                        leftPadding: Math.round(10 * root.px)
                        rightPadding: Math.round(10 * root.px)
                        color: ThemeInk.textStrong
                        placeholderTextColor: ThemeInk.dim
                        placeholderText: root.chrome.consoleFilterPlaceholder ?? ""
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        text: root.model.consoleFilter
                        background: Item {}
                        onTextChanged: if (text !== root.model.consoleFilter) root.model.consoleFilter = text
                    }
                }

                ListView {
                    id: consoleList
                    objectName: "desktopConsoleList"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(3 * root.px)
                    model: root.consoles.consoles ?? []
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                    }

                    delegate: Rectangle {
                        id: conRow
                        required property var modelData
                        readonly property bool current: root.model.consoleKey === (conRow.modelData.key ?? "")
                        width: consoleList.width
                        height: Math.round(32 * root.px)
                        radius: Math.round(7 * root.px)
                        color: conRow.current ? ThemeInk.glowWeak : conHover.hovered ? ThemeInk.surfaceRaised : "transparent"
                        border.width: conRow.current ? 1 : 0
                        border.color: ThemeInk.coronaSoft

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(8 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)
                            spacing: Math.round(8 * root.px)

                            CardPlanet {
                                unit: root.px
                                side: 16
                                glowBlur: 6
                                glowSpread: 1
                                planet: conRow.modelData.planet ?? -2
                            }

                            Text {
                                Layout.fillWidth: true
                                text: conRow.modelData.name ?? ""
                                elide: Text.ElideRight
                                color: conRow.current ? ThemeInk.textStrong : ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: conRow.current ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                            }

                            Text {
                                text: String(conRow.modelData.count ?? 0)
                                color: conRow.current ? ThemeInk.textStrong : ThemeInk.dim
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        HoverHandler {
                            id: conHover
                        }

                        TapHandler {
                            onTapped: root.model.selectConsole(conRow.modelData.key ?? "")
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.hairline

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(10 * root.px)
                        anchors.rightMargin: Math.round(10 * root.px)
                        spacing: Math.round(8 * root.px)

                        Rectangle {
                            Layout.preferredWidth: Math.round(8 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            radius: width / 2
                            color: root.model.controllers > 0 ? ThemeInk.success : ThemeInk.muted
                        }

                        Text {
                            Layout.fillWidth: true
                            text: root.consoles.controllersLine ?? ""
                            elide: Text.ElideRight
                            color: ThemeInk.text
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }
                }
            }
        }

        // Main center stage: console header, filter/sort/letter strip, and responsive game grid.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Math.round(20 * root.px)
                spacing: Math.round(14 * root.px)

                // Active collection heading & core badge.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(12 * root.px)

                    Text {
                        objectName: "desktopLibraryHeading"
                        text: root.consoles.activeHeading ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(20, Math.round(28 * root.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    Rectangle {
                        visible: (root.consoles.activeCore ?? "").length > 0
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: height / 2
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.hairline
                        implicitWidth: coreRow.implicitWidth + Math.round(20 * root.px)

                        RowLayout {
                            id: coreRow
                            anchors.centerIn: parent
                            spacing: Math.round(7 * root.px)

                            CardPlanet {
                                unit: root.px
                                side: 14
                                glowBlur: 6
                                glowSpread: 1
                                planet: root.consoles.activePlanet ?? -2
                            }

                            Text {
                                text: root.consoles.activeCore ?? ""
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightSentence
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    Rectangle {
                        visible: (root.consoles.activeCore ?? "").length > 0
                        Layout.preferredHeight: Math.round(30 * root.px)
                        radius: Math.round(8 * root.px)
                        color: optHover.hovered ? ThemeInk.surfaceRaised : ThemeInk.glass
                        border.width: 1
                        border.color: ThemeInk.pillBorder
                        implicitWidth: optLabel.implicitWidth + Math.round(22 * root.px)

                        Text {
                            id: optLabel
                            anchors.centerIn: parent
                            text: root.chrome.consoleOptionsLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        HoverHandler {
                            id: optHover
                        }

                        TapHandler {
                            onTapped: root.model.openConsoleOptions(root.model.consoleKey)
                        }
                    }
                }

                // Filter chips, sort toggle, and quick-jump letters.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Repeater {
                        model: root.chrome.filters ?? []
                        delegate: Rectangle {
                            id: filterChip
                            required property var modelData
                            readonly property string fkey: filterChip.modelData.key ?? ""
                            readonly property bool active: root.model.filterKey === fkey
                            readonly property int countVal: root.games.filterCounts ? (root.games.filterCounts[fkey] ?? 0) : 0
                            height: Math.round(28 * root.px)
                            radius: height / 2
                            color: filterChip.active ? Kit.Tokens.textStrong : chipHover.hovered ? ThemeInk.surfaceRaised : ThemeInk.glass
                            border.width: 1
                            border.color: filterChip.active ? Kit.Tokens.textStrong : ThemeInk.hairline
                            implicitWidth: chipRow.implicitWidth + Math.round(20 * root.px)

                            RowLayout {
                                id: chipRow
                                anchors.centerIn: parent
                                spacing: Math.round(6 * root.px)

                                Text {
                                    text: filterChip.modelData.label ?? ""
                                    color: filterChip.active ? Kit.Tokens.voidFill : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }

                                Text {
                                    text: String(filterChip.countVal)
                                    color: filterChip.active ? Kit.Tokens.voidFill : ThemeInk.dim
                                    font.family: Kit.Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            HoverHandler {
                                id: chipHover
                            }

                            TapHandler {
                                onTapped: root.model.filterKey = filterChip.fkey
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                    }

                    Rectangle {
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: Math.round(7 * root.px)
                        color: sortHover.hovered ? ThemeInk.surfaceRaised : ThemeInk.glass
                        border.width: 1
                        border.color: ThemeInk.hairline
                        implicitWidth: sortText.implicitWidth + Math.round(20 * root.px)

                        Text {
                            id: sortText
                            anchors.centerIn: parent
                            text: (root.chrome.sortLabel ?? "") + (root.chrome.sortOptions ? (root.chrome.sortOptions[root.model.sortIndex] ?? "") : "")
                            color: ThemeInk.text
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }

                        HoverHandler {
                            id: sortHover
                        }

                        TapHandler {
                            onTapped: root.model.stepSort()
                        }
                    }

                    Rectangle {
                        visible: (root.games.letters ?? []).length > 0
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: Math.round(7 * root.px)
                        color: ThemeInk.glass
                        border.width: 1
                        border.color: ThemeInk.hairline
                        implicitWidth: Math.min(Math.round(240 * root.px), letterRow.implicitWidth + Math.round(14 * root.px))
                        clip: true

                        RowLayout {
                            id: letterRow
                            anchors.centerIn: parent
                            spacing: Math.round(2 * root.px)

                            Repeater {
                                model: root.games.letters ?? []
                                delegate: Item {
                                    id: letterChip
                                    required property string modelData
                                    width: Math.round(16 * root.px)
                                    height: Math.round(22 * root.px)

                                    Text {
                                        anchors.centerIn: parent
                                        text: letterChip.modelData
                                        color: letterHover.hovered ? ThemeInk.textStrong : ThemeInk.muted
                                        font.family: Kit.Tokens.faceCode
                                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    }

                                    HoverHandler {
                                        id: letterHover
                                    }

                                    TapHandler {
                                        onTapped: gameGrid.positionViewAtIndex(Math.max(0, root.model.jumpLetter(letterChip.modelData)), GridView.Beginning)
                                    }
                                }
                            }
                        }
                    }
                }

                // Cover Grid or empty state.
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true

                    GridView {
                        id: gameGrid
                        objectName: "desktopLibraryGrid"
                        anchors.fill: parent
                        visible: (root.games.count ?? 0) > 0
                        clip: true
                        readonly property int cols: Math.max(2, Math.min(5, Math.floor((gameGrid.width + Math.round(14 * root.px)) / Math.round(212 * root.px))))
                        cellWidth: Math.floor(gameGrid.width / gameGrid.cols)
                        cellHeight: Math.round(gameGrid.cellWidth * 0.78)
                        model: root.games.rows ?? []
                        ScrollBar.vertical: ScrollBar {
                            policy: ScrollBar.AsNeeded
                        }

                        delegate: Item {
                            id: cardSlot
                            required property var modelData
                            required property int index
                            readonly property string itemPath: cardSlot.modelData.path ?? ""
                            readonly property bool selected: root.selectedPath === cardSlot.itemPath
                            width: gameGrid.cellWidth
                            height: gameGrid.cellHeight

                            Rectangle {
                                id: cardItem
                                anchors.fill: parent
                                anchors.margins: Math.round(6 * root.px)
                                radius: Math.round(Math.min(12, Kit.Tokens.tileRadius) * root.px)
                                color: ThemeInk.surfaceRaised
                                border.width: cardSlot.selected ? 2 : 1
                                border.color: cardSlot.selected ? ThemeInk.corona : cardHover.hovered ? ThemeInk.pillBorder : ThemeInk.hairline

                                EclipseArtPlate {
                                    anchors.fill: parent
                                    anchors.margins: 1
                                    unit: root.px
                                    radius: Math.min(11, Kit.Tokens.tileRadius)
                                    cover: cardSlot.modelData.coverUrl ?? ""
                                    swatch: cardSlot.modelData.swatch ?? null
                                    slot: cardSlot.index % 3
                                }

                                // Top badge for running/saved state.
                                Rectangle {
                                    visible: (cardSlot.modelData.badge ?? "").length > 0
                                    anchors.left: parent.left
                                    anchors.top: parent.top
                                    anchors.margins: Math.round(8 * root.px)
                                    height: Math.round(20 * root.px)
                                    radius: height / 2
                                    color: ThemeInk.overArtScrim
                                    border.width: 1
                                    border.color: ThemeInk.pillBorder
                                    implicitWidth: badgeText.implicitWidth + Math.round(14 * root.px)

                                    Text {
                                        id: badgeText
                                        anchors.centerIn: parent
                                        text: cardSlot.modelData.badge ?? ""
                                        color: ThemeInk.overArt
                                        font.family: Kit.Tokens.faceCode
                                        font.pixelSize: Math.max(9, Math.round(10 * root.px))
                                        font.weight: Kit.Tokens.weightTitle
                                    }
                                }

                                // Top-right favourite button.
                                Rectangle {
                                    anchors.right: parent.right
                                    anchors.top: parent.top
                                    anchors.margins: Math.round(8 * root.px)
                                    width: Math.round(24 * root.px)
                                    height: width
                                    radius: width / 2
                                    color: ThemeInk.overArtScrim

                                    Item {
                                        anchors.centerIn: parent
                                        width: Math.round(13 * root.px)
                                        height: width
                                        Shape {
                                            width: 24
                                            height: 24
                                            scale: parent.width / 24
                                            transformOrigin: Item.TopLeft
                                            preferredRendererType: Shape.CurveRenderer
                                            ShapePath {
                                                fillColor: (cardSlot.modelData.favourite ?? false) ? ThemeInk.accent : "transparent"
                                                strokeColor: (cardSlot.modelData.favourite ?? false) ? ThemeInk.accent : ThemeInk.overArt
                                                strokeWidth: 1.8
                                                capStyle: ShapePath.RoundCap
                                                joinStyle: ShapePath.RoundJoin
                                                PathSvg { path: root.chrome.icons?.heart ?? "" }
                                            }
                                        }
                                    }

                                    TapHandler {
                                        onTapped: {
                                            root.model.selectGame(cardSlot.itemPath)
                                            root.model.toggleSelectedFavourite()
                                        }
                                    }
                                }

                                // Bottom gradient caption bar over art.
                                Rectangle {
                                    anchors.left: parent.left
                                    anchors.right: parent.right
                                    anchors.bottom: parent.bottom
                                    anchors.margins: 1
                                    height: Math.round(54 * root.px)
                                    radius: Math.round(Math.min(11, Kit.Tokens.tileRadius) * root.px)
                                    gradient: Gradient {
                                        GradientStop { position: 0.0; color: "transparent" }
                                        GradientStop { position: 0.45; color: ThemeInk.overArtScrim }
                                        GradientStop { position: 1.0; color: ThemeInk.overArtScrim }
                                    }

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: Math.round(10 * root.px)
                                        anchors.rightMargin: Math.round(10 * root.px)
                                        anchors.bottomMargin: Math.round(7 * root.px)
                                        spacing: Math.round(6 * root.px)

                                        ColumnLayout {
                                            Layout.fillWidth: true
                                            Layout.alignment: Qt.AlignBottom
                                            spacing: 1

                                            Text {
                                                Layout.fillWidth: true
                                                text: cardSlot.modelData.title ?? ""
                                                elide: Text.ElideRight
                                                color: ThemeInk.overArt
                                                font.family: Kit.Tokens.faceInterface
                                                font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                                font.weight: Kit.Tokens.weightTitle
                                            }

                                            Text {
                                                Layout.fillWidth: true
                                                text: cardSlot.modelData.subtitle ?? ""
                                                elide: Text.ElideRight
                                                color: Qt.alpha(ThemeInk.overArt, 0.78)
                                                font.family: Kit.Tokens.faceInterface
                                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                            }
                                        }

                                        Text {
                                            Layout.alignment: Qt.AlignBottom
                                            text: cardSlot.modelData.playedWords ?? ""
                                            color: Qt.alpha(ThemeInk.overArt, 0.78)
                                            font.family: Kit.Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }
                                    }
                                }

                                HoverHandler {
                                    id: cardHover
                                }

                                TapHandler {
                                    onTapped: root.model.selectGame(cardSlot.itemPath)
                                    onDoubleTapped: root.model.launchRequested(cardSlot.itemPath)
                                }
                            }
                        }
                    }

                    // Empty filtered/library placeholder.
                    ColumnLayout {
                        anchors.centerIn: parent
                        visible: (root.games.count ?? 0) === 0
                        spacing: Math.round(12 * root.px)

                        Text {
                            Layout.alignment: Qt.AlignHCenter
                            text: root.isFiltered ? (root.chrome.emptyFiltered ?? "") : (root.chrome.emptyLibrary ?? "")
                            color: ThemeInk.muted
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(13, Math.round(15 * root.px))
                        }

                        Rectangle {
                            Layout.alignment: Qt.AlignHCenter
                            visible: root.isFiltered
                            Layout.preferredHeight: Math.round(30 * root.px)
                            radius: height / 2
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.pillBorder
                            implicitWidth: clearBtnText.implicitWidth + Math.round(24 * root.px)

                            Text {
                                id: clearBtnText
                                anchors.centerIn: parent
                                text: root.chrome.clearFiltersLabel ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.clearFilters()
                            }
                        }
                    }
                }
            }
        }

        // Collapsible right Game Inspector pane.
        Rectangle {
            id: inspectorPane
            objectName: "desktopLibraryInspector"
            visible: root.model.inspectorOpen && root.hasSelection
            Layout.fillHeight: true
            Layout.preferredWidth: Math.round(344 * root.px)
            color: ThemeInk.glass
            border.width: 1
            border.color: ThemeInk.hairline

            Flickable {
                anchors.fill: parent
                anchors.margins: Math.round(16 * root.px)
                contentHeight: inspectorCol.implicitHeight
                clip: true

                ColumnLayout {
                    id: inspectorCol
                    width: parent.width
                    spacing: Math.round(12 * root.px)

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Math.round(8 * root.px)

                        Text {
                            Layout.fillWidth: true
                            text: root.chrome.inspectorHeading ?? ""
                            color: ThemeInk.muted
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            font.weight: Kit.Tokens.weightWordmark
                            font.letterSpacing: 1.1
                        }

                        Rectangle {
                            Layout.preferredWidth: Math.round(28 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            radius: width / 2
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.hairline

                            Item {
                                anchors.centerIn: parent
                                width: Math.round(13 * root.px)
                                height: width
                                Shape {
                                    width: 24
                                    height: 24
                                    scale: parent.width / 24
                                    transformOrigin: Item.TopLeft
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        fillColor: (root.inspector.favourite ?? false) ? ThemeInk.accent : "transparent"
                                        strokeColor: (root.inspector.favourite ?? false) ? ThemeInk.accent : ThemeInk.text
                                        strokeWidth: 1.8
                                        capStyle: ShapePath.RoundCap
                                        joinStyle: ShapePath.RoundJoin
                                        PathSvg { path: root.chrome.icons?.heart ?? "" }
                                    }
                                }
                            }

                            TapHandler {
                                onTapped: root.model.toggleSelectedFavourite()
                            }
                        }
                    }

                    // Hero banner preview.
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(128 * root.px)
                        radius: Math.round(10 * root.px)
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.hairline

                        EclipseArtPlate {
                            anchors.fill: parent
                            anchors.margins: 1
                            unit: root.px
                            radius: 9
                            cover: root.inspector.heroUrl ?? ""
                            swatch: root.inspector.swatch ?? null
                            slot: 0
                        }
                    }

                    Text {
                        Layout.fillWidth: true
                        text: root.inspector.subtitle ?? ""
                        elide: Text.ElideRight
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceCode
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.letterSpacing: 0.6
                    }

                    Text {
                        objectName: "desktopInspectorTitle"
                        Layout.fillWidth: true
                        text: root.inspector.title ?? ""
                        wrapMode: Text.WordWrap
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(18, Math.round(22 * root.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    // Play / Resume buttons.
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Math.round(8 * root.px)

                        Rectangle {
                            objectName: "desktopPlayButton"
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(38 * root.px)
                            radius: height / 2
                            color: Kit.Tokens.accent

                            Text {
                                anchors.centerIn: parent
                                text: root.chrome.playLabel ?? ""
                                color: Kit.Tokens.overAccent
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.launchRequested(root.selectedPath)
                            }
                        }

                        Rectangle {
                            objectName: "desktopResumeButton"
                            visible: root.inspector.running ?? false
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(38 * root.px)
                            radius: height / 2
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.corona

                            Text {
                                anchors.centerIn: parent
                                text: root.chrome.resumeSlotLabel ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.resumeRequested()
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: ThemeInk.hairline
                    }

                    Text {
                        text: root.inspector.statesHeading ?? (root.chrome.saveStatesLabel ?? "")
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Kit.Tokens.weightWordmark
                        font.letterSpacing: 1.0
                    }

                    Text {
                        visible: (root.inspector.statesCount ?? 0) === 0
                        text: root.chrome.noStatesLabel ?? ""
                        color: ThemeInk.dim
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: ThemeInk.hairline
                    }

                    Text {
                        text: root.chrome.programFactsLabel ?? ""
                        color: ThemeInk.muted
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Kit.Tokens.weightWordmark
                        font.letterSpacing: 1.0
                    }

                    // Key-value facts rows.
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: Math.round(6 * root.px)

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factCore ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            CardPlanet {
                                unit: root.px
                                side: 12
                                planet: root.inspector.planet ?? -2
                            }
                            Text {
                                text: root.inspector.coreLine ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factPlayed ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            Text {
                                text: root.inspector.playedWords ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factPlayers ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            Text {
                                text: root.inspector.players ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factGenre ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            Text {
                                text: root.inspector.genre ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factFile ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            Text {
                                Layout.maximumWidth: Math.round(180 * root.px)
                                elide: Text.ElideMiddle
                                text: root.inspector.fileName ?? ""
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: root.chrome.factSize ?? ""
                                color: ThemeInk.dim
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }
                            Text {
                                text: root.inspector.sizeWords ?? ""
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        radius: Math.round(8 * root.px)
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.hairline
                        implicitHeight: contractNoteLabel.implicitHeight + Math.round(16 * root.px)

                        Text {
                            id: contractNoteLabel
                            anchors.fill: parent
                            anchors.margins: Math.round(8 * root.px)
                            wrapMode: Text.WordWrap
                            text: root.chrome.contractNote ?? ""
                            color: ThemeInk.dim
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }

                    // Bottom secondary action row: Page, Favourite, Folder.
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Math.round(6 * root.px)

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.hairline

                            Text {
                                anchors.centerIn: parent
                                text: root.chrome.detailsLabel ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.pageRequested(root.selectedPath)
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.hairline

                            Text {
                                anchors.centerIn: parent
                                text: (root.inspector.favourite ?? false) ? (root.chrome.unfavouriteLabel ?? "") : (root.chrome.favouriteLabel ?? "")
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.toggleSelectedFavourite()
                            }
                        }

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: ThemeInk.hairline

                            Text {
                                anchors.centerIn: parent
                                text: root.chrome.folderLabel ?? ""
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.model.openSelectedFolder()
                            }
                        }
                    }
                }
            }
        }
    }
}
