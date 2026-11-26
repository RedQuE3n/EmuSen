pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Reference Desktop Mode workspace shell: top studio bar, active Desktop view host, and bottom status bar.
Item {
    id: shellRoot
    objectName: "desktopShell"
    focus: true
    required property DesktopModel model
    required property SessionModel session
    required property SettingsModel settings
    required property LibraryModel library
    required property string platformLibrary
    required property var builtInManifests
    property bool themedViewActive: false

    signal bigPictureRequested()
    signal fullScreenRequested()
    signal libraryRequested()
    signal themeUsed(string folder)
    signal themeRestyled(bool esde)
    signal displaySettingsRequested()

    readonly property var chrome: JSON.parse(shellRoot.model.chromeJson)
    readonly property var consoles: JSON.parse(shellRoot.model.consolesJson)
    readonly property var games: JSON.parse(shellRoot.model.gamesJson)
    readonly property var inspector: JSON.parse(shellRoot.model.inspectorJson)
    readonly property var playing: JSON.parse(shellRoot.model.playingJson)
    readonly property var settingsData: JSON.parse(shellRoot.model.settingsJson)
    readonly property real px: Math.max(0.64, Math.min(1.30, shellRoot.height / Kit.CanvasUnits.designHeight))

    // Workspace backdrop in the active theme palette (hidden when a custom Schema 2 Desktop slot or game is foregrounded).
    Rectangle {
        id: workspaceBackground
        objectName: "desktopWorkspaceBg"
        anchors.fill: parent
        visible: !shellRoot.themedViewActive && !shellRoot.model.gameShown
        color: Kit.Tokens.voidFill

        Rectangle {
            visible: Kit.Tokens.coronaRim > 0
            anchors.left: parent.left
            anchors.top: parent.top
            width: Math.round(520 * shellRoot.px)
            height: width
            radius: width / 2
            x: -width * 0.45
            y: -height * 0.45
            color: Qt.alpha(ThemeInk.backdropGlow, 0.10)
        }

        Rectangle {
            visible: Kit.Tokens.coronaRim > 0
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            width: Math.round(560 * shellRoot.px)
            height: width
            radius: width / 2
            x: shellRoot.width - width * 0.55
            y: shellRoot.height - height * 0.55
            color: Qt.alpha(ThemeInk.backdropHalo, 0.10)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        visible: !shellRoot.themedViewActive && !shellRoot.model.gameShown

        // Top Desktop Bar: wordmark, mode badge, view navigation, search box, session resume, Big Picture button.
        Rectangle {
            id: topBar
            objectName: "desktopTopBar"
            Layout.fillWidth: true
            Layout.preferredHeight: Math.round(52 * shellRoot.px)
            color: ThemeInk.glassStrong
            border.width: 1
            border.color: ThemeInk.hairline

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Math.round(14 * shellRoot.px)
                anchors.rightMargin: Math.round(14 * shellRoot.px)
                spacing: Math.round(10 * shellRoot.px)

                // Wordmark and active look badge.
                RowLayout {
                    spacing: Math.round(8 * shellRoot.px)

                    Text {
                        objectName: "desktopWordmark"
                        text: shellRoot.chrome.wordmark ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(14, Math.round(17 * shellRoot.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    Rectangle {
                        objectName: "desktopThemeBadge"
                        Layout.preferredHeight: Math.round(22 * shellRoot.px)
                        radius: height / 2
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.coronaSoft
                        implicitWidth: badgeWord.implicitWidth + Math.round(16 * shellRoot.px)

                        Text {
                            id: badgeWord
                            anchors.centerIn: parent
                            text: shellRoot.model.modeBadge
                            color: ThemeInk.corona
                            font.family: Kit.Tokens.faceCode
                            font.pixelSize: Math.max(9, Math.round(10 * shellRoot.px))
                            font.weight: Kit.Tokens.weightWordmark
                        }
                    }
                }

                Rectangle {
                    Layout.preferredWidth: Math.round(30 * shellRoot.px)
                    Layout.preferredHeight: Layout.preferredWidth
                    radius: Math.round(7 * shellRoot.px)
                    color: shellRoot.model.sidebarOpen ? ThemeInk.surfaceRaised : "transparent"
                    border.width: 1
                    border.color: ThemeInk.hairline

                    Item {
                        anchors.centerIn: parent
                        width: Math.round(15 * shellRoot.px)
                        height: width
                        Shape {
                            width: 24
                            height: 24
                            scale: parent.width / 24
                            transformOrigin: Item.TopLeft
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                fillColor: "transparent"
                                strokeColor: ThemeInk.textStrong
                                strokeWidth: 1.7
                                PathSvg { path: shellRoot.chrome.icons?.sidebar ?? "" }
                            }
                        }
                    }

                    TapHandler {
                        onTapped: shellRoot.model.toggleSidebar()
                    }
                }

                // Navigation tabs: Library (0), Table (1), Settings (2).
                RowLayout {
                    spacing: Math.round(4 * shellRoot.px)

                    Repeater {
                        model: shellRoot.chrome.navTabs ?? []
                        delegate: Rectangle {
                            id: navTab
                            required property string modelData
                            required property int index
                            readonly property bool active: shellRoot.model.view === navTab.index
                            objectName: (shellRoot.chrome.navObjectNames ?? [])[navTab.index] ?? ""
                            height: Math.round(32 * shellRoot.px)
                            radius: height / 2
                            color: navTab.active ? ThemeInk.glowWeak : navHover.hovered ? ThemeInk.surfaceRaised : "transparent"
                            border.width: navTab.active ? 1 : 0
                            border.color: ThemeInk.corona
                            implicitWidth: navTxt.implicitWidth + Math.round(24 * shellRoot.px)

                            Text {
                                id: navTxt
                                anchors.centerIn: parent
                                text: navTab.modelData
                                color: navTab.active ? ThemeInk.textStrong : ThemeInk.muted
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(12, Math.round(13 * shellRoot.px))
                                font.weight: navTab.active ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                            }

                            HoverHandler {
                                id: navHover
                            }

                            TapHandler {
                                onTapped: shellRoot.model.view = navTab.index
                            }
                        }
                    }
                }

                Item {
                    Layout.fillWidth: true
                }

                // Global filter search field.
                Rectangle {
                    Layout.preferredWidth: Math.round(340 * shellRoot.px)
                    Layout.preferredHeight: Math.round(34 * shellRoot.px)
                    radius: Math.round(8 * shellRoot.px)
                    color: ThemeInk.inputSurface
                    border.width: 1
                    border.color: globalSearchInput.activeFocus ? ThemeInk.corona : ThemeInk.hairline

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(10 * shellRoot.px)
                        anchors.rightMargin: Math.round(10 * shellRoot.px)
                        spacing: Math.round(8 * shellRoot.px)

                        Item {
                            Layout.preferredWidth: Math.round(14 * shellRoot.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            Shape {
                                width: 24
                                height: 24
                                scale: parent.width / 24
                                transformOrigin: Item.TopLeft
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    fillColor: "transparent"
                                    strokeColor: ThemeInk.muted
                                    strokeWidth: 1.8
                                    PathSvg { path: shellRoot.chrome.icons?.search ?? "" }
                                }
                            }
                        }

                        TextField {
                            id: globalSearchInput
                            objectName: "desktopGlobalSearch"
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            color: ThemeInk.textStrong
                            placeholderTextColor: ThemeInk.dim
                            placeholderText: shellRoot.chrome.searchPlaceholder ?? ""
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * shellRoot.px))
                            text: shellRoot.model.searchQuery
                            background: Item {}
                            signal syncModelText()
                            onSyncModelText: if (text !== shellRoot.model.searchQuery) text = shellRoot.model.searchQuery
                            onTextChanged: {
                                if (text !== shellRoot.model.searchQuery) {
                                    shellRoot.model.searchQuery = text
                                    if (text.length > 0 && shellRoot.model.view === 2) shellRoot.model.view = 0
                                }
                            }
                            Connections {
                                target: shellRoot.model
                                function onSearchQueryChanged() { globalSearchInput.syncModelText() }
                            }
                        }
                    }
                }

                Item {
                    Layout.fillWidth: true
                }

                // Resume button while a game is suspended in memory.
                Rectangle {
                    visible: shellRoot.model.hasGame
                    Layout.preferredHeight: Math.round(30 * shellRoot.px)
                    radius: height / 2
                    color: Kit.Tokens.accent
                    implicitWidth: resTxt.implicitWidth + Math.round(20 * shellRoot.px)

                    Text {
                        id: resTxt
                        anchors.centerIn: parent
                        text: shellRoot.chrome.resumeLabel ?? ""
                        color: Kit.Tokens.overAccent
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * shellRoot.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    TapHandler {
                        onTapped: shellRoot.model.resumeRequested()
                    }
                }

                Rectangle {
                    visible: shellRoot.model.view === 0 || shellRoot.model.view === 2
                    Layout.preferredWidth: Math.round(30 * shellRoot.px)
                    Layout.preferredHeight: Layout.preferredWidth
                    radius: Math.round(7 * shellRoot.px)
                    color: shellRoot.model.inspectorOpen ? ThemeInk.surfaceRaised : "transparent"
                    border.width: 1
                    border.color: ThemeInk.hairline

                    Item {
                        anchors.centerIn: parent
                        width: Math.round(15 * shellRoot.px)
                        height: width
                        Shape {
                            width: 24
                            height: 24
                            scale: parent.width / 24
                            transformOrigin: Item.TopLeft
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                fillColor: "transparent"
                                strokeColor: ThemeInk.textStrong
                                strokeWidth: 1.7
                                PathSvg { path: shellRoot.chrome.icons?.inspector ?? "" }
                            }
                        }
                    }

                    TapHandler {
                        onTapped: shellRoot.model.toggleInspector()
                    }
                }

                // Big Picture toggle pill (F10).
                Rectangle {
                    id: bigPictureBtn
                    objectName: "desktopBigPictureButton"
                    Layout.preferredHeight: Math.round(32 * shellRoot.px)
                    radius: height / 2
                    color: bpHover.hovered ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.pillBorder
                    implicitWidth: bpRow.implicitWidth + Math.round(22 * shellRoot.px)

                    RowLayout {
                        id: bpRow
                        anchors.centerIn: parent
                        spacing: Math.round(7 * shellRoot.px)

                        Text {
                            text: shellRoot.chrome.bigPictureLabel ?? ""
                            color: bpHover.hovered ? Kit.Tokens.voidFill : ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * shellRoot.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        Text {
                            text: shellRoot.chrome.bigPictureHint ?? ""
                            color: bpHover.hovered ? Kit.Tokens.voidFill : ThemeInk.muted
                            font.family: Kit.Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                        }
                    }

                    HoverHandler {
                        id: bpHover
                    }

                    TapHandler {
                        onTapped: shellRoot.bigPictureRequested()
                    }
                }
            }
        }

        // Main Workspace Stage (Library, Table, Settings).
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            DesktopLibrary {
                anchors.fill: parent
                visible: shellRoot.model.view === 0
                model: shellRoot.model
                chrome: shellRoot.chrome
                consoles: shellRoot.consoles
                games: shellRoot.games
                inspector: shellRoot.inspector
                px: shellRoot.px
            }

            DesktopTable {
                anchors.fill: parent
                visible: shellRoot.model.view === 1
                model: shellRoot.model
                chrome: shellRoot.chrome
                consoles: shellRoot.consoles
                games: shellRoot.games
                px: shellRoot.px
            }

            DesktopSettings {
                anchors.fill: parent
                visible: shellRoot.model.view === 2
                model: shellRoot.model
                settings: shellRoot.settings
                library: shellRoot.library
                platformLibrary: shellRoot.platformLibrary
                builtInManifests: shellRoot.builtInManifests
                chrome: shellRoot.chrome
                settingsData: shellRoot.settingsData
                px: shellRoot.px
                onThemeUsed: (folder) => shellRoot.themeUsed(folder)
                onThemeRestyled: (esde) => shellRoot.themeRestyled(esde)
            }
        }

        // Bottom Desktop Status Bar.
        Rectangle {
            objectName: "desktopStatusBar"
            Layout.fillWidth: true
            Layout.preferredHeight: Math.round(28 * shellRoot.px)
            color: ThemeInk.glassStrong
            border.width: 1
            border.color: ThemeInk.hairline

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Math.round(14 * shellRoot.px)
                anchors.rightMargin: Math.round(14 * shellRoot.px)
                spacing: Math.round(12 * shellRoot.px)

                Text {
                    objectName: "desktopStatusText"
                    Layout.fillWidth: true
                    elide: Text.ElideRight
                    text: shellRoot.model.statusText
                    color: ThemeInk.text
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                }

                Text {
                    text: shellRoot.chrome.statusHints ?? ""
                    color: ThemeInk.dim
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                }

                Text {
                    visible: shellRoot.model.battery >= 0
                    text: (shellRoot.chrome.batteryPrefix ?? "") + String(shellRoot.model.battery) + (shellRoot.chrome.percent ?? "")
                    color: ThemeInk.dim
                    font.family: Kit.Tokens.faceCode
                    font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                }

                Text {
                    visible: shellRoot.model.clockText.length > 0
                    text: shellRoot.model.clockText
                    color: ThemeInk.text
                    font.family: Kit.Tokens.faceCode
                    font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                }

                Text {
                    objectName: "desktopStatusSummary"
                    text: shellRoot.model.statusSummary
                    color: ThemeInk.muted
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * shellRoot.px))
                }
            }
        }
    }

    // Playing studio strip & drawer over live gameplay when no custom Schema 2 DesktopPlaying slot overrides it.
    DesktopPlaying {
        anchors.fill: parent
        visible: !shellRoot.themedViewActive && shellRoot.model.gameShown
        model: shellRoot.model
        session: shellRoot.session
        chrome: shellRoot.chrome
        playing: shellRoot.playing
        px: shellRoot.px
        onLibraryRequested: {
            shellRoot.model.view = 0
            shellRoot.libraryRequested()
        }
        onFullScreenRequested: shellRoot.fullScreenRequested()
        onBigPictureRequested: shellRoot.bigPictureRequested()
        onResumeRequested: shellRoot.model.resumeRequested()
        onDisplaySettingsRequested: {
            shellRoot.session.pause()
            shellRoot.model.openSettingsTool(1)
            shellRoot.model.view = 2
            shellRoot.displaySettingsRequested()
        }
    }

    Shortcut {
        enabled: shellRoot.visible && !shellRoot.model.gameShown
        sequence: "Ctrl+K"
        onActivated: globalSearchInput.forceActiveFocus()
    }

    Shortcut {
        enabled: shellRoot.visible && !shellRoot.model.gameShown
        sequence: "Escape"
        onActivated: {
            if (shellRoot.model.searchQuery.length > 0) shellRoot.model.clearFilters()
        }
    }
}
