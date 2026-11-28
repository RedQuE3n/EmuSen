pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme
import "../../parts"
import "../../parts/desktop"

// Eclipse Schema 2 Desktop Table screen: high-density catalog rows and multi-select batch action bar.
ThemeScreen {
    id: screen
    objectName: "eclipseDesktopTableScreen"
    role: ThemeScreen.DesktopTable

    Item {
        id: root
        objectName: "eclipseDesktopTable"
        anchors.fill: parent
        readonly property real px: Math.max(0.84, root.height / 1000)
        readonly property real rowHeight: Math.min(48 * root.px, Math.max(36 * root.px, Tokens.rowHeight * root.px))
        readonly property bool inputFocused: Window.activeFocusItem instanceof TextInput

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
                showInspectorToggle: false
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                DesktopConstellationRail {
                    visible: Workspace.sidebarOpen
                    px: root.px
                    tableMode: true
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
                                objectName: "eclipseDesktopTableHeading"
                                text: Words.desktopTableHeading
                                color: ink.textStrong
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(20, Math.round(Tokens.fontTitle * root.px))
                                font.weight: Tokens.weightTitle
                            }

                            Text {
                                text: Workspace.summaryLine
                                color: ink.muted
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(Tokens.fontBody * root.px))
                            }

                            Item {
                                Layout.fillWidth: true
                            }

                            Rectangle {
                                id: tClearFiltersBtn
                                visible: Workspace.searchQuery.length > 0 || Workspace.count !== Workspace.totalCount
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopClearFiltersLabel
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: ink.surfaceRaised
                                border.width: tClearFiltersBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                border.color: tClearFiltersBtn.activeFocus ? ink.corona : ink.pillBorder
                                implicitWidth: tClearText.implicitWidth + Math.round(20 * root.px)

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        Actions.workspaceClearFilters()
                                        event.accepted = true
                                    }
                                }

                                Text {
                                    id: tClearText
                                    anchors.centerIn: parent
                                    text: Words.desktopClearFiltersLabel
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                }

                                TapHandler {
                                    onTapped: Actions.workspaceClearFilters()
                                }
                            }
                        }

                        // Filter chips row.
                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(8 * root.px)

                            Repeater {
                                model: Workspace.filters
                                delegate: Rectangle {
                                    id: tFilterChip
                                    required property string key
                                    required property string label
                                    required property int count
                                    readonly property bool active: Workspace.filterKey === tFilterChip.key
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: tFilterChip.label
                                    height: Math.round(26 * root.px)
                                    radius: height / 2
                                    color: tFilterChip.active ? ink.textStrong : ink.glass
                                    border.width: tFilterChip.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: tFilterChip.activeFocus ? ink.corona : tFilterChip.active ? ink.textStrong : ink.hairline
                                    implicitWidth: tfRow.implicitWidth + Math.round(18 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceSetFilter(tFilterChip.key)
                                            event.accepted = true
                                        }
                                    }

                                    RowLayout {
                                        id: tfRow
                                        anchors.centerIn: parent
                                        spacing: Math.round(5 * root.px)

                                        Text {
                                            text: tFilterChip.label
                                            color: tFilterChip.active ? ink.surface : ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                            font.weight: Tokens.weightTitle
                                        }

                                        Text {
                                            text: String(tFilterChip.count)
                                            color: tFilterChip.active ? ink.surface : ink.dim
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceSetFilter(tFilterChip.key)
                                    }
                                }
                            }

                            Item {
                                Layout.fillWidth: true
                            }
                        }

                        // Sortable column headings bar.
                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(32 * root.px)
                            color: ink.glass
                            border.width: 1
                            border.color: ink.hairline
                            radius: Math.round(6 * root.px)

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Math.round(10 * root.px)
                                anchors.rightMargin: Math.round(10 * root.px)
                                spacing: Math.round(8 * root.px)

                                Rectangle {
                                    id: tSelectAllBtn
                                    objectName: "eclipseDesktopTableSelectAll"
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.CheckBox
                                    Accessible.name: Words.desktopTableColumns[0]
                                    Layout.preferredWidth: Math.round(18 * root.px)
                                    Layout.preferredHeight: Layout.preferredWidth
                                    radius: Math.round(4 * root.px)
                                    color: Workspace.allMarked ? ink.accent : "transparent"
                                    border.width: tSelectAllBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: tSelectAllBtn.activeFocus ? ink.corona : Workspace.allMarked ? ink.accent : ink.pillBorder

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceMarkAll()
                                            event.accepted = true
                                        }
                                    }

                                    DesktopGlyph {
                                        anchors.centerIn: parent
                                        width: Math.round(12 * root.px)
                                        height: width
                                        visible: Workspace.allMarked
                                        kind: 8
                                        color: ink.overAccent
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceMarkAll()
                                    }
                                }

                                Repeater {
                                    model: Words.desktopTableColumns
                                    delegate: Item {
                                        id: colHead
                                        required property string modelData
                                        required property int index
                                        readonly property bool sorted: Workspace.sortIndex === colHead.index
                                        activeFocusOnTab: true
                                        Accessible.role: Accessible.Button
                                        Accessible.name: colHead.modelData
                                        Layout.fillWidth: colHead.index === 0 || colHead.index === 1
                                        Layout.preferredWidth: colHead.index === 0 ? Math.round(220 * root.px) : colHead.index === 1 ? Math.round(130 * root.px) : colHead.index === 2 ? Math.round(108 * root.px) : colHead.index === 3 ? Math.round(58 * root.px) : colHead.index === 4 ? Math.round(98 * root.px) : colHead.index === 5 ? Math.round(88 * root.px) : colHead.index === 6 ? Math.round(78 * root.px) : colHead.index === 7 ? Math.round(72 * root.px) : Math.round(92 * root.px)
                                        Layout.fillHeight: true

                                        Keys.onPressed: (event) => {
                                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                Actions.workspaceSortByColumn(colHead.index < 7 ? colHead.index : 0)
                                                event.accepted = true
                                            }
                                        }

                                        Rectangle {
                                            anchors.fill: parent
                                            radius: Math.round(4 * root.px)
                                            visible: colHead.activeFocus
                                            color: "transparent"
                                            border.width: Math.max(1.5, Tokens.ringThickness)
                                            border.color: ink.corona
                                        }

                                        RowLayout {
                                            anchors.verticalCenter: parent.verticalCenter
                                            spacing: Math.round(4 * root.px)

                                            Text {
                                                text: colHead.modelData
                                                color: colHead.sorted ? ink.textStrong : ink.muted
                                                font.family: Tokens.faceInterface
                                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                                font.weight: Tokens.weightWordmark
                                                font.letterSpacing: 0.6
                                            }

                                            DesktopGlyph {
                                                visible: colHead.sorted
                                                Layout.preferredWidth: Math.round(11 * root.px)
                                                Layout.preferredHeight: Layout.preferredWidth
                                                kind: Workspace.sortAscending ? 12 : 13
                                                color: ink.corona
                                            }
                                        }

                                        TapHandler {
                                            onTapped: Actions.workspaceSortByColumn(colHead.index < 7 ? colHead.index : 0)
                                        }
                                    }
                                }
                            }
                        }

                        // Catalog rows.
                        EdgeFade {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            fadeTop: !tableRows.atYBeginning
                            fadeBottom: !tableRows.atYEnd
                            length: Math.round(14 * root.px)

                            ListView {
                                id: tableRows
                                objectName: "eclipseDesktopTableRows"
                                anchors.fill: parent
                                clip: true
                                spacing: Math.round(2 * root.px)
                                model: Workspace.games
                                readonly property bool fadeTop: !atYBeginning
                                readonly property bool fadeBottom: !atYEnd
                                ScrollBar.vertical: ScrollBar {
                                    policy: ScrollBar.AsNeeded
                                }

                                delegate: Rectangle {
                                    id: tRow
                                    required property int index
                                    required property var model
                                    required property string key
                                    required property string title
                                    required property string system
                                    required property string core
                                    required property int planet
                                    required property string year
                                    required property string genre
                                    required property string playedWords
                                    required property string statesLabel
                                    required property string format
                                    required property string statusLabel
                                    required property string cover
                                    required property bool favourite
                                    required property bool marked
                                    required property bool resumeReady
                                    readonly property bool selected: Workspace.selectedKey === tRow.key
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: tRow.title
                                    width: tableRows.width
                                    height: root.rowHeight
                                    radius: Math.round(6 * root.px)
                                    color: tRow.marked ? ink.glowWeak : tRow.selected ? ink.surfaceRaised : tRowHover.hovered ? ink.glass : "transparent"
                                    border.width: tRow.activeFocus ? Math.max(1.5, Tokens.ringThickness) : tRow.marked || tRow.selected ? 1 : 0
                                    border.color: tRow.activeFocus || tRow.marked ? ink.coronaSoft : ink.hairline

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                                            Actions.workspaceSelectGame(tRow.key)
                                            Actions.play(tRow.key)
                                            event.accepted = true
                                        } else if (event.key === Qt.Key_Space) {
                                            Actions.workspaceMarkToggle(tRow.key)
                                            event.accepted = true
                                        }
                                    }

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: Math.round(10 * root.px)
                                        anchors.rightMargin: Math.round(10 * root.px)
                                        spacing: Math.round(8 * root.px)

                                        Rectangle {
                                            id: tRowCheck
                                            activeFocusOnTab: true
                                            Accessible.role: Accessible.CheckBox
                                            Accessible.name: tRow.title
                                            Layout.preferredWidth: Math.round(18 * root.px)
                                            Layout.preferredHeight: Layout.preferredWidth
                                            radius: Math.round(4 * root.px)
                                            color: tRow.marked ? ink.accent : "transparent"
                                            border.width: tRowCheck.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                            border.color: tRowCheck.activeFocus ? ink.corona : tRow.marked ? ink.accent : ink.pillBorder

                                            Keys.onPressed: (event) => {
                                                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                    Actions.workspaceMarkToggle(tRow.key)
                                                    event.accepted = true
                                                }
                                            }

                                            DesktopGlyph {
                                                anchors.centerIn: parent
                                                width: Math.round(12 * root.px)
                                                height: width
                                                visible: tRow.marked
                                                kind: 8
                                                color: ink.overAccent
                                            }

                                            TapHandler {
                                                onTapped: Actions.workspaceMarkToggle(tRow.key)
                                            }
                                        }

                                        // Title + thumbnail.
                                        RowLayout {
                                            Layout.fillWidth: true
                                            Layout.preferredWidth: Math.round(220 * root.px)
                                            spacing: Math.round(10 * root.px)

                                            DesktopArtTile {
                                                Layout.preferredWidth: Math.round(26 * root.px)
                                                Layout.preferredHeight: Layout.preferredWidth
                                                px: root.px
                                                radius: 5
                                                source: tRow.cover
                                                planet: tRow.planet
                                                slot: tRow.index
                                                pixelScale: root.px
                                            }

                                            Text {
                                                Layout.fillWidth: true
                                                text: tRow.title
                                                elide: Text.ElideRight
                                                color: ink.textStrong
                                                font.family: Tokens.faceInterface
                                                font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                                font.weight: Tokens.weightTitle
                                            }

                                            Item {
                                                visible: tRow.favourite
                                                Layout.preferredWidth: Math.round(12 * root.px)
                                                Layout.preferredHeight: Layout.preferredWidth
                                                Shape {
                                                    width: 24
                                                    height: 24
                                                    scale: parent.width / 24
                                                    transformOrigin: Item.TopLeft
                                                    preferredRendererType: Shape.CurveRenderer
                                                    ShapePath {
                                                        fillColor: ink.accent
                                                        strokeColor: ink.accent
                                                        strokeWidth: 1.6
                                                        PathSvg { path: Marks.heart }
                                                    }
                                                }
                                            }
                                        }

                                        Text {
                                            Layout.fillWidth: true
                                            Layout.preferredWidth: Math.round(130 * root.px)
                                            text: tRow.model?.console ?? tRow.system
                                            elide: Text.ElideRight
                                            color: ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }

                                        RowLayout {
                                            Layout.preferredWidth: Math.round(108 * root.px)
                                            spacing: Math.round(6 * root.px)

                                            PlanetSphere {
                                                unit: root.px
                                                side: 12
                                                glowBlur: 5
                                                glowSpread: 1
                                                planet: tRow.planet
                                            }

                                            Text {
                                                Layout.fillWidth: true
                                                text: tRow.core
                                                elide: Text.ElideRight
                                                color: ink.text
                                                font.family: Tokens.faceInterface
                                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                            }
                                        }

                                        Text {
                                            Layout.preferredWidth: Math.round(58 * root.px)
                                            text: tRow.year.length > 0 ? tRow.year : Words.desktopDash
                                            color: ink.muted
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }

                                        Text {
                                            Layout.preferredWidth: Math.round(98 * root.px)
                                            text: tRow.genre.length > 0 ? tRow.genre : Words.desktopDash
                                            elide: Text.ElideRight
                                            color: ink.muted
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }

                                        Text {
                                            Layout.preferredWidth: Math.round(88 * root.px)
                                            text: tRow.playedWords
                                            color: ink.text
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }

                                        Text {
                                            Layout.preferredWidth: Math.round(78 * root.px)
                                            text: tRow.statesLabel
                                            color: ink.muted
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }

                                        Text {
                                            Layout.preferredWidth: Math.round(72 * root.px)
                                            text: tRow.format
                                            elide: Text.ElideRight
                                            color: ink.dim
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                        }

                                        Rectangle {
                                            Layout.preferredWidth: Math.round(92 * root.px)
                                            Layout.preferredHeight: Math.round(24 * root.px)
                                            radius: height / 2
                                            color: tRow.resumeReady ? ink.successFill : tRow.favourite ? ink.attentionFill : ink.surfaceRaised
                                            border.width: 1
                                            border.color: tRow.resumeReady ? ink.success : tRow.favourite ? ink.attention : ink.hairline

                                            Text {
                                                anchors.centerIn: parent
                                                text: tRow.statusLabel
                                                color: tRow.resumeReady ? ink.success : tRow.favourite ? ink.attention : ink.text
                                                font.family: Tokens.faceInterface
                                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                                font.weight: Tokens.weightTitle
                                            }
                                        }
                                    }

                                    HoverHandler {
                                        id: tRowHover
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceSelectGame(tRow.key)
                                        onDoubleTapped: Actions.play(tRow.key)
                                    }
                                }
                            }
                        }

                        // Floating batch action bar when one or more rows are checked.
                        Rectangle {
                            id: batchBar
                            objectName: "eclipseDesktopTableBatchBar"
                            visible: Workspace.markedCount > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(50 * root.px)
                            radius: Math.round(12 * root.px)
                            color: ink.glassStrong
                            border.width: 1
                            border.color: ink.corona

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Math.round(14 * root.px)
                                anchors.rightMargin: Math.round(14 * root.px)
                                spacing: Math.round(10 * root.px)

                                Rectangle {
                                    Layout.preferredHeight: Math.round(30 * root.px)
                                    radius: height / 2
                                    color: ink.accent
                                    implicitWidth: countBadgeWord.implicitWidth + Math.round(20 * root.px)

                                    Text {
                                        id: countBadgeWord
                                        anchors.centerIn: parent
                                        text: Words.games(Workspace.markedCount)
                                        color: ink.overAccent
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }
                                }

                                Rectangle {
                                    id: tBatchClearBtn
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: Words.desktopClearLabel
                                    Layout.preferredHeight: Math.round(30 * root.px)
                                    radius: height / 2
                                    color: "transparent"
                                    border.width: tBatchClearBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: tBatchClearBtn.activeFocus ? ink.corona : ink.hairline
                                    implicitWidth: clearSelLabel.implicitWidth + Math.round(18 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceClearMarked()
                                            event.accepted = true
                                        }
                                    }

                                    Text {
                                        id: clearSelLabel
                                        anchors.centerIn: parent
                                        text: Words.desktopClearLabel
                                        color: ink.muted
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceClearMarked()
                                    }
                                }

                                Item {
                                    Layout.fillWidth: true
                                }

                                Rectangle {
                                    id: tBatchFavBtn
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: Words.desktopFavouriteLabel
                                    Layout.preferredHeight: Math.round(32 * root.px)
                                    radius: height / 2
                                    color: ink.surfaceRaised
                                    border.width: tBatchFavBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: tBatchFavBtn.activeFocus ? ink.corona : ink.pillBorder
                                    implicitWidth: markFavText.implicitWidth + Math.round(20 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceBatchFavourite(true)
                                            event.accepted = true
                                        }
                                    }

                                    Text {
                                        id: markFavText
                                        anchors.centerIn: parent
                                        text: Words.desktopFavouriteLabel
                                        color: ink.textStrong
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceBatchFavourite(true)
                                    }
                                }

                                Rectangle {
                                    id: tBatchUnfavBtn
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: Words.desktopUnmarkFavLabel
                                    Layout.preferredHeight: Math.round(32 * root.px)
                                    radius: height / 2
                                    color: ink.surfaceRaised
                                    border.width: tBatchUnfavBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                                    border.color: tBatchUnfavBtn.activeFocus ? ink.corona : ink.pillBorder
                                    implicitWidth: unmarkFavText.implicitWidth + Math.round(20 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.workspaceBatchFavourite(false)
                                            event.accepted = true
                                        }
                                    }

                                    Text {
                                        id: unmarkFavText
                                        anchors.centerIn: parent
                                        text: Words.desktopUnmarkFavLabel
                                        color: ink.textStrong
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }

                                    TapHandler {
                                        onTapped: Actions.workspaceBatchFavourite(false)
                                    }
                                }

                                Rectangle {
                                    id: tBatchPlayBtn
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: Words.desktopPlayLabel
                                    Layout.preferredHeight: Math.round(32 * root.px)
                                    radius: height / 2
                                    color: ink.accent
                                    border.width: tBatchPlayBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 0
                                    border.color: ink.corona
                                    implicitWidth: playMarkedText.implicitWidth + Math.round(22 * root.px)

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            if (Workspace.markedKeys.length > 0) Actions.play(Workspace.markedKeys[0])
                                            event.accepted = true
                                        }
                                    }

                                    Text {
                                        id: playMarkedText
                                        anchors.centerIn: parent
                                        text: Words.desktopPlayLabel
                                        color: ink.overAccent
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }

                                    TapHandler {
                                        onTapped: if (Workspace.markedKeys.length > 0) Actions.play(Workspace.markedKeys[0])
                                    }
                                }
                            }
                        }
                    }
                }
            }

            DesktopStatusBar {
                px: root.px
            }
        }

        Shortcut {
            enabled: root.visible && !root.inputFocused
            sequence: "Ctrl+A"
            onActivated: Actions.workspaceMarkAll()
        }
    }
}
