pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import EmuSen.Theme

// Translucent slide-over live utility drawer for Eclipse Desktop Playing HUD.
Rectangle {
    id: root
    required property real px
    required property real pixelScale

    objectName: "eclipsePlayingDrawer"
    width: Math.round(360 * root.px)
    radius: Math.round(14 * root.px)
    color: ink.glassStrong
    border.width: 1
    border.color: ink.pillBorder

    DesktopInk {
        id: ink
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Math.round(16 * root.px)
        spacing: Math.round(12 * root.px)

        // Drawer tabs and close button.
        RowLayout {
            Layout.fillWidth: true
            spacing: Math.round(6 * root.px)

            Repeater {
                model: Words.desktopDrawerTabs
                delegate: Rectangle {
                    id: dTab
                    required property string modelData
                    required property int index
                    readonly property bool active: Workspace.drawerTab === dTab.index
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: dTab.modelData
                    Layout.fillWidth: true
                    height: Math.round(28 * root.px)
                    radius: height / 2
                    color: dTab.active ? ink.textStrong : ink.surfaceRaised
                    border.width: dTab.activeFocus ? 2 : 1
                    border.color: dTab.activeFocus ? ink.corona : (dTab.active ? ink.textStrong : ink.hairline)

                    Text {
                        anchors.centerIn: parent
                        text: dTab.modelData
                        color: dTab.active ? ink.surface : ink.text
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            event.accepted = true
                            Actions.workspaceSetDrawerTab(dTab.index)
                        }
                    }

                    TapHandler {
                        onTapped: Actions.workspaceSetDrawerTab(dTab.index)
                    }
                }
            }

            Rectangle {
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopClearLabel
                Layout.preferredWidth: Math.round(26 * root.px)
                Layout.preferredHeight: Layout.preferredWidth
                radius: width / 2
                color: ink.surfaceRaised
                border.width: activeFocus ? 2 : 1
                border.color: activeFocus ? ink.corona : ink.hairline

                DesktopGlyph {
                    anchors.centerIn: parent
                    width: Math.round(11 * root.px)
                    height: width
                    kind: 5
                    color: ink.textStrong
                }

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        event.accepted = true
                        Actions.workspaceSetDrawer(false)
                    }
                }

                TapHandler {
                    onTapped: Actions.workspaceSetDrawer(false)
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: ink.hairline
        }

        // Tab 0: Display & Shaders + Speed (DrawerTab: 0 Shaders, 1 States, 2 Cheats, 3 Controls).
        ColumnLayout {
            visible: Workspace.drawerTab === 0
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Math.round(12 * root.px)

            Text {
                text: Words.desktopShadersHeading
                color: ink.textStrong
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(14, Math.round(16 * root.px))
                font.weight: Tokens.weightTitle
            }

            Text {
                text: Words.desktopSpeedLabel
                color: ink.muted
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(11, Math.round(12 * root.px))
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Math.round(6 * root.px)

                Repeater {
                    model: 4
                    delegate: Rectangle {
                        id: spdBtn
                        required property int index
                        readonly property int percentVal: spdBtn.index === 0 ? 25 : spdBtn.index === 1 ? 100 : spdBtn.index === 2 ? 300 : 0
                        readonly property bool active: Session.speed === spdBtn.percentVal
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: spdBtn.index === 3 ? Words.desktopSpeedMaxLabel : Words.percent(spdBtn.percentVal)
                        Layout.fillWidth: true
                        height: Math.round(30 * root.px)
                        radius: Math.round(7 * root.px)
                        color: spdBtn.active ? ink.accent : ink.surfaceRaised
                        border.width: spdBtn.activeFocus ? 2 : 1
                        border.color: spdBtn.activeFocus ? ink.corona : (spdBtn.active ? ink.accent : ink.hairline)

                        Text {
                            anchors.centerIn: parent
                            text: spdBtn.index === 3 ? Words.desktopSpeedMaxLabel : Words.percent(spdBtn.percentVal)
                            color: spdBtn.active ? ink.overAccent : ink.textStrong
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Tokens.weightTitle
                        }

                        Keys.onPressed: (event) => {
                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                event.accepted = true
                                Actions.sessionChooseSpeed(spdBtn.percentVal)
                            }
                        }

                        TapHandler {
                            onTapped: Actions.sessionChooseSpeed(spdBtn.percentVal)
                        }
                    }
                }
            }

            Text {
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: Words.desktopNoDisplayRowsLabel
                color: ink.dim
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(11, Math.round(12 * root.px))
            }

            Rectangle {
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopStripShaders
                Layout.preferredHeight: Math.round(34 * root.px)
                radius: Math.round(8 * root.px)
                color: ink.surfaceRaised
                border.width: activeFocus ? 2 : 1
                border.color: activeFocus ? ink.corona : ink.pillBorder
                implicitWidth: openShadersTxt.implicitWidth + Math.round(24 * root.px)

                Text {
                    id: openShadersTxt
                    anchors.centerIn: parent
                    text: Words.desktopStripShaders
                    color: ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    font.weight: Tokens.weightTitle
                }

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        event.accepted = true
                        Actions.workspaceShaders()
                    }
                }

                TapHandler {
                    onTapped: Actions.workspaceShaders()
                }
            }

            Item {
                Layout.fillHeight: true
            }
        }

        // Tab 1: Save States.
        ColumnLayout {
            visible: Workspace.drawerTab === 1
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Math.round(10 * root.px)

            RowLayout {
                Layout.fillWidth: true
                spacing: Math.round(8 * root.px)

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopQuickSaveLabel
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ink.accent
                    border.width: activeFocus ? 2 : 0
                    border.color: ink.corona

                    Text {
                        anchors.centerIn: parent
                        text: Words.desktopQuickSaveLabel
                        color: ink.overAccent
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            event.accepted = true
                            Actions.sessionSaveState(Session.slot)
                        }
                    }

                    TapHandler {
                        onTapped: Actions.sessionSaveState(Session.slot)
                    }
                }

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopQuickLoadLabel
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ink.surfaceRaised
                    border.width: activeFocus ? 2 : 1
                    border.color: activeFocus ? ink.corona : ink.pillBorder

                    Text {
                        anchors.centerIn: parent
                        text: Words.desktopQuickLoadLabel
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            event.accepted = true
                            Actions.sessionLoadState(Session.slot)
                        }
                    }

                    TapHandler {
                        onTapped: Actions.sessionLoadState(Session.slot)
                    }
                }
            }

            EdgeFade {
                Layout.fillWidth: true
                Layout.fillHeight: true
                fadeTop: !slotsList.atYBeginning
                fadeBottom: !slotsList.atYEnd
                length: Math.round(12 * root.px)

                ListView {
                    id: slotsList
                    anchors.fill: parent
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: Session.slots
                    readonly property bool fadeTop: !atYBeginning
                    readonly property bool fadeBottom: !atYEnd

                    delegate: Rectangle {
                        id: slotRow
                        required property int slot
                        required property bool exists
                        required property string when
                        required property string picture
                        required property bool active
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopSlotPrefix
                        width: slotsList.width
                        height: Math.round(48 * root.px)
                        radius: Math.round(8 * root.px)
                        color: slotRow.active ? ink.glowWeak : ink.surfaceRaised
                        border.width: slotRow.activeFocus ? 2 : 1
                        border.color: (slotRow.active || slotRow.activeFocus) ? ink.corona : ink.hairline

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
                                        text: Words.desktopSlotPrefix
                                        color: ink.textStrong
                                        font.family: Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }
                                    Text {
                                        text: String(slotRow.slot)
                                        color: ink.textStrong
                                        font.family: Tokens.faceCode
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Tokens.weightTitle
                                    }
                                }
                                Text {
                                    text: slotRow.exists ? slotRow.when : Words.desktopEmptySlotLabel
                                    color: ink.dim
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            Rectangle {
                                visible: slotRow.exists
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopQuickLoadLabel
                                Layout.preferredHeight: Math.round(26 * root.px)
                                radius: height / 2
                                color: ink.glass
                                border.width: activeFocus ? 2 : 1
                                border.color: activeFocus ? ink.corona : ink.hairline
                                implicitWidth: loadSlotTxt.implicitWidth + Math.round(14 * root.px)

                                Text {
                                    id: loadSlotTxt
                                    anchors.centerIn: parent
                                    text: Words.desktopQuickLoadLabel
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        event.accepted = true
                                        Actions.sessionLoadState(slotRow.slot)
                                    }
                                }

                                TapHandler {
                                    onTapped: Actions.sessionLoadState(slotRow.slot)
                                }
                            }
                        }

                        Keys.onPressed: (event) => {
                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                event.accepted = true
                                Actions.sessionChooseSlot(slotRow.slot)
                            }
                        }

                        TapHandler {
                            onTapped: Actions.sessionChooseSlot(slotRow.slot)
                        }
                    }
                }
            }
        }

        // Tab 2: Cheats.
        ColumnLayout {
            visible: Workspace.drawerTab === 2
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Math.round(10 * root.px)

            Rectangle {
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopMasterCheatsLabel
                Layout.fillWidth: true
                Layout.preferredHeight: Math.round(38 * root.px)
                radius: Math.round(8 * root.px)
                color: ink.surfaceRaised
                border.width: activeFocus ? 2 : 1
                border.color: activeFocus ? ink.corona : ink.hairline

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Math.round(12 * root.px)
                    anchors.rightMargin: Math.round(12 * root.px)

                    Text {
                        Layout.fillWidth: true
                        text: Words.desktopMasterCheatsLabel
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(12, Math.round(13 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    Rectangle {
                        Layout.preferredWidth: Math.round(38 * root.px)
                        Layout.preferredHeight: Math.round(22 * root.px)
                        radius: height / 2
                        color: Session.masterCheats ? ink.accent : ink.track

                        Rectangle {
                            x: Session.masterCheats ? parent.width - width - 3 : 3
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.round(16 * root.px)
                            height: width
                            radius: width / 2
                            color: ink.overAccent
                        }
                    }
                }

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        event.accepted = true
                        Actions.sessionSetMasterCheats(!Session.masterCheats)
                    }
                }

                TapHandler {
                    onTapped: Actions.sessionSetMasterCheats(!Session.masterCheats)
                }
            }

            Text {
                visible: cheatsList.count === 0
                text: Words.desktopNoCheatsLabel
                color: ink.dim
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(11, Math.round(12 * root.px))
            }

            EdgeFade {
                Layout.fillWidth: true
                Layout.fillHeight: true
                fadeTop: !cheatsList.atYBeginning
                fadeBottom: !cheatsList.atYEnd
                length: Math.round(12 * root.px)

                ListView {
                    id: cheatsList
                    anchors.fill: parent
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: Session.cheats
                    readonly property bool fadeTop: !atYBeginning
                    readonly property bool fadeBottom: !atYEnd

                    delegate: Rectangle {
                        id: cheatRow
                        required property int index
                        required property string description
                        required property bool on
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: cheatRow.description
                        width: cheatsList.width
                        height: Math.round(38 * root.px)
                        radius: Math.round(7 * root.px)
                        color: cheatRow.on ? ink.glowWeak : ink.surfaceRaised
                        border.width: cheatRow.activeFocus ? 2 : 1
                        border.color: cheatRow.activeFocus ? ink.corona : (cheatRow.on ? ink.coronaSoft : ink.hairline)

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)

                            Text {
                                Layout.fillWidth: true
                                text: cheatRow.description
                                elide: Text.ElideRight
                                color: ink.textStrong
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Rectangle {
                                Layout.preferredWidth: Math.round(10 * root.px)
                                Layout.preferredHeight: Layout.preferredWidth
                                radius: width / 2
                                color: cheatRow.on ? ink.accent : ink.muted
                            }
                        }

                        Keys.onPressed: (event) => {
                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                event.accepted = true
                                Actions.sessionSwitchCheat(cheatRow.index)
                            }
                        }

                        TapHandler {
                            onTapped: Actions.sessionSwitchCheat(cheatRow.index)
                        }
                    }
                }
            }
        }

        // Tab 3: Options & Session controls.
        ColumnLayout {
            visible: Workspace.drawerTab === 3
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Math.round(10 * root.px)

            EdgeFade {
                Layout.fillWidth: true
                Layout.fillHeight: true
                fadeTop: !optsList.atYBeginning
                fadeBottom: !optsList.atYEnd
                length: Math.round(12 * root.px)

                ListView {
                    id: optsList
                    anchors.fill: parent
                    clip: true
                    spacing: Math.round(6 * root.px)
                    model: Session.consoleOptions
                    readonly property bool fadeTop: !atYBeginning
                    readonly property bool fadeBottom: !atYEnd

                    delegate: Rectangle {
                        id: optRow
                        required property string key
                        required property string label
                        required property string value
                        required property string displayValue
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: optRow.label
                        width: optsList.width
                        height: Math.round(40 * root.px)
                        radius: Math.round(7 * root.px)
                        color: ink.surfaceRaised
                        border.width: optRow.activeFocus ? 2 : 1
                        border.color: optRow.activeFocus ? ink.corona : ink.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(10 * root.px)
                            anchors.rightMargin: Math.round(10 * root.px)

                            Text {
                                Layout.fillWidth: true
                                text: optRow.label
                                elide: Text.ElideRight
                                color: ink.text
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Text {
                                text: optRow.displayValue.length > 0 ? optRow.displayValue : optRow.value
                                color: ink.textStrong
                                font.family: Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        Keys.onPressed: (event) => {
                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                event.accepted = true
                                Actions.sessionStepConsoleOption(optRow.key, 1)
                            }
                        }

                        TapHandler {
                            onTapped: Actions.sessionStepConsoleOption(optRow.key, 1)
                        }
                    }
                }
            }

            Rectangle {
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopSwapPlayersLabel
                Layout.fillWidth: true
                Layout.preferredHeight: Math.round(34 * root.px)
                radius: Math.round(8 * root.px)
                color: ink.surfaceRaised
                border.width: activeFocus ? 2 : 1
                border.color: activeFocus ? ink.corona : ink.hairline

                Text {
                    anchors.centerIn: parent
                    text: Words.desktopSwapPlayersLabel
                    color: ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        event.accepted = true
                        Actions.sessionSwapPlayers()
                    }
                }

                TapHandler {
                    onTapped: Actions.sessionSwapPlayers()
                }
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: Math.round(8 * root.px)

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopRestartLabel
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ink.surfaceRaised
                    border.width: activeFocus ? 2 : 1
                    border.color: activeFocus ? ink.corona : ink.hairline

                    Text {
                        anchors.centerIn: parent
                        text: Words.desktopRestartLabel
                        color: ink.textStrong
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            event.accepted = true
                            Actions.sessionRestart()
                        }
                    }

                    TapHandler {
                        onTapped: Actions.sessionRestart()
                    }
                }

                Rectangle {
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Words.desktopQuitGameLabel
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(34 * root.px)
                    radius: Math.round(8 * root.px)
                    color: ink.errorFill
                    border.width: activeFocus ? 2 : 1
                    border.color: activeFocus ? ink.corona : ink.error

                    Text {
                        anchors.centerIn: parent
                        text: Words.desktopQuitGameLabel
                        color: ink.error
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        font.weight: Tokens.weightTitle
                    }

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            event.accepted = true
                            Actions.sessionClose()
                        }
                    }

                    TapHandler {
                        onTapped: Actions.sessionClose()
                    }
                }
            }
        }
    }
}
