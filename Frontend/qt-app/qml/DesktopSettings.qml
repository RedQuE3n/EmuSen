pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// Reference Desktop Mode Settings & Tools screen: sections rail, interactive rows, inline Themes switcher, and live detail pane.
Item {
    id: root
    objectName: "desktopSettings"
    required property DesktopModel model
    required property SettingsModel settings
    required property LibraryModel library
    required property string platformLibrary
    required property var builtInManifests
    required property var chrome
    required property var settingsData
    required property real px

    signal themeUsed(string folder)
    signal themeRestyled(bool esde)

    readonly property bool showingThemes: root.model.settingsToolIndex === 5

    ThemesModel {
        id: themesModel
        objectName: "desktopThemesModel"
        platformLibrary: root.platformLibrary
        builtIns: root.builtInManifests
        open: root.visible && root.showingThemes
        onUsed: (folder) => root.themeUsed(folder)
        onRestyled: (esde) => {
            root.themeRestyled(esde)
            root.library.restyle()
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        // Left navigation rail: filter input, SETTINGS sections, and TOOLS.
        Rectangle {
            objectName: "desktopSettingsRail"
            visible: root.model.sidebarOpen
            Layout.fillHeight: true
            Layout.preferredWidth: Math.round(248 * root.px)
            color: ThemeInk.glass
            border.width: 1
            border.color: ThemeInk.hairline

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Math.round(14 * root.px)
                spacing: Math.round(10 * root.px)

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(32 * root.px)
                    radius: Math.round(7 * root.px)
                    color: ThemeInk.inputSurface
                    border.width: 1
                    border.color: filterField.activeFocus ? ThemeInk.corona : ThemeInk.hairline

                    TextField {
                        id: filterField
                        objectName: "desktopSettingsFilterInput"
                        anchors.fill: parent
                        leftPadding: Math.round(10 * root.px)
                        rightPadding: Math.round(10 * root.px)
                        color: ThemeInk.textStrong
                        placeholderTextColor: ThemeInk.dim
                        placeholderText: root.chrome.settingsFilterPlaceholder ?? ""
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        text: root.model.settingsFilter
                        background: Item {}
                        onTextChanged: if (text !== root.model.settingsFilter) root.model.settingsFilter = text
                    }
                }

                Text {
                    text: root.chrome.settingsHeading ?? ""
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
                        model: root.settingsData.sections ?? []
                        delegate: Rectangle {
                            id: secBtn
                            required property var modelData
                            readonly property int secIdx: secBtn.modelData.index ?? 0
                            readonly property bool current: !root.showingThemes && (root.settingsData.section ?? 0) === secIdx
                            Layout.fillWidth: true
                            height: Math.round(34 * root.px)
                            radius: Math.round(7 * root.px)
                            color: secBtn.current ? Kit.Tokens.textStrong : secHover.hovered ? ThemeInk.surfaceRaised : "transparent"
                            border.width: secBtn.current ? 1 : 0
                            border.color: ThemeInk.coronaSoft

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Math.round(12 * root.px)
                                anchors.right: parent.right
                                anchors.rightMargin: Math.round(12 * root.px)
                                elide: Text.ElideRight
                                text: secBtn.modelData.name ?? ""
                                color: secBtn.current ? Kit.Tokens.voidFill : ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(13 * root.px))
                                font.weight: secBtn.current ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                            }

                            HoverHandler {
                                id: secHover
                            }

                            TapHandler {
                                onTapped: root.model.selectSettingsSection(secBtn.secIdx)
                            }
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: ThemeInk.hairline
                }

                Text {
                    text: root.chrome.toolsHeading ?? ""
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
                        model: root.chrome.toolsList ?? []
                        delegate: Rectangle {
                            id: toolBtn
                            required property var modelData
                            required property int index
                            readonly property bool current: root.model.settingsToolIndex === toolBtn.index
                            Layout.fillWidth: true
                            height: Math.round(32 * root.px)
                            radius: Math.round(7 * root.px)
                            color: toolBtn.current ? Kit.Tokens.textStrong : toolHover.hovered ? ThemeInk.surfaceRaised : "transparent"

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: parent.left
                                anchors.leftMargin: Math.round(12 * root.px)
                                anchors.right: parent.right
                                anchors.rightMargin: Math.round(12 * root.px)
                                elide: Text.ElideRight
                                text: toolBtn.modelData.label ?? ""
                                color: toolBtn.current ? Kit.Tokens.voidFill : ThemeInk.text
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: toolBtn.current ? Kit.Tokens.weightTitle : Kit.Tokens.weightSentence
                            }

                            HoverHandler {
                                id: toolHover
                            }

                            TapHandler {
                                onTapped: root.model.openSettingsTool(toolBtn.index)
                            }
                        }
                    }
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }

        // Center content area: either Settings rows OR Desktop Themes manager.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Settings rows view.
            ColumnLayout {
                visible: !root.showingThemes
                anchors.fill: parent
                anchors.margins: Math.round(22 * root.px)
                spacing: Math.round(14 * root.px)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(10 * root.px)

                    Rectangle {
                        visible: root.settingsData.canBack ?? false
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: height / 2
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.pillBorder
                        implicitWidth: backTxt.implicitWidth + Math.round(18 * root.px)
                        Text {
                            id: backTxt
                            anchors.centerIn: parent
                            text: root.chrome.backLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        TapHandler {
                            onTapped: root.model.settingsBack()
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2

                        Text {
                            visible: (root.settingsData.trailLine ?? "").length > 0
                            text: root.settingsData.trailLine ?? ""
                            color: ThemeInk.muted
                            font.family: Kit.Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }

                        Text {
                            objectName: "desktopSettingsHeading"
                            text: root.settingsData.heading ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(20, Math.round(26 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                    }

                    Rectangle {
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: height / 2
                        color: root.model.settingsModifiedOnly ? Kit.Tokens.accent : ThemeInk.glass
                        border.width: 1
                        border.color: root.model.settingsModifiedOnly ? Kit.Tokens.accent : ThemeInk.hairline
                        implicitWidth: modTxt.implicitWidth + Math.round(20 * root.px)

                        Text {
                            id: modTxt
                            anchors.centerIn: parent
                            text: root.chrome.modifiedOnlyLabel ?? ""
                            color: root.model.settingsModifiedOnly ? Kit.Tokens.overAccent : ThemeInk.text
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }

                        TapHandler {
                            onTapped: root.model.settingsModifiedOnly = !root.model.settingsModifiedOnly
                        }
                    }
                }

                ListView {
                    id: settingsList
                    objectName: "desktopSettingsRows"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(8 * root.px)
                    model: root.settingsData.rows ?? []
                    ScrollBar.vertical: ScrollBar {
                        policy: ScrollBar.AsNeeded
                    }

                    delegate: Rectangle {
                        id: sRow
                        required property var modelData
                        readonly property int rowIdx: sRow.modelData.index ?? 0
                        readonly property bool shown: !root.model.settingsModifiedOnly || (sRow.modelData.marked ?? false)
                        readonly property bool focused: (root.settingsData.focused ?? -1) === sRow.rowIdx
                        width: settingsList.width
                        visible: sRow.shown
                        height: sRow.shown ? Math.round(58 * root.px) : 0
                        radius: Math.round(10 * root.px)
                        color: sRow.focused ? ThemeInk.glowWeak : ThemeInk.surfaceRaised
                        border.width: sRow.focused ? 1.5 : 1
                        border.color: sRow.focused ? ThemeInk.corona : ThemeInk.hairline

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
                                        text: sRow.modelData.label ?? ""
                                        color: ThemeInk.textStrong
                                        font.family: Kit.Tokens.faceInterface
                                        font.pixelSize: Math.max(12, Math.round(14 * root.px))
                                        font.weight: Kit.Tokens.weightTitle
                                    }
                                    Rectangle {
                                        visible: sRow.modelData.marked ?? false
                                        Layout.preferredWidth: Math.round(6 * root.px)
                                        Layout.preferredHeight: Layout.preferredWidth
                                        radius: width / 2
                                        color: Kit.Tokens.accent
                                    }
                                }

                                Text {
                                    visible: (sRow.modelData.about ?? "").length > 0
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    text: sRow.modelData.about ?? ""
                                    color: ThemeInk.dim
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                }
                            }

                            // Switch pill when row is boolean.
                            Rectangle {
                                visible: sRow.modelData.isSwitch ?? false
                                Layout.preferredWidth: Math.round(44 * root.px)
                                Layout.preferredHeight: Math.round(24 * root.px)
                                radius: height / 2
                                color: (sRow.modelData.switchOn ?? false) ? Kit.Tokens.accent : ThemeInk.track
                                Rectangle {
                                    x: (sRow.modelData.switchOn ?? false) ? parent.width - width - 3 : 3
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: Math.round(18 * root.px)
                                    height: width
                                    radius: width / 2
                                    color: Kit.Tokens.overAccent
                                }
                            }

                            // Value chip and stepper controls when non-boolean.
                            RowLayout {
                                visible: !(sRow.modelData.isSwitch ?? false) && (sRow.modelData.value ?? "").length > 0
                                spacing: Math.round(6 * root.px)

                                Rectangle {
                                    visible: (sRow.modelData.isChoice ?? false) || (sRow.modelData.isSlider ?? false)
                                    Layout.preferredWidth: Math.round(24 * root.px)
                                    Layout.preferredHeight: Layout.preferredWidth
                                    radius: width / 2
                                    color: ThemeInk.glass
                                    border.width: 1
                                    border.color: ThemeInk.hairline
                                    Text {
                                        anchors.centerIn: parent
                                        text: root.chrome.icons ? (root.chrome.sortDesc ?? "") : ""
                                        rotation: 90
                                        color: ThemeInk.textStrong
                                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    }
                                    TapHandler {
                                        onTapped: {
                                            root.model.settingsFocus(sRow.rowIdx)
                                            root.model.settingsStep(sRow.rowIdx, -1)
                                        }
                                    }
                                }

                                Rectangle {
                                    Layout.preferredHeight: Math.round(28 * root.px)
                                    radius: height / 2
                                    color: ThemeInk.glass
                                    border.width: 1
                                    border.color: ThemeInk.pillBorder
                                    implicitWidth: valTxt.implicitWidth + Math.round(20 * root.px)
                                    Text {
                                        id: valTxt
                                        anchors.centerIn: parent
                                        text: sRow.modelData.value ?? ""
                                        color: ThemeInk.textStrong
                                        font.family: (sRow.modelData.code ?? false) ? Kit.Tokens.faceCode : Kit.Tokens.faceInterface
                                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                        font.weight: Kit.Tokens.weightTitle
                                    }
                                    TapHandler {
                                        onTapped: {
                                            root.model.settingsFocus(sRow.rowIdx)
                                            root.model.settingsChoose(sRow.rowIdx)
                                        }
                                    }
                                }

                                Rectangle {
                                    visible: (sRow.modelData.isChoice ?? false) || (sRow.modelData.isSlider ?? false)
                                    Layout.preferredWidth: Math.round(24 * root.px)
                                    Layout.preferredHeight: Layout.preferredWidth
                                    radius: width / 2
                                    color: ThemeInk.glass
                                    border.width: 1
                                    border.color: ThemeInk.hairline
                                    Text {
                                        anchors.centerIn: parent
                                        text: root.chrome.icons ? (root.chrome.sortDesc ?? "") : ""
                                        rotation: -90
                                        color: ThemeInk.textStrong
                                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                                    }
                                    TapHandler {
                                        onTapped: {
                                            root.model.settingsFocus(sRow.rowIdx)
                                            root.model.settingsStep(sRow.rowIdx, 1)
                                        }
                                    }
                                }
                            }
                        }

                        TapHandler {
                            onTapped: {
                                root.model.settingsFocus(sRow.rowIdx)
                                if ((sRow.modelData.isSwitch ?? false) || (sRow.modelData.isDoor ?? false)) root.model.settingsChoose(sRow.rowIdx)
                            }
                        }
                    }
                }
            }

            // Inline Themes Manager view when toolIndex === 5.
            ColumnLayout {
                visible: root.showingThemes
                anchors.fill: parent
                anchors.margins: Math.round(22 * root.px)
                spacing: Math.round(14 * root.px)

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(12 * root.px)

                    Rectangle {
                        Layout.preferredHeight: Math.round(28 * root.px)
                        radius: height / 2
                        color: ThemeInk.surfaceRaised
                        border.width: 1
                        border.color: ThemeInk.pillBorder
                        implicitWidth: tBackTxt.implicitWidth + Math.round(18 * root.px)
                        Text {
                            id: tBackTxt
                            anchors.centerIn: parent
                            text: root.chrome.backLabel ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        TapHandler {
                            onTapped: root.model.settingsBack()
                        }
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        Text {
                            text: root.chrome.themesHeading ?? ""
                            color: ThemeInk.textStrong
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(20, Math.round(26 * root.px))
                            font.weight: Kit.Tokens.weightTitle
                        }
                        Text {
                            text: root.chrome.themesSubtitle ?? ""
                            color: ThemeInk.muted
                            font.family: Kit.Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                    }

                    RowLayout {
                        spacing: Math.round(6 * root.px)
                        Repeater {
                            model: themesModel.kinds
                            delegate: Rectangle {
                                id: kBtn
                                required property string modelData
                                readonly property bool active: themesModel.kind === kBtn.modelData
                                height: Math.round(28 * root.px)
                                radius: height / 2
                                color: kBtn.active ? Kit.Tokens.textStrong : ThemeInk.glass
                                border.width: 1
                                border.color: kBtn.active ? Kit.Tokens.textStrong : ThemeInk.hairline
                                implicitWidth: kTxt.implicitWidth + Math.round(18 * root.px)
                                Text {
                                    id: kTxt
                                    anchors.centerIn: parent
                                    text: kBtn.modelData
                                    color: kBtn.active ? Kit.Tokens.voidFill : ThemeInk.text
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }
                                TapHandler {
                                    onTapped: themesModel.kind = kBtn.modelData
                                }
                            }
                        }
                    }
                }

                ListView {
                    id: themesList
                    objectName: "desktopThemesRows"
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    clip: true
                    spacing: Math.round(8 * root.px)
                    model: themesModel

                    delegate: Rectangle {
                        id: themeRow
                        required property var model
                        required property int index
                        readonly property bool focused: themesModel.focused === themeRow.index
                        width: themesList.width
                        height: Math.round(58 * root.px)
                        radius: Math.round(10 * root.px)
                        color: themeRow.focused ? ThemeInk.glowWeak : ThemeInk.surfaceRaised
                        border.width: themeRow.focused ? 1.5 : 1
                        border.color: themeRow.focused ? ThemeInk.corona : ThemeInk.hairline

                        RowLayout {
                            anchors.fill: parent
                            anchors.leftMargin: Math.round(14 * root.px)
                            anchors.rightMargin: Math.round(14 * root.px)
                            spacing: Math.round(10 * root.px)

                            Text {
                                Layout.fillWidth: true
                                text: themeRow.model.label
                                elide: Text.ElideRight
                                color: ThemeInk.textStrong
                                font.family: Kit.Tokens.faceInterface
                                font.pixelSize: Math.max(12, Math.round(14 * root.px))
                                font.weight: Kit.Tokens.weightTitle
                            }

                            Text {
                                text: themeRow.model.value
                                color: ThemeInk.muted
                                font.family: Kit.Tokens.faceCode
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            }

                            Rectangle {
                                Layout.preferredHeight: Math.round(28 * root.px)
                                radius: height / 2
                                color: Kit.Tokens.accent
                                implicitWidth: useThemeTxt.implicitWidth + Math.round(18 * root.px)

                                Text {
                                    id: useThemeTxt
                                    anchors.centerIn: parent
                                    text: themeRow.model.steps ? (root.chrome.sortDesc ?? "") : (root.chrome.useThemeLabel ?? "")
                                    rotation: themeRow.model.steps ? -90 : 0
                                    color: Kit.Tokens.overAccent
                                    font.family: Kit.Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: Kit.Tokens.weightTitle
                                }

                                TapHandler {
                                    onTapped: {
                                        themesModel.focused = themeRow.index
                                        if (themeRow.model.steps) themesModel.step(themeRow.index, 1)
                                        else themesModel.accept()
                                    }
                                }
                            }
                        }

                        TapHandler {
                            onTapped: themesModel.focused = themeRow.index
                        }
                    }
                }
            }
        }

        // Right context detail pane: either focused setting description or focused theme preview.
        Rectangle {
            objectName: "desktopSettingsAside"
            visible: root.model.inspectorOpen
            Layout.fillHeight: true
            Layout.preferredWidth: Math.round(332 * root.px)
            color: ThemeInk.glass
            border.width: 1
            border.color: ThemeInk.hairline

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Math.round(18 * root.px)
                spacing: Math.round(12 * root.px)

                Text {
                    text: root.chrome.aboutDetailHeading ?? ""
                    color: ThemeInk.muted
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Kit.Tokens.weightWordmark
                    font.letterSpacing: 1.1
                }

                Rectangle {
                    visible: root.showingThemes
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(170 * root.px)
                    radius: Math.round(10 * root.px)
                    color: ThemeInk.surfaceRaised
                    border.width: 1
                    border.color: ThemeInk.hairline

                    Kit.RoundPicture {
                        anchors.fill: parent
                        anchors.margins: 1
                        visible: themesModel.preview.length > 0
                        source: themesModel.preview
                        radius: Math.round(9 * root.px)
                    }
                }

                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: root.showingThemes ? themesModel.name : (root.settingsData.asideHeading ?? "")
                    color: ThemeInk.textStrong
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(16, Math.round(20 * root.px))
                    font.weight: Kit.Tokens.weightTitle
                }

                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: root.showingThemes ? themesModel.line : (root.settingsData.asideSentence ?? "")
                    color: ThemeInk.text
                    font.family: Kit.Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(13 * root.px))
                }

                Repeater {
                    model: root.showingThemes ? [] : (root.settingsData.asideLines ?? [])
                    delegate: Text {
                        id: aLine
                        required property string modelData
                        Layout.fillWidth: true
                        wrapMode: Text.WordWrap
                        text: aLine.modelData
                        color: ThemeInk.text
                        font.family: Kit.Tokens.faceInterface
                        font.pixelSize: Math.max(11, Math.round(12 * root.px))
                    }
                }

                Text {
                    visible: root.showingThemes && themesModel.findings.length > 0
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: themesModel.findings
                    color: ThemeInk.dim
                    font.family: Kit.Tokens.faceCode
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                }

                Item {
                    Layout.fillHeight: true
                }
            }
        }
    }
}
