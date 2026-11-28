pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme

// Top studio ribbon shared across Eclipse Desktop views.
Rectangle {
    id: root
    required property real px
    property bool showSidebarToggle: true
    property bool showInspectorToggle: Workspace.view !== 1
    property bool showViewSwitcher: Workspace.view !== 2
    readonly property bool settingsMode: Workspace.view === 2

    objectName: "eclipseDesktopRibbon"
    Layout.fillWidth: true
    Layout.preferredHeight: Math.round(52 * root.px)
    color: ink.glassStrong
    border.width: 1
    border.color: ink.hairline

    DesktopInk {
        id: ink
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Math.round(14 * root.px)
        anchors.rightMargin: Math.round(14 * root.px)
        spacing: Math.round(10 * root.px)

        // Crescent mark, wordmark, and Eclipse mode badge.
        RowLayout {
            spacing: Math.round(8 * root.px)

            Item {
                Layout.preferredWidth: Math.round(18 * root.px)
                Layout.preferredHeight: Layout.preferredWidth
                Shape {
                    width: 24
                    height: 24
                    scale: parent.width / 24
                    transformOrigin: Item.TopLeft
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        fillColor: Tokens.accent
                        strokeColor: "transparent"
                        fillRule: ShapePath.OddEvenFill
                        PathSvg { path: Marks.crescent }
                    }
                }
            }

            Text {
                objectName: "eclipseDesktopWordmark"
                text: Words.name
                color: ink.textStrong
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(14, Math.round(16 * root.px))
                font.weight: Tokens.weightTitle
            }

            Rectangle {
                objectName: "eclipseDesktopBadgePill"
                Layout.preferredHeight: Math.round(22 * root.px)
                radius: height / 2
                color: ink.surfaceRaised
                border.width: 1
                border.color: ink.coronaSoft
                implicitWidth: badgeWord.implicitWidth + Math.round(16 * root.px)

                Text {
                    id: badgeWord
                    objectName: "eclipseDesktopBadge"
                    anchors.centerIn: parent
                    text: Workspace.modeBadge
                    color: ink.corona
                    font.family: Tokens.faceCode
                    font.pixelSize: Math.max(9, Math.round(10 * root.px))
                    font.weight: Tokens.weightWordmark
                }
            }
        }

        // Sidebar collapse button.
        Rectangle {
            visible: root.showSidebarToggle
            objectName: "eclipseDesktopSidebarBtn"
            activeFocusOnTab: true
            Accessible.role: Accessible.Button
            Accessible.name: Words.desktopSidebarTip
            Layout.preferredWidth: Math.round(30 * root.px)
            Layout.preferredHeight: Layout.preferredWidth
            radius: Math.round(7 * root.px)
            color: Workspace.sidebarOpen ? ink.surfaceRaised : sideHover.hovered ? ink.surfaceRaised : "transparent"
            border.width: 1
            border.color: sideHover.hovered ? ink.pillBorder : ink.hairline

            DesktopGlyph {
                anchors.centerIn: parent
                width: Math.round(15 * root.px)
                height: width
                kind: 0
                color: ink.textStrong
            }

            HoverHandler {
                id: sideHover
            }

            TapHandler {
                onTapped: Actions.workspaceSetSidebar(!Workspace.sidebarOpen)
            }
        }

        // Primary navigation pills.
        RowLayout {
            spacing: Math.round(4 * root.px)

            // Library tab.
            Rectangle {
                id: libTab
                objectName: "eclipseDesktopNavLibrary"
                readonly property bool active: !root.settingsMode
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopNavTabs[0]
                Layout.preferredHeight: Math.round(32 * root.px)
                radius: height / 2
                color: libTab.active ? ink.glowWeak : libTabHover.hovered ? ink.surfaceRaised : "transparent"
                border.width: libTab.active ? 1 : 0
                border.color: ink.corona
                implicitWidth: libTabTxt.implicitWidth + Math.round(22 * root.px)

                Text {
                    id: libTabTxt
                    anchors.centerIn: parent
                    text: Words.desktopNavTabs[0]
                    color: libTab.active ? ink.textStrong : ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(12, Math.round(13 * root.px))
                    font.weight: libTab.active ? Tokens.weightTitle : Tokens.weightSentence
                }

                HoverHandler {
                    id: libTabHover
                }

                TapHandler {
                    onTapped: Actions.workspaceView(0)
                }
            }

            // Cores quick-entry from Tools directory when populated.
            Repeater {
                model: Tools.entries
                delegate: Rectangle {
                    id: coreQuickBtn
                    required property int index
                    required property string key
                    required property string name
                    required property bool attention
                    visible: coreQuickBtn.index === 0
                    objectName: "eclipseDesktopNavCores"
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: coreQuickBtn.name
                    Layout.preferredHeight: Math.round(32 * root.px)
                    radius: height / 2
                    color: coreQuickHover.hovered ? ink.surfaceRaised : "transparent"
                    implicitWidth: visible ? coreQuickRow.implicitWidth + Math.round(20 * root.px) : 0

                    RowLayout {
                        id: coreQuickRow
                        anchors.centerIn: parent
                        spacing: Math.round(6 * root.px)

                        Text {
                            text: coreQuickBtn.name
                            color: coreQuickHover.hovered ? ink.textStrong : ink.muted
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(12, Math.round(13 * root.px))
                            font.weight: Tokens.weightSentence
                        }

                        Rectangle {
                            visible: coreQuickBtn.attention
                            Layout.preferredWidth: Math.round(6 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            radius: width / 2
                            color: ink.rose
                        }
                    }

                    HoverHandler {
                        id: coreQuickHover
                    }

                    TapHandler {
                        onTapped: Actions.openTool(coreQuickBtn.key)
                    }
                }
            }

            // Shaders quick-door button.
            Rectangle {
                id: shadersTab
                objectName: "eclipseDesktopNavShaders"
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopStripShaders
                Layout.preferredHeight: Math.round(32 * root.px)
                radius: height / 2
                color: shadersHover.hovered ? ink.surfaceRaised : "transparent"
                implicitWidth: shadersTxt.implicitWidth + Math.round(20 * root.px)

                Text {
                    id: shadersTxt
                    anchors.centerIn: parent
                    text: Words.desktopStripShaders
                    color: shadersHover.hovered ? ink.textStrong : ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(12, Math.round(13 * root.px))
                    font.weight: Tokens.weightSentence
                }

                HoverHandler {
                    id: shadersHover
                }

                TapHandler {
                    onTapped: Actions.openDesktopDoor(Words.desktopStripShaders.toLowerCase())
                }
            }

            // Settings & Tools tab.
            Rectangle {
                id: setTab
                objectName: "eclipseDesktopNavSettings"
                readonly property bool active: root.settingsMode
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopNavTabs[2]
                Layout.preferredHeight: Math.round(32 * root.px)
                radius: height / 2
                color: setTab.active ? ink.glowWeak : setTabHover.hovered ? ink.surfaceRaised : "transparent"
                border.width: setTab.active ? 1 : 0
                border.color: ink.corona
                implicitWidth: setTabTxt.implicitWidth + Math.round(22 * root.px)

                Text {
                    id: setTabTxt
                    anchors.centerIn: parent
                    text: Words.desktopNavTabs[2]
                    color: setTab.active ? ink.textStrong : ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(12, Math.round(13 * root.px))
                    font.weight: setTab.active ? Tokens.weightTitle : Tokens.weightSentence
                }

                HoverHandler {
                    id: setTabHover
                }

                TapHandler {
                    onTapped: Actions.workspaceView(2)
                }
            }
        }

        Item {
            Layout.fillWidth: true
        }

        // Global search / filter field.
        Rectangle {
            Layout.fillWidth: true
            Layout.minimumWidth: Math.round(160 * root.px)
            Layout.maximumWidth: Math.round(340 * root.px)
            Layout.preferredWidth: Math.round(290 * root.px)
            Layout.preferredHeight: Math.round(34 * root.px)
            radius: Math.round(8 * root.px)
            color: ink.inputSurface
            border.width: 1
            border.color: globalSearchInput.activeFocus ? ink.corona : ink.hairline

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Math.round(10 * root.px)
                anchors.rightMargin: Math.round(10 * root.px)
                spacing: Math.round(8 * root.px)

                DesktopGlyph {
                    Layout.preferredWidth: Math.round(14 * root.px)
                    Layout.preferredHeight: Layout.preferredWidth
                    kind: 2
                    color: ink.muted
                }

                TextField {
                    id: globalSearchInput
                    objectName: "eclipseDesktopGlobalSearch"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: ink.textStrong
                    placeholderTextColor: ink.dim
                    placeholderText: root.settingsMode ? Words.desktopSettingsFilterPlaceholder : Words.desktopSearchPlaceholder
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    text: root.settingsMode ? Settings.filter : Workspace.searchQuery
                    background: Item {}
                    signal syncModelText()
                    onSyncModelText: if (text !== (root.settingsMode ? Settings.filter : Workspace.searchQuery)) text = root.settingsMode ? Settings.filter : Workspace.searchQuery
                    onTextChanged: if (root.settingsMode) { if (text !== Settings.filter) Actions.settingsSetFilter(text) } else { if (text !== Workspace.searchQuery) Actions.workspaceSetSearch(text) }
                    Connections {
                        target: Workspace
                        function onChanged() { globalSearchInput.syncModelText() }
                    }
                    Connections {
                        target: Settings
                        function onShown() { globalSearchInput.syncModelText() }
                    }
                }
            }
        }

        Item {
            Layout.fillWidth: true
        }

        // Resume pill when a session is suspended.
        Rectangle {
            visible: Session.hasGame
            objectName: "eclipseDesktopResumeBtn"
            activeFocusOnTab: true
            Accessible.role: Accessible.Button
            Accessible.name: Words.desktopResumeLabel
            Layout.preferredHeight: Math.round(30 * root.px)
            radius: height / 2
            color: ink.accent
            implicitWidth: resTxt.implicitWidth + Math.round(20 * root.px)

            Text {
                id: resTxt
                anchors.centerIn: parent
                text: Words.desktopResumeLabel
                color: ink.overAccent
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                font.weight: Tokens.weightTitle
            }

            TapHandler {
                onTapped: Actions.play(Session.key)
            }
        }

        // Grid / Table segmented view switcher.
        Rectangle {
            visible: root.showViewSwitcher
            Layout.preferredHeight: Math.round(32 * root.px)
            radius: Math.round(8 * root.px)
            color: ink.inputSurface
            border.width: 1
            border.color: ink.hairline
            implicitWidth: viewSwitchRow.implicitWidth + Math.round(6 * root.px)

            RowLayout {
                id: viewSwitchRow
                anchors.centerIn: parent
                spacing: Math.round(3 * root.px)

                Rectangle {
                    id: gridSwitch
                    objectName: "eclipseDesktopViewGrid"
                    readonly property bool active: Workspace.view === 0
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopViewModes[0]
                    Layout.preferredHeight: Math.round(26 * root.px)
                    radius: Math.round(6 * root.px)
                    color: gridSwitch.active ? ink.surfaceRaised : "transparent"
                    border.width: gridSwitch.active ? 1 : 0
                    border.color: ink.coronaSoft
                    implicitWidth: gridSwRow.implicitWidth + Math.round(14 * root.px)

                    RowLayout {
                        id: gridSwRow
                        anchors.centerIn: parent
                        spacing: Math.round(5 * root.px)

                        DesktopGlyph {
                            Layout.preferredWidth: Math.round(12 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            kind: 10
                            color: gridSwitch.active ? ink.textStrong : ink.muted
                        }

                        Text {
                            text: Words.desktopViewModes[0]
                            color: gridSwitch.active ? ink.textStrong : ink.muted
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: gridSwitch.active ? Tokens.weightTitle : Tokens.weightSentence
                        }
                    }

                    TapHandler {
                        onTapped: Actions.workspaceView(0)
                    }
                }

                Rectangle {
                    id: tableSwitch
                    objectName: "eclipseDesktopViewTable"
                    readonly property bool active: Workspace.view === 1
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopViewModes[1]
                    Layout.preferredHeight: Math.round(26 * root.px)
                    radius: Math.round(6 * root.px)
                    color: tableSwitch.active ? ink.surfaceRaised : "transparent"
                    border.width: tableSwitch.active ? 1 : 0
                    border.color: ink.coronaSoft
                    implicitWidth: tableSwRow.implicitWidth + Math.round(14 * root.px)

                    RowLayout {
                        id: tableSwRow
                        anchors.centerIn: parent
                        spacing: Math.round(5 * root.px)

                        DesktopGlyph {
                            Layout.preferredWidth: Math.round(12 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            kind: 11
                            color: tableSwitch.active ? ink.textStrong : ink.muted
                        }

                        Text {
                            text: Words.desktopViewModes[1]
                            color: tableSwitch.active ? ink.textStrong : ink.muted
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: tableSwitch.active ? Tokens.weightTitle : Tokens.weightSentence
                        }
                    }

                    TapHandler {
                        onTapped: Actions.workspaceView(1)
                    }
                }
            }
        }

        // Inspector toggle button.
        Rectangle {
            visible: root.showInspectorToggle
            objectName: "eclipseDesktopInspectorBtn"
            activeFocusOnTab: true
            Accessible.role: Accessible.Button
            Accessible.name: Words.desktopInspectorTip
            Layout.preferredWidth: Math.round(30 * root.px)
            Layout.preferredHeight: Layout.preferredWidth
            radius: Math.round(7 * root.px)
            color: Workspace.inspectorOpen ? ink.surfaceRaised : inspHover.hovered ? ink.surfaceRaised : "transparent"
            border.width: 1
            border.color: inspHover.hovered ? ink.pillBorder : ink.hairline

            DesktopGlyph {
                anchors.centerIn: parent
                width: Math.round(15 * root.px)
                height: width
                kind: 1
                color: ink.textStrong
            }

            HoverHandler {
                id: inspHover
            }

            TapHandler {
                onTapped: Actions.workspaceSetInspector(!Workspace.inspectorOpen)
            }
        }

        // Big Picture switch pill.
        Rectangle {
            id: bigPictureBtn
            objectName: "eclipseDesktopBigPictureBtn"
            activeFocusOnTab: true
            Accessible.role: Accessible.Button
            Accessible.name: Words.desktopBigPictureLabel
            Layout.preferredHeight: Math.round(32 * root.px)
            radius: height / 2
            color: bpHover.hovered ? ink.textStrong : ink.surfaceRaised
            border.width: 1
            border.color: ink.pillBorder
            implicitWidth: bpRow.implicitWidth + Math.round(20 * root.px)

            RowLayout {
                id: bpRow
                anchors.centerIn: parent
                spacing: Math.round(6 * root.px)

                Text {
                    text: Words.desktopBigPictureLabel
                    color: bpHover.hovered ? ink.surface : ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    font.weight: Tokens.weightTitle
                }

                Text {
                    text: Workspace.bigPictureHint
                    color: bpHover.hovered ? ink.surface : ink.muted
                    font.family: Tokens.faceCode
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                }
            }

            HoverHandler {
                id: bpHover
            }

            TapHandler {
                onTapped: Actions.setBigScreen(true)
            }
        }
    }
}
