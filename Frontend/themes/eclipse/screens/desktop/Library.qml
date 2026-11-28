pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme
import "../../parts"
import "../../parts/desktop"

// Eclipse Schema 2 Desktop Library screen: Constellation rail, cover grid, and live Game Inspector.
ThemeScreen {
    id: screen
    objectName: "eclipseDesktopLibraryScreen"
    role: ThemeScreen.DesktopLibrary

    Item {
        id: root
        objectName: "eclipseDesktopLibrary"
        anchors.fill: parent
        readonly property real px: Math.max(0.84, root.height / 1000)

        DesktopInk {
            id: ink
        }

        DesktopBackdrop {
            anchors.fill: parent
            px: root.px
            pixelScale: root.px
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            DesktopTopRibbon {
                px: root.px
            }

            // Three-pane studio workspace.
            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                DesktopConstellationRail {
                    visible: Workspace.sidebarOpen
                    px: root.px
                    tableMode: false
                }

                // Main center stage: active heading, filter chips, and game cover grid.
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Math.round(20 * root.px)
                        spacing: Math.round(14 * root.px)

                        // Console/collection heading and active core badge.
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(12 * root.px)

                            Text {
                                objectName: "eclipseDesktopLibraryHeading"
                                text: Workspace.activeHeading
                                color: ink.textStrong
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(20, Math.round(Tokens.fontTitle * root.px))
                                font.weight: Tokens.weightTitle
                            }

                            Rectangle {
                                visible: Workspace.activeCore.length > 0
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: ink.surfaceRaised
                                border.width: 1
                                border.color: ink.hairline
                                implicitWidth: coreChipRow.implicitWidth + Math.round(20 * root.px)

                                RowLayout {
                                    id: coreChipRow
                                    anchors.centerIn: parent
                                    spacing: Math.round(7 * root.px)

                                    PlanetSphere {
                                        unit: root.px
                                        side: 14
                                        glowBlur: 6
                                        glowSpread: 1
                                        planet: Workspace.activePlanet
                                    }

                                    Text {
                                        text: Workspace.activeCore
                                        color: ink.text
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightSentence
                                    }
                                }
                            }

                            Item {
                                Layout.fillWidth: true
                            }

                            Rectangle {
                                id: libConsoleOptBtn
                                visible: Workspace.activeCore.length > 0
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopConsoleOptionsLabel
                                Layout.preferredHeight: Math.round(30 * root.px)
                                radius: Math.round(8 * root.px)
                                color: optHover.hovered ? ink.surfaceRaised : ink.glass
                                border.width: libConsoleOptBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                border.color: libConsoleOptBtn.activeFocus ? ink.corona : ink.pillBorder
                                implicitWidth: optLabel.implicitWidth + Math.round(22 * root.px)

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        Actions.consoleOptions(Workspace.consoleKey)
                                        event.accepted = true
                                    }
                                }

                                Text {
                                    id: optLabel
                                    anchors.centerIn: parent
                                    text: Words.desktopConsoleOptionsLabel
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                HoverHandler {
                                    id: optHover
                                }

                                TapHandler {
                                    onTapped: Actions.consoleOptions(Workspace.consoleKey)
                                }
                            }
                        }

                        // Filter chips, sort button, and quick-jump letter strip.
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(8 * root.px)

                            Repeater {
                                model: Workspace.filters
                                delegate: Rectangle {
                                    id: filterChip
                                    required property string key
                                    required property string label
                                    required property int count
                                    readonly property bool active: Workspace.filterKey === filterChip.key
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: filterChip.label
                                    height: Math.round(28 * root.px)
                                    radius: height / 2
                                    color: filterChip.active ? ink.textStrong : chipHover.hovered ? ink.surfaceRaised : ink.glass
                                    border.width: filterChip.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: filterChip.activeFocus ? ink.corona : filterChip.active ? ink.textStrong : ink.hairline
                                    implicitWidth: chipRow.implicitWidth + Math.round(20 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceSetFilter(filterChip.key)
                                            event.accepted = true
                                        }
                                    }

                                    RowLayout {
                                        id: chipRow
                                        anchors.centerIn: parent
                                        spacing: Math.round(6 * root.px)

                                        Text {
                                            text: filterChip.label
                                            color: filterChip.active ? ink.surface : ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                            font.weight: Tokens.weightTitle
                                        }

                                        Text {
                                            text: String(filterChip.count)
                                            color: filterChip.active ? ink.surface : ink.dim
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }
                                    }

                                    HoverHandler {
                                        id: chipHover
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceSetFilter(filterChip.key)
                                    }
                                }
                            }

                            Item {
                                Layout.fillWidth: true
                            }

                            Rectangle {
                                id: libSortBtn
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopSortLabel
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: Math.round(7 * root.px)
                                color: sortHover.hovered ? ink.surfaceRaised : ink.glass
                                border.width: libSortBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                border.color: libSortBtn.activeFocus ? ink.corona : ink.hairline
                                implicitWidth: sortRow.implicitWidth + Math.round(20 * root.px)

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        Actions.workspaceSetSort((Workspace.sortIndex + 1) % Words.desktopSortLabels.length)
                                        event.accepted = true
                                    }
                                }

                                RowLayout {
                                    id: sortRow
                                    anchors.centerIn: parent
                                    spacing: Math.round(5 * root.px)

                                    Text {
                                        text: Words.desktopSortLabel
                                        color: ink.muted
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    }

                                    Text {
                                        text: Words.desktopSortLabels[Workspace.sortIndex] ?? Words.desktopSortLabels[0]
                                        color: ink.textStrong
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }
                                }

                                HoverHandler {
                                    id: sortHover
                                }

                                TapHandler {
                                    onTapped: Actions.workspaceSetSort((Workspace.sortIndex + 1) % Words.desktopSortLabels.length)
                                }
                            }

                            Rectangle {
                                visible: root.width >= 1400 && letterRepeater.count > 0
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: Math.round(7 * root.px)
                                color: ink.glass
                                border.width: 1
                                border.color: ink.hairline
                                implicitWidth: Math.min(Math.round(230 * root.px), letterRow.implicitWidth + Math.round(14 * root.px))
                                clip: true

                                RowLayout {
                                    id: letterRow
                                    anchors.centerIn: parent
                                    spacing: Math.round(4 * root.px)

                                    Repeater {
                                        id: letterRepeater
                                        model: Workspace.letters
                                        delegate: Text {
                                            required property string letter
                                            text: letter
                                            color: ink.muted
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }
                                    }
                                }
                            }
                        }

                        // Cover Grid or empty state.
                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            EdgeFade {
                                anchors.fill: parent
                                visible: Workspace.count > 0
                                fadeTop: !gameGrid.atYBeginning
                                fadeBottom: !gameGrid.atYEnd
                                length: Math.round(18 * root.px)

                                GridView {
                                    id: gameGrid
                                    objectName: "eclipseDesktopLibraryGrid"
                                    anchors.fill: parent
                                    clip: true
                                    readonly property int cols: Math.max(2, Math.min(5, Math.floor((gameGrid.width + Math.round(14 * root.px)) / Math.round(212 * root.px))))
                                    cellWidth: Math.floor(gameGrid.width / gameGrid.cols)
                                    cellHeight: Math.round(gameGrid.cellWidth * 0.78)
                                    model: Workspace.games
                                    readonly property bool fadeTop: !atYBeginning
                                    readonly property bool fadeBottom: !atYEnd
                                    ScrollBar.vertical: ScrollBar {
                                        policy: ScrollBar.AsNeeded
                                    }

                                    delegate: Item {
                                        id: cardSlot
                                        required property int index
                                        required property string key
                                        required property string title
                                        required property string subtitle
                                        required property string playedWords
                                        required property string badge
                                        required property string cover
                                        required property int planet
                                        required property bool favourite
                                        readonly property bool selected: Workspace.inspectorKey === cardSlot.key
                                        width: gameGrid.cellWidth
                                        height: gameGrid.cellHeight

                                        Rectangle {
                                            id: cardItem
                                            activeFocusOnTab: true
                                            Accessible.role: Accessible.Button
                                            Accessible.name: cardSlot.title
                                            anchors.fill: parent
                                            anchors.margins: Math.round(6 * root.px)
                                            radius: Math.round(Tokens.tileRadius * root.px)
                                            color: ink.surfaceRaised
                                            border.width: cardItem.activeFocus || cardSlot.selected ? 2 : 1
                                            border.color: cardItem.activeFocus || cardSlot.selected ? ink.corona : cardHover.hovered ? ink.pillBorder : ink.hairline

                                            Keys.onPressed: (event) => {
                                                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                                                    Actions.workspaceSelectGame(cardSlot.key)
                                                    Actions.play(cardSlot.key)
                                                    event.accepted = true
                                                } else if (event.key === Qt.Key_Space) {
                                                    Actions.workspaceSelectGame(cardSlot.key)
                                                    event.accepted = true
                                                }
                                            }

                                            Glow {
                                                anchors.fill: parent
                                                visible: cardSlot.selected
                                                radius: parent.radius
                                                color: ink.glow
                                                blur: Tokens.glowBlur * root.px
                                                spread: Tokens.glowSpread * root.px
                                                pixelScale: root.px
                                            }

                                            DesktopArtTile {
                                                anchors.fill: parent
                                                anchors.margins: 1
                                                px: root.px
                                                radius: Tokens.tileRadius - 1
                                                source: cardSlot.cover
                                                planet: cardSlot.planet
                                                slot: cardSlot.index
                                                pixelScale: root.px
                                            }

                                            // Top-left state/resume pill over art.
                                            Rectangle {
                                                visible: cardSlot.badge.length > 0
                                                anchors.left: parent.left
                                                anchors.top: parent.top
                                                anchors.margins: Math.round(8 * root.px)
                                                height: Math.round(20 * root.px)
                                                radius: height / 2
                                                color: ink.overArtScrim
                                                border.width: 1
                                                border.color: ink.pillBorder
                                                implicitWidth: badgeText.implicitWidth + Math.round(14 * root.px)

                                                Text {
                                                    id: badgeText
                                                    anchors.centerIn: parent
                                                    text: cardSlot.badge
                                                    color: ink.overArt
                                                    font.family: Tokens.faceCode
                                                    font.pixelSize: Math.max(9, Math.round(10 * root.px))
                                                    font.weight: Tokens.weightTitle
                                                }
                                            }

                                            // Top-right favourite toggle button.
                                            Rectangle {
                                                id: cardFavBtn
                                                activeFocusOnTab: true
                                                Accessible.role: Accessible.Button
                                                Accessible.name: cardSlot.favourite ? Words.desktopUnfavouriteLabel : Words.desktopFavouriteLabel
                                                anchors.right: parent.right
                                                anchors.top: parent.top
                                                anchors.margins: Math.round(8 * root.px)
                                                width: Math.round(24 * root.px)
                                                height: width
                                                radius: width / 2
                                                color: ink.overArtScrim
                                                border.width: cardFavBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 0
                                                border.color: ink.corona

                                                Keys.onPressed: (event) => {
                                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                        Actions.workspaceSelectGame(cardSlot.key)
                                                        Actions.favourite(cardSlot.key)
                                                        event.accepted = true
                                                    }
                                                }

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
                                                            fillColor: cardSlot.favourite ? ink.accent : "transparent"
                                                            strokeColor: cardSlot.favourite ? ink.accent : ink.overArt
                                                            strokeWidth: 1.8
                                                            capStyle: ShapePath.RoundCap
                                                            joinStyle: ShapePath.RoundJoin
                                                            PathSvg { path: Marks.heart }
                                                        }
                                                    }
                                                }

                                                TapHandler {
                                                    onTapped: {
                                                        Actions.workspaceSelectGame(cardSlot.key)
                                                        Actions.favourite(cardSlot.key)
                                                    }
                                                }
                                            }

                                            // Bottom caption bar over art.
                                            Rectangle {
                                                anchors.left: parent.left
                                                anchors.right: parent.right
                                                anchors.bottom: parent.bottom
                                                anchors.margins: 1
                                                height: Math.round(54 * root.px)
                                                radius: Math.round((Tokens.tileRadius - 1) * root.px)
                                                gradient: Gradient {
                                                    GradientStop { position: 0.0; color: "transparent" }
                                                    GradientStop { position: 0.45; color: ink.overArtScrim }
                                                    GradientStop { position: 1.0; color: ink.overArtScrim }
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
                                                            text: cardSlot.title
                                                            elide: Text.ElideRight
                                                            color: ink.overArt
                                                            font.family: Tokens.faceInterface
                                                            font.pixelSize: Math.max(12, Math.round(Tokens.fontRow * root.px))
                                                            font.weight: Tokens.weightTitle
                                                        }

                                                        Text {
                                                            Layout.fillWidth: true
                                                            text: cardSlot.subtitle
                                                            elide: Text.ElideRight
                                                            color: Qt.alpha(ink.overArt, 0.80)
                                                            font.family: Tokens.faceInterface
                                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                                        }
                                                    }

                                                    Text {
                                                        Layout.alignment: Qt.AlignBottom
                                                        text: cardSlot.playedWords
                                                        color: Qt.alpha(ink.overArt, 0.80)
                                                        font.family: Tokens.faceCode
                                                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                                    }
                                                }
                                            }

                                            HoverHandler {
                                                id: cardHover
                                            }

                                            TapHandler {
                                                onTapped: Actions.workspaceSelectGame(cardSlot.key)
                                                onDoubleTapped: Actions.play(cardSlot.key)
                                            }
                                        }
                                    }
                                }
                            }

                            // Empty library / filter state.
                            ColumnLayout {
                                anchors.centerIn: parent
                                visible: Workspace.count === 0
                                spacing: Math.round(12 * root.px)

                                Text {
                                    Layout.alignment: Qt.AlignHCenter
                                    text: Workspace.emptyWords
                                    color: ink.muted
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(13, Math.round(15 * root.px))
                                }

                                Rectangle {
                                    id: libClearFiltersBtn
                                    Layout.alignment: Qt.AlignHCenter
                                    visible: Workspace.searchQuery.length > 0 || Workspace.count !== Workspace.totalCount
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: Words.desktopClearFiltersLabel
                                    Layout.preferredHeight: Math.round(30 * root.px)
                                    radius: height / 2
                                    color: ink.surfaceRaised
                                    border.width: libClearFiltersBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: libClearFiltersBtn.activeFocus ? ink.corona : ink.pillBorder
                                    implicitWidth: clearBtnText.implicitWidth + Math.round(24 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceClearFilters()
                                            event.accepted = true
                                        }
                                    }

                                    Text {
                                        id: clearBtnText
                                        anchors.centerIn: parent
                                        text: Words.desktopClearFiltersLabel
                                        color: ink.textStrong
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceClearFilters()
                                    }
                                }
                            }
                        }
                    }
                }

                DesktopInspectorPane {
                    visible: Workspace.inspectorOpen && Workspace.hasSelection
                    px: root.px
                    pixelScale: root.px
                }
            }

            DesktopStatusBar {
                px: root.px
            }
        }

        Shortcut {
            enabled: root.visible && Workspace.searchQuery.length > 0
            sequence: "Escape"
            onActivated: Actions.workspaceClearFilters()
        }
    }
}
