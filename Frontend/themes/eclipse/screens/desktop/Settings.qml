pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme
import "../../parts"
import "../../parts/desktop"

// Eclipse Schema 2 Desktop Settings & Tools screen: unified 6 Settings + 6 Tools rail, interactive rows, inline Themes, and live detail pane.
ThemeScreen {
    id: screen
    objectName: "eclipseDesktopSettingsScreen"
    role: ThemeScreen.DesktopSettings

    Item {
        id: root
        objectName: "eclipseDesktopSettings"
        anchors.fill: parent
        readonly property real px: Math.max(0.84, root.height / 1000)
        readonly property bool showingThemes: Settings.toolIndex === 5
        readonly property string filterNeedle: Settings.filter.trim().toLowerCase()

        DesktopInk {
            id: ink
        }

        DesktopBackdrop {
            anchors.fill: parent
            px: root.px
            pixelScale: root.px
            showLimb: true
        }

        ColumnLayout {
            anchors.fill: parent
            spacing: 0

            DesktopTopRibbon {
                px: root.px
                showInspectorToggle: true
                showViewSwitcher: false
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: 0

                // Left Settings & Tools navigation rail.
                Rectangle {
                    objectName: "eclipseDesktopSettingsRail"
                    visible: Workspace.sidebarOpen
                    Layout.fillHeight: true
                    Layout.preferredWidth: Math.round(248 * root.px)
                    color: ink.railSurface
                    border.width: 1
                    border.color: ink.hairline

                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: Math.round(14 * root.px)
                        spacing: Math.round(10 * root.px)

                        Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: ink.inputSurface
                            border.width: 1
                            border.color: filterField.activeFocus ? ink.corona : ink.hairline

                            TextField {
                                id: filterField
                                objectName: "eclipseDesktopSettingsFilterInput"
                                anchors.fill: parent
                                leftPadding: Math.round(10 * root.px)
                                rightPadding: Math.round(10 * root.px)
                                color: ink.textStrong
                                placeholderTextColor: ink.dim
                                placeholderText: Words.desktopSettingsFilterPlaceholder
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                text: Settings.filter
                                background: Item {}
                                signal syncModelText()
                                onSyncModelText: if (text !== Settings.filter) text = Settings.filter
                                onTextChanged: if (text !== Settings.filter) Actions.settingsSetFilter(text)
                                Connections {
                                    target: Settings
                                    function onShown() { filterField.syncModelText() }
                                }
                            }
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: Words.desktopSettingsHeading
                                color: ink.muted
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                font.weight: Tokens.weightWordmark
                                font.letterSpacing: 1.1
                            }
                            Text {
                                text: String(Settings.sections.length)
                                color: ink.dim
                                font.family: Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(3 * root.px)

                            Repeater {
                                model: Settings.sections
                                delegate: Rectangle {
                                    id: secBtn
                                    required property int index
                                    required property string modelData
                                    readonly property bool current: !root.showingThemes && Settings.section === secBtn.index
                                    readonly property string badgeWord: Settings.sectionBadges[secBtn.index] ?? ""
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: secBtn.modelData
                                    Layout.fillWidth: true
                                    height: Math.round(34 * root.px)
                                    radius: Math.round(8 * root.px)
                                    color: secBtn.current ? ink.textStrong : secHover.hovered ? ink.surfaceRaised : "transparent"
                                    border.width: secBtn.activeFocus ? 2 : (secBtn.current ? 1 : 0)
                                    border.color: secBtn.activeFocus ? ink.corona : ink.coronaSoft

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: Math.round(10 * root.px)
                                        anchors.rightMargin: Math.round(10 * root.px)
                                        spacing: Math.round(8 * root.px)

                                        Image {
                                            Layout.preferredWidth: Math.round(16 * root.px)
                                            Layout.preferredHeight: Layout.preferredWidth
                                            source: "../../assets/sections.svg"
                                            sourceSize: Qt.size(6 * 96, 2 * 96)
                                            sourceClipRect: Qt.rect((secBtn.index % 6) * 96, (secBtn.current ? 1 : 0) * 96, 96, 96)
                                            smooth: true
                                        }

                                        Text {
                                            Layout.fillWidth: true
                                            text: secBtn.modelData
                                            color: secBtn.current ? ink.surface : ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(13 * root.px))
                                            font.weight: secBtn.current ? Tokens.weightTitle : Tokens.weightSentence
                                        }

                                        Text {
                                            visible: secBtn.badgeWord.length > 0
                                            text: secBtn.badgeWord
                                            color: secBtn.current ? ink.surface : ink.dim
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(10, Math.round(10.5 * root.px))
                                        }
                                    }

                                    HoverHandler {
                                        id: secHover
                                    }

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            event.accepted = true
                                            Actions.settingsSection(secBtn.index)
                                        }
                                    }

                                    TapHandler {
                                        onTapped: Actions.settingsSection(secBtn.index)
                                    }
                                }
                            }
                        }

                        Rectangle {
                            visible: Tools.count > 0
                            Layout.fillWidth: true
                            Layout.preferredHeight: 1
                            color: ink.hairline
                        }

                        RowLayout {
                            visible: Tools.count > 0
                            Layout.fillWidth: true
                            Text {
                                Layout.fillWidth: true
                                text: Words.desktopToolsHeading
                                color: ink.muted
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                font.weight: Tokens.weightWordmark
                                font.letterSpacing: 1.1
                            }
                            Text {
                                text: String(Tools.count)
                                color: ink.dim
                                font.family: Tokens.faceCode
                                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                            }
                        }

                        ColumnLayout {
                            visible: Tools.count > 0
                            Layout.fillWidth: true
                            spacing: Math.round(3 * root.px)

                            Repeater {
                                model: Tools.entries
                                delegate: Rectangle {
                                    id: toolBtn
                                    required property int index
                                    required property string key
                                    required property string name
                                    required property string badge
                                    required property string icon
                                    required property bool attention
                                    readonly property bool isThemes: toolBtn.key === Workspace.themesDoorKey
                                    readonly property bool current: root.showingThemes && toolBtn.isThemes
                                    activeFocusOnTab: true
                                    Accessible.role: Accessible.Button
                                    Accessible.name: toolBtn.name
                                    Layout.fillWidth: true
                                    height: Math.round(32 * root.px)
                                    radius: Math.round(8 * root.px)
                                    color: toolBtn.current ? ink.textStrong : toolHover.hovered ? ink.surfaceRaised : "transparent"
                                    border.width: toolBtn.activeFocus ? 2 : 0
                                    border.color: ink.corona

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: Math.round(10 * root.px)
                                        anchors.rightMargin: Math.round(10 * root.px)
                                        spacing: Math.round(8 * root.px)

                                        Item {
                                            Layout.preferredWidth: Math.round(15 * root.px)
                                            Layout.preferredHeight: Layout.preferredWidth
                                            Shape {
                                                width: 24
                                                height: 24
                                                scale: parent.width / 24
                                                transformOrigin: Item.TopLeft
                                                preferredRendererType: Shape.CurveRenderer
                                                ShapePath {
                                                    fillColor: "transparent"
                                                    strokeColor: toolBtn.current ? ink.surface : ink.muted
                                                    strokeWidth: 1.7
                                                    capStyle: ShapePath.RoundCap
                                                    joinStyle: ShapePath.RoundJoin
                                                    PathSvg { path: toolBtn.icon }
                                                }
                                            }
                                        }

                                        Text {
                                            Layout.fillWidth: true
                                            text: toolBtn.name
                                            color: toolBtn.current ? ink.surface : ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                            font.weight: toolBtn.current ? Tokens.weightTitle : Tokens.weightSentence
                                        }

                                        Rectangle {
                                            visible: toolBtn.badge.length > 0
                                            Layout.preferredHeight: Math.round(18 * root.px)
                                            radius: height / 2
                                            color: toolBtn.attention ? ink.attentionFill : ink.surfaceRaised
                                            border.width: 1
                                            border.color: toolBtn.attention ? ink.attention : ink.hairline
                                            implicitWidth: tBadgeTxt.implicitWidth + Math.round(10 * root.px)

                                            Text {
                                                id: tBadgeTxt
                                                anchors.centerIn: parent
                                                text: toolBtn.badge
                                                color: toolBtn.attention ? ink.attention : ink.muted
                                                font.family: Tokens.faceCode
                                                font.pixelSize: Math.max(9, Math.round(10 * root.px))
                                            }
                                        }
                                    }

                                    HoverHandler {
                                        id: toolHover
                                    }

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            event.accepted = true
                                            if (toolBtn.isThemes) Actions.settingsOpenTool(5); else Actions.toolsOpen(toolBtn.key)
                                        }
                                    }

                                    TapHandler {
                                        onTapped: if (toolBtn.isThemes) Actions.settingsOpenTool(5); else Actions.toolsOpen(toolBtn.key)
                                    }
                                }
                            }
                        }

                        Item {
                            Layout.fillHeight: true
                        }
                    }
                }

                // Center stage: Settings rows or inline Themes Manager.
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true

                    // Settings rows pane.
                    ColumnLayout {
                        visible: !root.showingThemes
                        anchors.fill: parent
                        anchors.margins: Math.round(22 * root.px)
                        spacing: Math.round(14 * root.px)

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(10 * root.px)

                            Rectangle {
                                visible: Settings.canBack
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopBackLabel
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: ink.surfaceRaised
                                border.width: activeFocus ? 2 : 1
                                border.color: activeFocus ? ink.corona : ink.pillBorder
                                implicitWidth: backTxt.implicitWidth + Math.round(18 * root.px)

                                Text {
                                    id: backTxt
                                    anchors.centerIn: parent
                                    text: Words.desktopBackLabel
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        event.accepted = true
                                        Actions.settingsBack()
                                    }
                                }

                                TapHandler {
                                    onTapped: Actions.settingsBack()
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                Text {
                                    visible: Settings.trailLine.length > 0
                                    text: Settings.trailLine
                                    color: ink.rose
                                    font.family: Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }

                                Text {
                                    objectName: "eclipseDesktopSettingsHeading"
                                    text: Settings.heading
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(20, Math.round(Tokens.fontTitle * root.px))
                                    font.weight: Tokens.weightTitle
                                }
                            }

                            Rectangle {
                                activeFocusOnTab: true
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopModifiedOnlyLabel
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: Settings.modifiedOnly ? ink.accent : ink.glass
                                border.width: activeFocus ? 2 : 1
                                border.color: activeFocus ? ink.corona : (Settings.modifiedOnly ? ink.accent : ink.hairline)
                                implicitWidth: modTxt.implicitWidth + Math.round(20 * root.px)

                                Text {
                                    id: modTxt
                                    anchors.centerIn: parent
                                    text: Words.desktopModifiedOnlyLabel
                                    color: Settings.modifiedOnly ? ink.overAccent : ink.text
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        event.accepted = true
                                        Actions.settingsSetModifiedOnly(!Settings.modifiedOnly)
                                    }
                                }

                                TapHandler {
                                    onTapped: Actions.settingsSetModifiedOnly(!Settings.modifiedOnly)
                                }
                            }
                        }

                        EdgeFade {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            fadeTop: !settingsList.atYBeginning
                            fadeBottom: !settingsList.atYEnd
                            length: Math.round(14 * root.px)

                            ListView {
                                id: settingsList
                                objectName: "eclipseDesktopSettingsRows"
                                anchors.fill: parent
                                clip: true
                                spacing: 0
                                model: Settings.rows
                                readonly property bool fadeTop: !atYBeginning
                                readonly property bool fadeBottom: !atYEnd
                                ScrollBar.vertical: ScrollBar {
                                    policy: ScrollBar.AsNeeded
                                }

                                delegate: Item {
                                    id: sRow
                                    required property int index
                                    required property string key
                                    required property string label
                                    required property string about
                                    required property string value
                                    required property bool isSwitch
                                    required property bool switchOn
                                    required property bool isChoice
                                    required property bool isSlider
                                    required property bool isDoor
                                    required property bool isModified
                                    required property bool marked
                                    required property bool code
                                    readonly property bool matchesFilter: root.filterNeedle.length === 0 || (sRow.label + sRow.about + sRow.value).toLowerCase().indexOf(root.filterNeedle) >= 0
                                    readonly property bool shown: (!Settings.modifiedOnly || sRow.isModified) && sRow.matchesFilter
                                    readonly property bool focused: Settings.focused === sRow.index
                                    activeFocusOnTab: sRow.shown
                                    Accessible.role: Accessible.Button
                                    Accessible.name: sRow.label
                                    width: settingsList.width
                                    visible: sRow.shown
                                    height: sRow.shown ? Math.round(66 * root.px) : 0

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            Actions.settingsFocus(sRow.index)
                                            if (sRow.isSwitch || sRow.isDoor) Actions.settingsChoose(sRow.index)
                                            event.accepted = true
                                        }
                                    }

                                    Rectangle {
                                        anchors.left: parent.left
                                        anchors.right: parent.right
                                        anchors.top: parent.top
                                        height: Math.round(58 * root.px)
                                        radius: Math.round(10 * root.px)
                                        color: sRow.focused ? ink.glowWeak : sRowHover.hovered ? ink.cardHover : ink.surfaceRaised
                                        border.width: sRow.focused || sRow.activeFocus ? 1.5 : 1
                                        border.color: sRow.focused || sRow.activeFocus ? ink.corona : ink.hairline

                                        RowLayout {
                                            anchors.fill: parent
                                            anchors.leftMargin: Math.round(14 * root.px)
                                            anchors.rightMargin: Math.round(14 * root.px)
                                            spacing: Math.round(12 * root.px)

                                            ColumnLayout {
                                                Layout.fillWidth: true
                                                spacing: 2

                                                RowLayout {
                                                    spacing: Math.round(6 * root.px)

                                                    Text {
                                                        text: sRow.label
                                                        color: ink.textStrong
                                                        font.family: Tokens.faceInterface
                                                        font.pixelSize: Math.max(12, Math.round(Tokens.fontRow * root.px))
                                                        font.weight: Tokens.weightTitle
                                                    }

                                                    Rectangle {
                                                        visible: sRow.isModified || sRow.marked
                                                        Layout.preferredWidth: Math.round(6 * root.px)
                                                        Layout.preferredHeight: Layout.preferredWidth
                                                        radius: width / 2
                                                        color: ink.accent
                                                    }
                                                }

                                                Text {
                                                    visible: sRow.about.length > 0
                                                    Layout.fillWidth: true
                                                    elide: Text.ElideRight
                                                    text: sRow.about
                                                    color: ink.dim
                                                    font.family: Tokens.faceInterface
                                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                                }
                                            }

                                            // Boolean switch control.
                                            Rectangle {
                                                visible: sRow.isSwitch
                                                Layout.preferredWidth: Math.round(44 * root.px)
                                                Layout.preferredHeight: Math.round(24 * root.px)
                                                radius: height / 2
                                                color: sRow.switchOn ? ink.accent : ink.track

                                                Rectangle {
                                                    x: sRow.switchOn ? parent.width - width - 3 : 3
                                                    anchors.verticalCenter: parent.verticalCenter
                                                    width: Math.round(18 * root.px)
                                                    height: width
                                                    radius: width / 2
                                                    color: ink.overAccent
                                                }
                                            }

                                            // Value pill + steppers.
                                            RowLayout {
                                                visible: !sRow.isSwitch && sRow.value.length > 0
                                                spacing: Math.round(6 * root.px)

                                                Rectangle {
                                                    visible: sRow.isChoice || sRow.isSlider
                                                    activeFocusOnTab: sRow.shown && (sRow.isChoice || sRow.isSlider)
                                                    Accessible.role: Accessible.Button
                                                    Accessible.name: sRow.label
                                                    Layout.preferredWidth: Math.round(24 * root.px)
                                                    Layout.preferredHeight: Layout.preferredWidth
                                                    radius: width / 2
                                                    color: ink.glass
                                                    border.width: activeFocus ? 2 : 1
                                                    border.color: activeFocus ? ink.corona : ink.hairline

                                                    DesktopGlyph {
                                                        anchors.centerIn: parent
                                                        width: Math.round(11 * root.px)
                                                        height: width
                                                        kind: 3
                                                        color: ink.textStrong
                                                    }

                                                    Keys.onPressed: (event) => {
                                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                            event.accepted = true
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsStep(sRow.index, -1)
                                                        }
                                                    }

                                                    TapHandler {
                                                        onTapped: {
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsStep(sRow.index, -1)
                                                        }
                                                    }
                                                }

                                                Rectangle {
                                                    activeFocusOnTab: sRow.shown && !sRow.isSwitch && sRow.value.length > 0
                                                    Accessible.role: Accessible.Button
                                                    Accessible.name: sRow.value
                                                    Layout.preferredHeight: Math.round(28 * root.px)
                                                    radius: height / 2
                                                    color: ink.glass
                                                    border.width: activeFocus ? 2 : 1
                                                    border.color: activeFocus ? ink.corona : ink.pillBorder
                                                    implicitWidth: valTxt.implicitWidth + Math.round(20 * root.px)

                                                    Text {
                                                        id: valTxt
                                                        anchors.centerIn: parent
                                                        text: sRow.value
                                                        color: ink.textStrong
                                                        font.family: sRow.code ? Tokens.faceCode : Tokens.faceInterface
                                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                                        font.weight: Tokens.weightTitle
                                                    }

                                                    Keys.onPressed: (event) => {
                                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                            event.accepted = true
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsChoose(sRow.index)
                                                        }
                                                    }

                                                    TapHandler {
                                                        onTapped: {
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsChoose(sRow.index)
                                                        }
                                                    }
                                                }

                                                Rectangle {
                                                    visible: sRow.isChoice || sRow.isSlider
                                                    activeFocusOnTab: sRow.shown && (sRow.isChoice || sRow.isSlider)
                                                    Accessible.role: Accessible.Button
                                                    Accessible.name: sRow.label
                                                    Layout.preferredWidth: Math.round(24 * root.px)
                                                    Layout.preferredHeight: Layout.preferredWidth
                                                    radius: width / 2
                                                    color: ink.glass
                                                    border.width: activeFocus ? 2 : 1
                                                    border.color: activeFocus ? ink.corona : ink.hairline

                                                    DesktopGlyph {
                                                        anchors.centerIn: parent
                                                        width: Math.round(11 * root.px)
                                                        height: width
                                                        kind: 4
                                                        color: ink.textStrong
                                                    }

                                                    Keys.onPressed: (event) => {
                                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                            event.accepted = true
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsStep(sRow.index, 1)
                                                        }
                                                    }

                                                    TapHandler {
                                                        onTapped: {
                                                            Actions.settingsFocus(sRow.index)
                                                            Actions.settingsStep(sRow.index, 1)
                                                        }
                                                    }
                                                }
                                            }

                                            DesktopGlyph {
                                                visible: sRow.isDoor && sRow.value.length === 0
                                                Layout.preferredWidth: Math.round(13 * root.px)
                                                Layout.preferredHeight: Layout.preferredWidth
                                                kind: 4
                                                color: ink.muted
                                            }
                                        }

                                        HoverHandler {
                                            id: sRowHover
                                        }

                                        TapHandler {
                                            onTapped: {
                                                Actions.settingsFocus(sRow.index)
                                                if (sRow.isSwitch || sRow.isDoor) Actions.settingsChoose(sRow.index)
                                            }
                                        }
                                        }
                                }
                            }
                        }
                    }

                    // Inline Themes Manager pane when Settings.toolIndex === 5.
                    ColumnLayout {
                        visible: root.showingThemes
                        anchors.fill: parent
                        anchors.margins: Math.round(22 * root.px)
                        spacing: Math.round(14 * root.px)

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: Math.round(12 * root.px)

                            Rectangle {
                                activeFocusOnTab: root.showingThemes
                                Accessible.role: Accessible.Button
                                Accessible.name: Words.desktopBackLabel
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: ink.surfaceRaised
                                border.width: activeFocus ? 2 : 1
                                border.color: activeFocus ? ink.corona : ink.pillBorder
                                implicitWidth: tBackTxt.implicitWidth + Math.round(18 * root.px)

                                Text {
                                    id: tBackTxt
                                    anchors.centerIn: parent
                                    text: Words.desktopBackLabel
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                Keys.onPressed: (event) => {
                                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                        event.accepted = true
                                        Actions.settingsBack()
                                    }
                                }

                                TapHandler {
                                    onTapped: Actions.settingsBack()
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 2

                                Text {
                                    text: Words.desktopThemesHeading
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(20, Math.round(Tokens.fontTitle * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                Text {
                                    text: Words.desktopThemesSubtitle
                                    color: ink.muted
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                }
                            }

                            RowLayout {
                                spacing: Math.round(6 * root.px)
                                Repeater {
                                    model: Themes.kinds
                                    delegate: Rectangle {
                                        id: kBtn
                                        required property string modelData
                                        readonly property bool active: Themes.kind === kBtn.modelData
                                        activeFocusOnTab: root.showingThemes
                                        Accessible.role: Accessible.Button
                                        Accessible.name: kBtn.modelData
                                        height: Math.round(28 * root.px)
                                        radius: height / 2
                                        color: kBtn.active ? ink.textStrong : ink.glass
                                        border.width: kBtn.activeFocus ? 2 : 1
                                        border.color: kBtn.activeFocus ? ink.corona : (kBtn.active ? ink.textStrong : ink.hairline)
                                        implicitWidth: kTxt.implicitWidth + Math.round(18 * root.px)

                                        Text {
                                            id: kTxt
                                            anchors.centerIn: parent
                                            text: kBtn.modelData
                                            color: kBtn.active ? ink.surface : ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                            font.weight: Tokens.weightTitle
                                        }

                                        Keys.onPressed: (event) => {
                                            if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                event.accepted = true
                                                Actions.themesSetKind(kBtn.modelData)
                                            }
                                        }

                                        TapHandler {
                                            onTapped: Actions.themesSetKind(kBtn.modelData)
                                        }
                                    }
                                }
                            }
                        }

                        EdgeFade {
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            fadeTop: !themesList.atYBeginning
                            fadeBottom: !themesList.atYEnd
                            length: Math.round(14 * root.px)

                            ListView {
                                id: themesList
                                objectName: "eclipseDesktopThemesRows"
                                anchors.fill: parent
                                clip: true
                                spacing: Math.round(8 * root.px)
                                model: Themes.rows
                                readonly property bool fadeTop: !atYBeginning
                                readonly property bool fadeBottom: !atYEnd

                                delegate: Rectangle {
                                    id: themeRow
                                    required property int index
                                    required property string id
                                    required property string label
                                    required property string value
                                    required property bool steps
                                    required property bool active
                                    required property string badge
                                    readonly property bool focused: Themes.focused === themeRow.index
                                    activeFocusOnTab: root.showingThemes
                                    Accessible.role: Accessible.Button
                                    Accessible.name: themeRow.label
                                    width: themesList.width
                                    height: Math.round(58 * root.px)
                                    radius: Math.round(10 * root.px)
                                    color: themeRow.focused ? ink.glowWeak : ink.surfaceRaised
                                    border.width: themeRow.activeFocus ? 2 : (themeRow.focused ? 1.5 : 1)
                                    border.color: (themeRow.focused || themeRow.activeFocus) ? ink.corona : ink.hairline

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: Math.round(14 * root.px)
                                        anchors.rightMargin: Math.round(14 * root.px)
                                        spacing: Math.round(10 * root.px)

                                        Text {
                                            Layout.fillWidth: true
                                            text: themeRow.label
                                            elide: Text.ElideRight
                                            color: ink.textStrong
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(12, Math.round(14 * root.px))
                                            font.weight: Tokens.weightTitle
                                        }

                                        Rectangle {
                                            visible: themeRow.badge.length > 0
                                            Layout.preferredHeight: Math.round(22 * root.px)
                                            radius: height / 2
                                            color: ink.glass
                                            border.width: 1
                                            border.color: ink.hairline
                                            implicitWidth: thBadgeTxt.implicitWidth + Math.round(14 * root.px)

                                            Text {
                                                id: thBadgeTxt
                                                anchors.centerIn: parent
                                                text: themeRow.badge
                                                color: ink.muted
                                                font.family: Tokens.faceCode
                                                font.pixelSize: Math.max(10, Math.round(10.5 * root.px))
                                            }
                                        }

                                        Text {
                                            visible: !themeRow.active && themeRow.value.length > 0
                                            text: themeRow.value
                                            color: ink.muted
                                            font.family: Tokens.faceCode
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }

                                        Rectangle {
                                            activeFocusOnTab: root.showingThemes
                                            Accessible.role: Accessible.Button
                                            Accessible.name: themeRow.steps ? themeRow.value : Words.desktopUseThemeLabel
                                            Layout.preferredHeight: Math.round(28 * root.px)
                                            radius: height / 2
                                            color: themeRow.active ? ink.surfaceRaised : ink.accent
                                            border.width: activeFocus ? 2 : (themeRow.active ? 1 : 0)
                                            border.color: ink.corona
                                            implicitWidth: themeRow.steps ? Math.round(28 * root.px) : useThemeTxt.implicitWidth + Math.round(18 * root.px)

                                            Text {
                                                id: useThemeTxt
                                                visible: !themeRow.steps
                                                anchors.centerIn: parent
                                                text: themeRow.active ? Words.desktopInUseLabel : Words.desktopUseThemeLabel
                                                color: themeRow.active ? ink.textStrong : ink.overAccent
                                                font.family: Tokens.faceInterface
                                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                                font.weight: Tokens.weightTitle
                                            }

                                            DesktopGlyph {
                                                visible: themeRow.steps
                                                anchors.centerIn: parent
                                                width: Math.round(12 * root.px)
                                                height: width
                                                kind: 4
                                                color: themeRow.active ? ink.textStrong : ink.overAccent
                                            }

                                            Keys.onPressed: (event) => {
                                                if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                                    event.accepted = true
                                                    Actions.themesFocus(themeRow.index)
                                                    if (themeRow.steps) Actions.themesStep(themeRow.index, 1); else Actions.themesUse(themeRow.id)
                                                }
                                            }

                                            TapHandler {
                                                onTapped: {
                                                    Actions.themesFocus(themeRow.index)
                                                    if (themeRow.steps) Actions.themesStep(themeRow.index, 1); else Actions.themesUse(themeRow.id)
                                                }
                                            }
                                        }
                                    }

                                    Keys.onPressed: (event) => {
                                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                                            event.accepted = true
                                            Actions.themesFocus(themeRow.index)
                                        }
                                    }

                                    TapHandler {
                                        onTapped: Actions.themesFocus(themeRow.index)
                                    }
                                }
                            }
                        }
                    }
                }

                // Right live detail & preview pane.
                Rectangle {
                    objectName: "eclipseDesktopSettingsAside"
                    visible: Workspace.inspectorOpen
                    Layout.fillHeight: true
                    Layout.preferredWidth: Math.round(332 * root.px)
                    color: ink.railSurface
                    border.width: 1
                    border.color: ink.hairline

                    EdgeFade {
                        anchors.fill: parent
                        anchors.margins: Math.round(18 * root.px)
                        fadeTop: !asideFlick.atYBeginning
                        fadeBottom: !asideFlick.atYEnd
                        length: Math.round(12 * root.px)

                        Flickable {
                            id: asideFlick
                            anchors.fill: parent
                            contentWidth: width
                            contentHeight: asideCol.implicitHeight
                            clip: true
                            boundsBehavior: Flickable.StopAtBounds
                            readonly property bool fadeTop: !atYBeginning
                            readonly property bool fadeBottom: !atYEnd

                            ColumnLayout {
                                id: asideCol
                                width: asideFlick.width
                                spacing: Math.round(12 * root.px)

                                Text {
                                    text: Words.desktopAboutDetailHeading
                                    color: ink.muted
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    font.weight: Tokens.weightWordmark
                                    font.letterSpacing: 1.1
                                }

                                // Live visual preview plate (theme screenshot, settings screenshot, or Eclipse shader/planet plate).
                                Rectangle {
                                    Layout.fillWidth: true
                                    Layout.preferredHeight: Math.round(156 * root.px)
                                    radius: Math.round(10 * root.px)
                                    color: ink.surfaceRaised
                                    border.width: 1
                                    border.color: ink.hairline

                                    DesktopArtTile {
                                        anchors.fill: parent
                                        anchors.margins: 1
                                        visible: root.showingThemes ? Themes.preview.length === 0 : Settings.shot.length === 0
                                        px: root.px
                                        radius: 9
                                        planet: Settings.cardPlanet >= 0 ? Settings.cardPlanet : Settings.section
                                        slot: Settings.section + 2
                                        pixelScale: root.px
                                    }

                                    RoundPicture {
                                        anchors.fill: parent
                                        anchors.margins: 1
                                        visible: root.showingThemes ? Themes.preview.length > 0 : Settings.shot.length > 0
                                        source: root.showingThemes ? Themes.preview : Settings.shot
                                        radius: Math.round(9 * root.px)
                                        crop: true
                                        pixelScale: root.px
                                    }

                                    Rectangle {
                                        visible: !root.showingThemes && Settings.caption.length > 0
                                        anchors.left: parent.left
                                        anchors.top: parent.top
                                        anchors.margins: Math.round(10 * root.px)
                                        height: Math.round(24 * root.px)
                                        radius: height / 2
                                        color: ink.overArtScrim
                                        border.width: 1
                                        border.color: ink.pillBorder
                                        implicitWidth: capTxt.implicitWidth + Math.round(16 * root.px)

                                        Text {
                                            id: capTxt
                                            anchors.centerIn: parent
                                            text: Settings.caption
                                            color: ink.overArt
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                            font.weight: Tokens.weightTitle
                                        }
                                    }
                                }

                                Text {
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                    text: root.showingThemes ? Themes.name : (Settings.aboutHeading.length > 0 ? Settings.aboutHeading : Settings.heading)
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(16, Math.round(20 * root.px))
                                    font.weight: Tokens.weightTitle
                                }

                                Text {
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                    text: root.showingThemes ? Themes.line : Settings.sentence
                                    color: ink.text
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(13 * root.px))
                                }

                                ColumnLayout {
                                    visible: !root.showingThemes
                                    Layout.fillWidth: true
                                    spacing: Math.round(6 * root.px)

                                    Repeater {
                                        model: Settings.lines
                                        delegate: Text {
                                            id: aLine
                                            required property string modelData
                                            Layout.fillWidth: true
                                            wrapMode: Text.WordWrap
                                            text: aLine.modelData
                                            color: ink.text
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }
                                    }

                                    Repeater {
                                        model: Settings.muted
                                        delegate: Text {
                                            id: mLine
                                            required property string modelData
                                            Layout.fillWidth: true
                                            wrapMode: Text.WordWrap
                                            text: mLine.modelData
                                            color: ink.muted
                                            font.family: Tokens.faceInterface
                                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        }
                                    }
                                }

                                Text {
                                    visible: root.showingThemes && Themes.findings.length > 0
                                    Layout.fillWidth: true
                                    wrapMode: Text.WordWrap
                                    text: Themes.findings
                                    color: ink.dim
                                    font.family: Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
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
    }
}
