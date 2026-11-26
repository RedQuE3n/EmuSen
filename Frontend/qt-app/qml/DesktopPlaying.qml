pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Reference Desktop Mode Playing HUD: compact top control strip and collapsible Studio Drawer over live gameplay.
Item {
    id: root
    objectName: "desktopPlaying"
    required property DesktopModel model
    required property SessionModel session
    required property var chrome
    required property var playing
    required property real px

    signal libraryRequested()
    signal fullScreenRequested()
    signal bigPictureRequested()
    signal displaySettingsRequested()

    // Compact top control strip matching desktop playing studio layout.
    Rectangle {
        id: topStrip
        objectName: "desktopPlayingStrip"
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: Math.round(44 * root.px)
        color: ThemeInk.glassStrong
        border.width: 1
        border.color: ThemeInk.hairline

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: Math.round(12 * root.px)
            anchors.rightMargin: Math.round(12 * root.px)
            spacing: Math.round(8 * root.px)

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: height / 2
                color: libHover.hovered ? ThemeInk.surfaceRaised : ThemeInk.glass
                border.width: 1
                border.color: ThemeInk.pillBorder
                implicitWidth: backRow.implicitWidth + Math.round(18 * root.px)

                RowLayout {
                    id: backRow
                    anchors.centerIn: parent
                    spacing: Math.round(4 * root.px)
                    Item {
                        Layout.preferredWidth: Math.round(12 * root.px)
                        Layout.preferredHeight: Layout.preferredWidth
                        Shape {
                            width: 24
                            height: 24
                            scale: parent.width / 24
                            transformOrigin: Item.TopLeft
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                fillColor: "transparent"
                                strokeColor: ThemeInk.textStrong
                                strokeWidth: 2
                                PathSvg { path: root.chrome.icons?.chevronLeft ?? "" }
                            }
                        }
                    }
                    Text {
                        text: root.chrome.stripLibrary ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Kit.Tokens.weightTitle
                    }
                }

                HoverHandler {
                    id: libHover
                }

                TapHandler {
                    onTapped: root.libraryRequested()
                }
            }

            Text {
                text: root.playing.title ?? ""
                elide: Text.ElideRight
                Layout.maximumWidth: Math.round(260 * root.px)
                color: ThemeInk.textStrong
                font.family: Kit.Tokens.faceInterface
                font.pixelSize: Math.max(12, Math.round(14 * root.px))
                font.weight: Kit.Tokens.weightTitle
            }

            Rectangle {
                visible: (root.playing.core ?? "").length > 0
                Layout.preferredHeight: Math.round(24 * root.px)
                radius: height / 2
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: chipRow.implicitWidth + Math.round(16 * root.px)

                RowLayout {
                    id: chipRow
                    anchors.centerIn: parent
                    spacing: Math.round(6 * root.px)
                    CardPlanet {
                        unit: root.px
                        side: 12
                        planet: root.playing.planet ?? -2
                    }
                    Text {
                        text: root.playing.core ?? ""
                        color: ThemeInk.text
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    }
                    Text {
                        visible: (root.playing.fpsWords ?? "").length > 0
                        text: root.playing.fpsWords ?? ""
                        color: ThemeInk.success
                        font.family: Kit.Tokens.faceCode
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    }
                }
            }

            Item {
                Layout.fillWidth: true
            }

            // Playback controls: Pause/Resume and Rewind.
            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: pauseTxt.implicitWidth + Math.round(18 * root.px)

                Text {
                    id: pauseTxt
                    anchors.centerIn: parent
                    text: root.session.paused ? (root.chrome.stripResume ?? "") : (root.chrome.stripPause ?? "")
                    color: ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    font.weight: Kit.Tokens.weightTitle
                }

                TapHandler {
                    onTapped: if (root.session.paused) root.session.resume(); else root.session.pause()
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                opacity: (root.playing.hasRewind ?? false) ? 1.0 : 0.48
                implicitWidth: rewindTxt.implicitWidth + Math.round(18 * root.px)

                Text {
                    id: rewindTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripRewind ?? ""
                    color: ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                TapHandler {
                    onTapped: if (root.playing.hasRewind ?? false) root.session.rewindTo(root.playing.latestRewindFrame ?? 0)
                }
            }

            // Drawer tabs: States (1), Shaders (0), Cheats (2), Options (3).
            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: root.model.drawerOpen && root.model.drawerTab === 1 ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: statesTabTxt.implicitWidth + Math.round(18 * root.px)
                Text {
                    id: statesTabTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripStates ?? ""
                    color: root.model.drawerOpen && root.model.drawerTab === 1 ? Kit.Tokens.voidFill : ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.model.openDrawerTab(1)
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: root.model.drawerOpen && root.model.drawerTab === 0 ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: shadersTabTxt.implicitWidth + Math.round(18 * root.px)
                Text {
                    id: shadersTabTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripShaders ?? ""
                    color: root.model.drawerOpen && root.model.drawerTab === 0 ? Kit.Tokens.voidFill : ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.model.openDrawerTab(0)
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: root.model.drawerOpen && root.model.drawerTab === 2 ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: cheatsTabTxt.implicitWidth + Math.round(18 * root.px)
                Text {
                    id: cheatsTabTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripCheats ?? ""
                    color: root.model.drawerOpen && root.model.drawerTab === 2 ? Kit.Tokens.voidFill : ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.model.openDrawerTab(2)
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: root.model.drawerOpen && root.model.drawerTab === 3 ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: optsTabTxt.implicitWidth + Math.round(18 * root.px)
                Text {
                    id: optsTabTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripControls ?? ""
                    color: root.model.drawerOpen && root.model.drawerTab === 3 ? Kit.Tokens.voidFill : ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.model.openDrawerTab(3)
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: shotTxt.implicitWidth + Math.round(16 * root.px)
                Text {
                    id: shotTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripScreenshot ?? ""
                    color: ThemeInk.text
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.session.screenshot()
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: Math.round(7 * root.px)
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.hairline
                implicitWidth: fsTxt.implicitWidth + Math.round(16 * root.px)
                Text {
                    id: fsTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripFullscreen ?? ""
                    color: ThemeInk.text
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }
                TapHandler {
                    onTapped: root.fullScreenRequested()
                }
            }

            Rectangle {
                Layout.preferredHeight: Math.round(28 * root.px)
                radius: height / 2
                color: ThemeInk.surfaceRaised
                border.width: 1
                border.color: ThemeInk.pillBorder
                implicitWidth: bpTxt.implicitWidth + Math.round(18 * root.px)
                Text {
                    id: bpTxt
                    anchors.centerIn: parent
                    text: root.chrome.stripBigPicture ?? ""
                    color: ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    font.weight: Kit.Tokens.weightTitle
                }
                TapHandler {
                    onTapped: root.bigPictureRequested()
                }
            }
        }
    }

    // Collapsible right-hand Studio Drawer over the right edge of the game.
    Rectangle {
        id: drawer
        objectName: "desktopPlayingDrawer"
        visible: root.model.drawerOpen
        anchors.top: topStrip.bottom
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Math.round(14 * root.px)
        width: Math.round(356 * root.px)
        radius: Math.round(14 * root.px)
        color: ThemeInk.glassStrong
        border.width: 1
        border.color: ThemeInk.pillBorder

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: Math.round(16 * root.px)
            spacing: Math.round(12 * root.px)

            // Drawer tabs header.
            RowLayout {
                Layout.fillWidth: true
                spacing: Math.round(6 * root.px)

                Repeater {
                    model: root.chrome.drawerTabs ?? []
                    delegate: Rectangle {
                        id: dTab
                        required property string modelData
                        required property int index
                        readonly property bool active: root.model.drawerTab === dTab.index
                        Layout.fillWidth: true
                        height: Math.round(28 * root.px)
                        radius: height / 2
                        color: dTab.active ? Kit.Tokens.textStrong : ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: dTab.active ? Kit.Tokens.textStrong : ThemeInk.hairline

                        Text {
                            anchors.centerIn: parent
                            text: dTab.modelData
                            color: dTab.active ? Kit.Tokens.voidFill : ThemeInk.text
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        TapHandler {
                            onTapped: root.model.drawerTab = dTab.index
                        }
                    }
                }

                Rectangle {
                    Layout.preferredWidth: Math.round(26 * root.px)
                    Layout.preferredHeight: Layout.preferredWidth
                    radius: width / 2
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.hairline

                    Item {
                        anchors.centerIn: parent
                        width: Math.round(12 * root.px)
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
                                strokeWidth: 2
                                PathSvg { path: root.chrome.icons?.close ?? "" }
                            }
                        }
                    }

                    TapHandler {
                        onTapped: root.model.drawerOpen = false
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 1
                color: ThemeInk.hairline
            }

            // Tab 0: Shaders & Speed.
            ColumnLayout {
                visible: root.model.drawerTab === 0
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Math.round(12 * root.px)

                Text {
                    text: root.chrome.shadersHeading ?? ""
                    color: ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(14, Math.round(16 * root.px))
                    font.weight: Kit.Tokens.weightTitle
                }

                Text {
                    text: root.chrome.speedLabel ?? ""
                    color: ThemeInk.muted
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(6 * root.px)

                    Repeater {
                        model: root.chrome.speeds ?? []
                        delegate: Rectangle {
                            id: spdBtn
                            required property var modelData
                            readonly property bool active: root.session.speed === (spdBtn.modelData.percent ?? 100)
                            Layout.fillWidth: true
                            height: Math.round(30 * root.px)
                            radius: Math.round(7 * root.px)
                            color: spdBtn.active ? Kit.Tokens.accent : ThemeInk.surfaceRaised
                            border.width: 1
                            border.color: spdBtn.active ? Kit.Tokens.accent : ThemeInk.hairline

                            Text {
                                anchors.centerIn: parent
                                text: spdBtn.modelData.label ?? ""
                                color: spdBtn.active ? Kit.Tokens.overAccent : ThemeInk.textStrong
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            TapHandler {
                                onTapped: root.session.chooseSpeed(spdBtn.modelData.percent ?? 100)
                            }
                        }
                    }
                }

                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: root.chrome.noDisplayRowsLabel ?? ""
                    color: ThemeInk.dim
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                Rectangle {
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.pillBorder
                    implicitWidth: openShadersBtnText.implicitWidth + Math.round(24 * root.px)

                    Text {
                        id: openShadersBtnText
                        anchors.centerIn: parent
                        text: root.chrome.stripShaders ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Kit.Tokens.weightTitle
                    }

                    TapHandler {
                        onTapped: root.displaySettingsRequested()
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }

            // Tab 1: Save States & Rewind.
            ColumnLayout {
                visible: root.model.drawerTab === 1
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Math.round(10 * root.px)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(34 * root.px)
                        radius: Math.round(8 * root.px)
                        color: Kit.Tokens.accent
                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.quickSaveLabel ?? ""
                            color: Kit.Tokens.overAccent
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        TapHandler {
                            onTapped: root.session.saveState(root.session.slot)
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(34 * root.px)
                        radius: Math.round(8 * root.px)
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.pillBorder
                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.quickLoadLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        TapHandler {
                            onTapped: root.session.loadState(root.session.slot)
                        }
                    }
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: root.playing.slots ?? []

                    delegate: Rectangle {
                        id: slotRow
                        required property var modelData
                        readonly property int slotNum: slotRow.modelData.slot ?? 1
                        readonly property bool active: root.session.slot === slotRow.slotNum
                        width: ListView.view.width
                        height: Math.round(48 * root.px)
                        radius: Math.round(8 * root.px)
                        color: slotRow.active ? ThemeInk.glowWeak : ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: slotRow.active ? ThemeInk.corona : ThemeInk.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)
                            spacing: Math.round(8 * root.px)

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 1
                                RowLayout {
                                    spacing: Math.round(4 * root.px)
                                    Text {
                                        text: root.chrome.slotPrefix ?? ""
                                        color: ThemeInk.textStrong
                                        font.family: Kit.Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Kit.Tokens.weightTitle
                                    }
                                    Text {
                                        text: String(slotRow.slotNum)
                                        color: ThemeInk.textStrong
                                        font.family: Kit.Tokens.faceCode
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Kit.Tokens.weightTitle
                                    }
                                }
                                Text {
                                    text: (slotRow.modelData.exists ?? false) ? (slotRow.modelData.when ?? "") : (root.chrome.emptySlotLabel ?? "")
                                    color: ThemeInk.dim
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            Rectangle {
                                visible: slotRow.modelData.exists ?? false
                                Layout.preferredHeight: Math.round(26 * root.px)
                                radius: height / 2
                                color: ThemeInk.glass
                                border.width: 1
                                border.color: ThemeInk.hairline
                                implicitWidth: loadSlotTxt.implicitWidth + Math.round(14 * root.px)
                                Text {
                                    id: loadSlotTxt
                                    anchors.centerIn: parent
                                    text: root.chrome.quickLoadLabel ?? ""
                                    color: ThemeInk.textStrong
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                                TapHandler {
                                    onTapped: root.session.loadState(slotRow.slotNum)
                                }
                            }
                        }

                        TapHandler {
                            onTapped: root.session.chooseSlot(slotRow.slotNum)
                        }
                    }
                }
            }

            // Tab 2: Cheats.
            ColumnLayout {
                visible: root.model.drawerTab === 2
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Math.round(10 * root.px)

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(38 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.hairline

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(12 * root.px)
                        anchors.rightMargin: Math.round(12 * root.px)

                        Text {
                            Layout.fillWidth: true
                            text: root.chrome.masterCheatsLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(12, Math.round(13 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        Rectangle {
                            Layout.preferredWidth: Math.round(38 * root.px)
                            Layout.preferredHeight: Math.round(22 * root.px)
                            radius: height / 2
                            color: root.session.masterCheats ? Kit.Tokens.accent : ThemeInk.track
                            Rectangle {
                                x: root.session.masterCheats ? parent.width - width - 3 : 3
                                anchors.verticalCenter: parent.verticalCenter
                                width: Math.round(16 * root.px)
                                height: width
                                radius: width / 2
                                color: Kit.Tokens.overAccent
                            }
                        }
                    }

                    TapHandler {
                        onTapped: root.session.setMasterCheats(!root.session.masterCheats)
                    }
                }

                Text {
                    visible: (root.playing.cheats ?? []).length === 0
                    text: root.chrome.noCheatsLabel ?? ""
                    color: ThemeInk.dim
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: root.playing.cheats ?? []

                    delegate: Rectangle {
                        id: cheatRow
                        required property var modelData
                        readonly property bool on: cheatRow.modelData.on ?? false
                        width: ListView.view.width
                        height: Math.round(38 * root.px)
                        radius: Math.round(7 * root.px)
                        color: cheatRow.on ? ThemeInk.glowWeak : ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: cheatRow.on ? ThemeInk.coronaSoft : ThemeInk.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)

                            Text {
                                Layout.fillWidth: true
                                text: cheatRow.modelData.description ?? ""
                                elide: Text.ElideRight
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Rectangle {
                                Layout.preferredWidth: Math.round(10 * root.px)
                                Layout.preferredHeight: Layout.preferredWidth
                                radius: width / 2
                                color: cheatRow.on ? ThemeInk.accent : ThemeInk.muted
                            }
                        }

                        TapHandler {
                            onTapped: root.session.switchCheat(cheatRow.modelData.index ?? 0, !cheatRow.on)
                        }
                    }
                }
            }

            // Tab 3: Options & Session controls.
            ColumnLayout {
                visible: root.model.drawerTab === 3
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: Math.round(10 * root.px)

                ListView {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: root.playing.consoleOptions ?? []

                    delegate: Rectangle {
                        id: optRow
                        required property var modelData
                        readonly property var choices: optRow.modelData.options ?? []
                        readonly property string curVal: optRow.modelData.value ?? ""
                        readonly property int nextIdx: choices.length > 0 ? ((choices.indexOf(curVal) + 1) % choices.length) : 0
                        width: ListView.view.width
                        height: Math.round(40 * root.px)
                        radius: Math.round(7 * root.px)
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)

                            Text {
                                Layout.fillWidth: true
                                text: optRow.modelData.label ?? ""
                                elide: Text.ElideRight
                                color: ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                text: optRow.curVal
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        TapHandler {
                            onTapped: if (optRow.choices.length > 0) root.session.setConsoleSetting(optRow.modelData.key ?? "", optRow.choices[optRow.nextIdx])
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
                    Text {
                        anchors.centerIn: parent
                        text: root.chrome.swapPlayersLabel ?? ""
                        color: ThemeInk.textStrong
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }
                    TapHandler {
                        onTapped: root.session.swapPlayers()
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(34 * root.px)
                        radius: Math.round(8 * root.px)
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.hairline
                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.restartLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        TapHandler {
                            onTapped: root.session.restart()
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(34 * root.px)
                        radius: Math.round(8 * root.px)
                        color: ThemeInk.errorFill
                        border.width: 1
                        border.color: ThemeInk.error
                        Text {
                            anchors.centerIn: parent
                            text: root.chrome.quitGameLabel ?? ""
                            color: ThemeInk.error
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        TapHandler {
                            onTapped: root.session.close()
                        }
                    }
                }
            }
        }
    }
}
