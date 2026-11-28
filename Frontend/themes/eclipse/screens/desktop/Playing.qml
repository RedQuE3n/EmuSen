pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import EmuSen.Theme
import "../../parts"
import "../../parts/desktop"

// Eclipse Schema 2 Desktop Playing HUD: pointer-clickable top strip and slide-over utility drawer over pure #000000 viewport.
ThemeScreen {
    id: screen
    objectName: "eclipseDesktopPlayingScreen"
    role: ThemeScreen.DesktopPlaying

    Item {
        id: root
        objectName: "eclipseDesktopPlaying"
        anchors.fill: parent
        focus: Workspace.drawerOpen
        readonly property real px: Math.max(0.84, root.height / 1000)

        DesktopInk {
            id: ink
        }

        Keys.onEscapePressed: (event) => {
            if (Workspace.drawerOpen) {
                Actions.workspaceSetDrawer(false)
                event.accepted = true
            }
        }

        // Top control strip remains clickable while the drawer is closed.
        Rectangle {
            id: topStrip
            objectName: "eclipsePlayingStrip"
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: Math.round(44 * root.px)
            color: ink.glassStrong
            border.width: 1
            border.color: ink.hairline

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Math.round(12 * root.px)
                anchors.rightMargin: Math.round(12 * root.px)
                spacing: Math.round(6 * root.px)

                Rectangle {
                    objectName: "eclipsePlayingBackLibrary"
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopNavTabs[0]
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: height / 2
                    color: libHover.hovered ? ink.surfaceRaised : ink.glass
                    border.width: 1
                    border.color: ink.pillBorder
                    implicitWidth: backRow.implicitWidth + Math.round(16 * root.px)

                    RowLayout {
                        id: backRow
                        anchors.centerIn: parent
                        spacing: Math.round(4 * root.px)

                        DesktopGlyph {
                            Layout.preferredWidth: Math.round(12 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            kind: 3
                            color: ink.textStrong
                        }

                        Text {
                            text: Words.desktopNavTabs[0]
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Tokens.weightTitle
                        }
                    }

                    HoverHandler {
                        id: libHover
                    }

                    TapHandler {
                        onTapped: {
                            Actions.workspaceSetDrawer(false)
                            Actions.workspaceView(0)
                            Actions.sessionClose()
                        }
                    }
                }

                Text {
                    text: Session.title
                    elide: Text.ElideRight
                    Layout.maximumWidth: Math.round(200 * root.px)
                    color: ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(12, Math.round(14 * root.px))
                    font.weight: Tokens.weightTitle
                }

                Rectangle {
                    visible: Session.core.length > 0
                    Layout.preferredHeight: Math.round(24 * root.px)
                    radius: height / 2
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    implicitWidth: chipRow.implicitWidth + Math.round(14 * root.px)

                    RowLayout {
                        id: chipRow
                        anchors.centerIn: parent
                        spacing: Math.round(6 * root.px)

                        PlanetSphere {
                            unit: root.px
                            side: 12
                            glowBlur: 5
                            glowSpread: 1
                            planet: Session.planet
                        }

                        Text {
                            text: Session.core
                            color: ink.text
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }

                        Text {
                            visible: Session.fpsWords.length > 0
                            text: Session.fpsWords
                            color: ink.success
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }
                }

                Item {
                    Layout.fillWidth: true
                }

                // Pause / Resume toggle.
                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Session.paused ? Words.desktopResumeLabel : Words.desktopStripPause
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    implicitWidth: pauseTxt.implicitWidth + Math.round(16 * root.px)

                    Text {
                        id: pauseTxt
                        anchors.centerIn: parent
                        text: Session.paused ? Words.desktopResumeLabel : Words.desktopStripPause
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    TapHandler {
                        onTapped: if (Session.paused) Actions.sessionResume(); else Actions.sessionPause()
                    }
                }

                // Rewind action.
                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopStripRewind
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    opacity: Session.hasRewind ? 1.0 : 0.55
                    implicitWidth: rewindTxt.implicitWidth + Math.round(16 * root.px)

                    Text {
                        id: rewindTxt
                        anchors.centerIn: parent
                        text: Words.desktopStripRewind
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }

                    TapHandler {
                        onTapped: if (Session.hasRewind) Actions.sessionRewindTo(Session.latestRewindFrame)
                    }
                }

                // Drawer tab pills: Shaders (0), States (1), Cheats (2), Options (3).
                Repeater {
                    model: Words.desktopDrawerTabs
                    delegate: Rectangle {
                        id: stripTabBtn
                        required property int index
                        required property string modelData
                        readonly property bool active: Workspace.drawerOpen && Workspace.drawerTab === stripTabBtn.index
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: stripTabBtn.modelData
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: Math.round(7 * root.px)
                        color: stripTabBtn.active ? ink.textStrong : ink.surfaceRaised
                        border.width: 1
                        border.color: stripTabBtn.active ? ink.textStrong : ink.hairline
                        implicitWidth: stripTabTxt.implicitWidth + Math.round(16 * root.px)

                        Text {
                            id: stripTabTxt
                            anchors.centerIn: parent
                            text: stripTabBtn.modelData
                            color: stripTabBtn.active ? ink.surface : ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: stripTabBtn.active ? Tokens.weightTitle : Tokens.weightSentence
                        }

                        TapHandler {
                            onTapped: if (Workspace.drawerOpen && Workspace.drawerTab === stripTabBtn.index) Actions.workspaceSetDrawer(false); else { Actions.workspaceSetDrawerTab(stripTabBtn.index); Actions.workspaceSetDrawer(true) }
                        }
                    }
                }

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopStripShot
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    implicitWidth: shotTxt.implicitWidth + Math.round(14 * root.px)

                    Text {
                        id: shotTxt
                        anchors.centerIn: parent
                        text: Words.desktopStripShot
                        color: ink.text
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }

                    TapHandler {
                        onTapped: Actions.sessionScreenshot()
                    }
                }

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopStripFullscreen
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    implicitWidth: fsTxt.implicitWidth + Math.round(14 * root.px)

                    Text {
                        id: fsTxt
                        anchors.centerIn: parent
                        text: Words.desktopStripFullscreen
                        color: ink.text
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }

                    TapHandler {
                        onTapped: Actions.toggleFullScreen()
                    }
                }

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopBigPictureLabel
                    Layout.preferredHeight: Math.round(28 * root.px)
                    radius: height / 2
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.pillBorder
                    implicitWidth: bpTxt.implicitWidth + Math.round(16 * root.px)

                    Text {
                        id: bpTxt
                        anchors.centerIn: parent
                        text: Words.desktopBigPictureLabel
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    TapHandler {
                        onTapped: Actions.setBigScreen(true)
                    }
                }
            }
        }

        // Slide-over utility drawer gated exclusively by Workspace.drawerOpen.
        DesktopPlayingDrawer {
            visible: Workspace.drawerOpen
            px: root.px
            pixelScale: root.px
            anchors.top: topStrip.bottom
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Math.round(14 * root.px)
        }
    }
}
