pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme
import ".."

// Collapsible Constellation & Consoles rail for Library and Table views.
Rectangle {
    id: root
    required property real px
    property bool tableMode: false

    objectName: "eclipseDesktopRail"
    Layout.fillHeight: true
    Layout.preferredWidth: Math.round(244 * root.px)
    color: ink.railSurface
    border.width: 1
    border.color: ink.hairline

    DesktopInk {
        id: ink
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Math.round(14 * root.px)
        spacing: Math.round(10 * root.px)

        Text {
            text: Words.desktopConstellationHeading
            color: ink.muted
            font.family: Tokens.faceInterface
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
            font.weight: Tokens.weightWordmark
            font.letterSpacing: 1.1
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: Math.round(3 * root.px)

            Repeater {
                model: Workspace.places
                delegate: Rectangle {
                    id: placeBtn
                    required property string key
                    required property string label
                    required property int count
                    readonly property bool current: Workspace.consoleKey === placeBtn.key
                    readonly property string iconPath: Marks.path(placeBtn.key)
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: placeBtn.label
                    Layout.fillWidth: true
                    height: Math.round(32 * root.px)
                    radius: Math.round(7 * root.px)
                    color: placeBtn.current ? ink.glowWeak : placeHover.hovered ? ink.surfaceRaised : "transparent"
                    border.width: placeBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : placeBtn.current ? 1 : 0
                    border.color: placeBtn.activeFocus ? ink.corona : ink.coronaSoft

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            Actions.workspaceSelectConsole(placeBtn.key)
                            event.accepted = true
                        }
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(10 * root.px)
                        anchors.rightMargin: Math.round(10 * root.px)
                        spacing: Math.round(8 * root.px)

                        Item {
                            visible: placeBtn.iconPath.length > 0
                            Layout.preferredWidth: Math.round(14 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            Shape {
                                width: 24
                                height: 24
                                scale: parent.width / 24
                                transformOrigin: Item.TopLeft
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    fillColor: "transparent"
                                    strokeColor: placeBtn.current ? ink.corona : ink.muted
                                    strokeWidth: 1.7
                                    capStyle: ShapePath.RoundCap
                                    joinStyle: ShapePath.RoundJoin
                                    PathSvg { path: placeBtn.iconPath }
                                }
                            }
                        }

                        DesktopGlyph {
                            visible: placeBtn.iconPath.length === 0
                            Layout.preferredWidth: Math.round(12 * root.px)
                            Layout.preferredHeight: Layout.preferredWidth
                            kind: 6
                            color: placeBtn.current ? ink.corona : ink.muted
                        }

                        Text {
                            Layout.fillWidth: true
                            text: placeBtn.label
                            color: placeBtn.current ? ink.textStrong : ink.text
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(13 * root.px))
                            font.weight: placeBtn.current ? Tokens.weightTitle : Tokens.weightSentence
                        }

                        Text {
                            text: String(placeBtn.count)
                            color: placeBtn.current ? ink.textStrong : ink.muted
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }

                    HoverHandler {
                        id: placeHover
                    }

                    TapHandler {
                        onTapped: Actions.workspaceSelectConsole(placeBtn.key)
                    }
                }
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: ink.hairline
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Math.round(6 * root.px)

            Text {
                Layout.fillWidth: true
                text: Words.desktopConsolesHeading
                color: ink.muted
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(10, Math.round(11 * root.px))
                font.weight: Tokens.weightWordmark
                font.letterSpacing: 1.1
            }

            Text {
                text: Workspace.countsLine
                color: ink.dim
                font.family: Tokens.faceCode
                font.pixelSize: Math.max(10, Math.round(11 * root.px))
            }
        }

        // Scope toggle pills: Active vs All.
        RowLayout {
            visible: !root.tableMode
            Layout.fillWidth: true
            spacing: Math.round(6 * root.px)

            Rectangle {
                id: scopeActiveBtn
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.desktopScopeActive
                Layout.fillWidth: true
                Layout.preferredHeight: Math.round(26 * root.px)
                radius: Math.round(6 * root.px)
                color: Workspace.consoleScope === 0 ? ink.surfaceRaised : "transparent"
                border.width: scopeActiveBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                border.color: scopeActiveBtn.activeFocus ? ink.corona : Workspace.consoleScope === 0 ? ink.coronaSoft : ink.hairline

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        Actions.workspaceSetConsoleScope(0)
                        event.accepted = true
                    }
                }

                Text {
                    anchors.centerIn: parent
                    text: Words.desktopScopeActive
                    color: Workspace.consoleScope === 0 ? ink.textStrong : ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Tokens.weightTitle
                }

                TapHandler {
                    onTapped: Actions.workspaceSetConsoleScope(0)
                }
            }

            Rectangle {
                id: scopeAllBtn
                activeFocusOnTab: true
                Accessible.role: Accessible.Button
                Accessible.name: Words.consoleShows[0]
                Layout.fillWidth: true
                Layout.preferredHeight: Math.round(26 * root.px)
                radius: Math.round(6 * root.px)
                color: Workspace.consoleScope === 1 ? ink.surfaceRaised : "transparent"
                border.width: scopeAllBtn.activeFocus ? Math.max(1.5, Tokens.ringThickness) : 1
                border.color: scopeAllBtn.activeFocus ? ink.corona : Workspace.consoleScope === 1 ? ink.coronaSoft : ink.hairline

                Keys.onPressed: (event) => {
                    if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                        Actions.workspaceSetConsoleScope(1)
                        event.accepted = true
                    }
                }

                Text {
                    anchors.centerIn: parent
                    text: Words.consoleShows[0]
                    color: Workspace.consoleScope === 1 ? ink.textStrong : ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Tokens.weightTitle
                }

                TapHandler {
                    onTapped: Actions.workspaceSetConsoleScope(1)
                }
            }
        }

        // Console filter input.
        Rectangle {
            visible: !root.tableMode
            Layout.fillWidth: true
            Layout.preferredHeight: Math.round(30 * root.px)
            radius: Math.round(7 * root.px)
            color: ink.inputSurface
            border.width: 1
            border.color: consoleFilterInput.activeFocus ? ink.corona : ink.hairline

            TextField {
                id: consoleFilterInput
                objectName: "eclipseDesktopConsoleFilter"
                anchors.fill: parent
                leftPadding: Math.round(10 * root.px)
                rightPadding: Math.round(10 * root.px)
                topPadding: 0
                bottomPadding: 0
                verticalAlignment: TextInput.AlignVCenter
                color: ink.textStrong
                placeholderTextColor: ink.dim
                placeholderText: Words.desktopConsoleFilterPlaceholder
                font.family: Tokens.faceInterface
                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                text: Workspace.consoleFilter
                background: Item {}
                signal syncModelText()
                onSyncModelText: if (text !== Workspace.consoleFilter) text = Workspace.consoleFilter
                onTextChanged: if (text !== Workspace.consoleFilter) Actions.workspaceSetConsoleFilter(text)
                Connections {
                    target: Workspace
                    function onChanged() { consoleFilterInput.syncModelText() }
                }
            }
        }

        EdgeFade {
            Layout.fillWidth: true
            Layout.fillHeight: true
            fadeTop: !consoleList.atYBeginning
            fadeBottom: !consoleList.atYEnd
            length: Math.round(14 * root.px)

            ListView {
                id: consoleList
                objectName: "eclipseDesktopConsoleList"
                anchors.fill: parent
                clip: true
                spacing: Math.round(3 * root.px)
                model: Workspace.consoles
                readonly property real usableWidth: Math.max(1, width - Math.max(12, Math.ceil(consoleScrollBar.width) + 2))
                readonly property bool fadeTop: !atYBeginning
                readonly property bool fadeBottom: !atYEnd
                ScrollBar.vertical: ScrollBar {
                    id: consoleScrollBar
                    policy: ScrollBar.AsNeeded
                }

                delegate: Rectangle {
                    id: conRow
                    required property string key
                    required property string name
                    required property int games
                    required property int planet
                    readonly property bool current: Workspace.consoleKey === conRow.key
                    activeFocusOnTab: true
                    Accessible.role: Accessible.Button
                    Accessible.name: conRow.name
                    width: consoleList.usableWidth
                    height: Math.round(31 * root.px)
                    radius: Math.round(7 * root.px)
                    color: conRow.current ? ink.glowWeak : conHover.hovered ? ink.surfaceRaised : "transparent"
                    border.width: conRow.activeFocus ? Math.max(1.5, Tokens.ringThickness) : conRow.current ? 1 : 0
                    border.color: conRow.activeFocus ? ink.corona : ink.coronaSoft

                    Keys.onPressed: (event) => {
                        if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter || event.key === Qt.Key_Space) {
                            Actions.workspaceSelectConsole(conRow.key)
                            event.accepted = true
                        }
                    }

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: Math.round(8 * root.px)
                        anchors.rightMargin: Math.round(10 * root.px)
                        spacing: Math.round(8 * root.px)

                        PlanetSphere {
                            unit: root.px
                            side: 15
                            glowBlur: 6
                            glowSpread: 1
                            planet: conRow.planet
                        }

                        Text {
                            Layout.fillWidth: true
                            text: conRow.name
                            elide: Text.ElideRight
                            color: conRow.current ? ink.textStrong : ink.text
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: conRow.current ? Tokens.weightTitle : Tokens.weightSentence
                        }

                        Text {
                            text: conRow.games > 0 ? String(conRow.games) : Words.desktopDash
                            color: conRow.current ? ink.textStrong : ink.dim
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }

                    HoverHandler {
                        id: conHover
                    }

                    TapHandler {
                        onTapped: Actions.workspaceSelectConsole(conRow.key)
                    }
                }
            }
        }

        // Bottom rail card: controller readiness on Library, read-only guarantee on Table.
        Rectangle {
            visible: !root.tableMode
            Layout.fillWidth: true
            Layout.preferredHeight: Math.round(34 * root.px)
            radius: Math.round(8 * root.px)
            color: ink.surfaceRaised
            border.width: 1
            border.color: ink.hairline

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Math.round(10 * root.px)
                anchors.rightMargin: Math.round(10 * root.px)
                spacing: Math.round(8 * root.px)

                Rectangle {
                    Layout.preferredWidth: Math.round(8 * root.px)
                    Layout.preferredHeight: Layout.preferredWidth
                    radius: width / 2
                    color: Device.controllers > 0 ? ink.success : ink.muted
                }

                Text {
                    Layout.fillWidth: true
                    text: Workspace.controllersLine
                    elide: Text.ElideRight
                    color: ink.text
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                }
            }
        }

        Rectangle {
            visible: root.tableMode
            Layout.fillWidth: true
            radius: Math.round(8 * root.px)
            color: ink.surfaceRaised
            border.width: 1
            border.color: ink.hairline
            implicitHeight: guaranteeCol.implicitHeight + Math.round(16 * root.px)

            ColumnLayout {
                id: guaranteeCol
                anchors.fill: parent
                anchors.margins: Math.round(8 * root.px)
                spacing: Math.round(4 * root.px)

                Text {
                    text: Words.desktopBatchGuaranteeTitle
                    color: ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Tokens.weightTitle
                }

                Text {
                    Layout.fillWidth: true
                    wrapMode: Text.WordWrap
                    text: Words.desktopContractNote
                    color: ink.dim
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(9, Math.round(10 * root.px))
                }
            }
        }
    }
}
