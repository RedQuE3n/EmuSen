pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import QtQuick.Shapes
import EmuSen.Theme
import ".."

// Right-hand Game Inspector pane for Eclipse Desktop Library.
Rectangle {
    id: root
    required property real px
    required property real pixelScale

    objectName: "eclipseDesktopInspector"
    Layout.fillHeight: true
    Layout.preferredWidth: Math.round(344 * root.px)
    color: ink.railSurface
    border.width: 1
    border.color: ink.hairline

    DesktopInk {
        id: ink
    }

    EdgeFade {
        anchors.fill: parent
        anchors.margins: Math.round(16 * root.px)
        fadeTop: !inspectorFlick.atYBeginning
        fadeBottom: !inspectorFlick.atYEnd
        length: Math.round(14 * root.px)

        Flickable {
            id: inspectorFlick
            anchors.fill: parent
            contentWidth: width
            contentHeight: inspectorCol.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            readonly property bool fadeTop: !atYBeginning
            readonly property bool fadeBottom: !atYEnd

            ColumnLayout {
                id: inspectorCol
                width: inspectorFlick.width
                spacing: Math.round(12 * root.px)

                // Header row: INSPECTOR + Favourite + Page buttons.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Text {
                        Layout.fillWidth: true
                        text: Words.desktopInspectorHeading
                        color: ink.muted
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        font.weight: Tokens.weightWordmark
                        font.letterSpacing: 1.1
                    }

                    Rectangle {
                        objectName: "eclipseInspectorFavBtn"
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Workspace.inspectorFavourite ? Words.desktopUnfavouriteLabel : Words.desktopFavouriteLabel
                        Layout.preferredWidth: Math.round(28 * root.px)
                        Layout.preferredHeight: Layout.preferredWidth
                        radius: width / 2
                        color: ink.surfaceRaised
                        border.width: 1
                        border.color: Workspace.inspectorFavourite ? ink.accentSoft : ink.hairline

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
                                    fillColor: Workspace.inspectorFavourite ? ink.accent : "transparent"
                                    strokeColor: Workspace.inspectorFavourite ? ink.accent : ink.text
                                    strokeWidth: 1.8
                                    capStyle: ShapePath.RoundCap
                                    joinStyle: ShapePath.RoundJoin
                                    PathSvg { path: Marks.heart }
                                }
                            }
                        }

                        TapHandler {
                            onTapped: Actions.favourite(Workspace.inspectorKey)
                        }
                    }

                    Rectangle {
                        objectName: "eclipseInspectorPageIconBtn"
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopDetailsLabel
                        Layout.preferredWidth: Math.round(28 * root.px)
                        Layout.preferredHeight: Layout.preferredWidth
                        radius: width / 2
                        color: ink.surfaceRaised
                        border.width: 1
                        border.color: ink.hairline

                        DesktopGlyph {
                            anchors.centerIn: parent
                            width: Math.round(13 * root.px)
                            height: width
                            kind: 4
                            color: ink.textStrong
                        }

                        TapHandler {
                            onTapped: Actions.openPage(Workspace.inspectorKey)
                        }
                    }
                }

                // Hero preview plate.
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: Math.round(128 * root.px)
                    radius: Math.round(10 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline

                    DesktopArtTile {
                        anchors.fill: parent
                        anchors.margins: 1
                        px: root.px
                        radius: 9
                        source: Workspace.inspectorHero.length > 0 ? Workspace.inspectorHero : Workspace.inspectorCover
                        planet: Workspace.inspectorPlanet
                        slot: 1
                        pixelScale: root.pixelScale
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: Workspace.inspectorSubtitle
                    elide: Text.ElideRight
                    color: ink.rose
                    font.family: Tokens.faceCode
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.letterSpacing: 0.6
                }

                Text {
                    objectName: "eclipseInspectorTitle"
                    Layout.fillWidth: true
                    text: Workspace.inspectorTitle
                    wrapMode: Text.WordWrap
                    color: ink.textStrong
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(18, Math.round(22 * root.px))
                    font.weight: Tokens.weightTitle
                }

                // Primary Play / Resume buttons.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(8 * root.px)

                    Rectangle {
                        objectName: "eclipseInspectorPlayBtn"
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopPlayLabel
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(38 * root.px)
                        radius: height / 2
                        color: ink.accent

                        RowLayout {
                            anchors.centerIn: parent
                            spacing: Math.round(6 * root.px)

                            DesktopGlyph {
                                Layout.preferredWidth: Math.round(12 * root.px)
                                Layout.preferredHeight: Layout.preferredWidth
                                kind: 6
                                color: ink.overAccent
                            }

                            Text {
                                text: Words.desktopPlayLabel
                                color: ink.overAccent
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(12, Math.round(13 * root.px))
                                font.weight: Tokens.weightTitle
                            }
                        }

                        TapHandler {
                            onTapped: Actions.play(Workspace.inspectorKey)
                        }
                    }

                    Rectangle {
                        objectName: "eclipseInspectorResumeBtn"
                        visible: Workspace.inspectorRunning
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopResumeLabel
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(38 * root.px)
                        radius: height / 2
                        color: ink.surfaceRaised
                        border.width: 1
                        border.color: ink.corona

                        Text {
                            anchors.centerIn: parent
                            text: Words.desktopResumeLabel
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(12, Math.round(13 * root.px))
                            font.weight: Tokens.weightTitle
                        }

                        TapHandler {
                            onTapped: Actions.play(Session.key)
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: ink.hairline
                }

                Text {
                    text: Workspace.inspectorStatesHeading.length > 0 ? Workspace.inspectorStatesHeading : Words.desktopSaveStatesLabel
                    color: ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Tokens.weightWordmark
                    font.letterSpacing: 1.0
                }

                Text {
                    visible: Workspace.inspectorStatesCount === 0
                    text: Words.desktopNoStatesLabel
                    color: ink.dim
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                }

                ColumnLayout {
                    visible: Workspace.inspectorStatesCount > 0
                    Layout.fillWidth: true
                    spacing: Math.round(6 * root.px)

                    Repeater {
                        model: Workspace.inspectorStates
                        delegate: Rectangle {
                            id: stRow
                            required property string path
                            required property string label
                            required property string whenWords
                            required property string picture
                            required property bool isAuto
                            activeFocusOnTab: true
                            Accessible.role: Accessible.Button
                            Accessible.name: stRow.label
                            Layout.fillWidth: true
                            Layout.preferredHeight: Math.round(40 * root.px)
                            radius: Math.round(8 * root.px)
                            color: stHover.hovered ? ink.glowWeak : ink.surfaceRaised
                            border.width: 1
                            border.color: stRow.isAuto ? ink.accentSoft : ink.hairline

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Math.round(8 * root.px)
                                anchors.rightMargin: Math.round(10 * root.px)
                                spacing: Math.round(8 * root.px)

                                Rectangle {
                                    visible: stRow.picture.length > 0
                                    Layout.preferredWidth: Math.round(36 * root.px)
                                    Layout.preferredHeight: Math.round(24 * root.px)
                                    radius: Math.round(4 * root.px)
                                    clip: true
                                    color: ink.glass

                                    RoundPicture {
                                        anchors.fill: parent
                                        source: stRow.picture
                                        radius: Math.round(4 * root.px)
                                        crop: true
                                        pixelScale: root.pixelScale
                                    }
                                }

                                Text {
                                    Layout.fillWidth: true
                                    elide: Text.ElideRight
                                    text: stRow.label
                                    color: ink.textStrong
                                    font.family: Tokens.faceInterface
                                    font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                    font.weight: stRow.isAuto ? Tokens.weightTitle : Tokens.weightSentence
                                }

                                Text {
                                    text: stRow.whenWords
                                    color: ink.muted
                                    font.family: Tokens.faceCode
                                    font.pixelSize: Math.max(10, Math.round(10 * root.px))
                                }
                            }

                            HoverHandler {
                                id: stHover
                            }

                            TapHandler {
                                onTapped: Actions.playFrom(Workspace.inspectorKey, stRow.path)
                            }
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 1
                    color: ink.hairline
                }

                Text {
                    text: Words.desktopProgramFactsLabel
                    color: ink.muted
                    font.family: Tokens.faceInterface
                    font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    font.weight: Tokens.weightWordmark
                    font.letterSpacing: 1.0
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(6 * root.px)

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Math.round(6 * root.px)
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactCore
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        PlanetSphere {
                            unit: root.px
                            side: 12
                            glowBlur: 5
                            glowSpread: 1
                            planet: Workspace.inspectorPlanet
                        }
                        Text {
                            text: Workspace.inspectorCoreLine
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactPlayed
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        Text {
                            text: Workspace.inspectorPlayedWords
                            color: ink.textStrong
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                    }

                    RowLayout {
                        visible: Workspace.inspectorPlayers.length > 0
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactPlayers
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        Text {
                            text: Workspace.inspectorPlayers
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                    }

                    RowLayout {
                        visible: Workspace.inspectorGenre.length > 0
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactGenre
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        Text {
                            text: Workspace.inspectorGenre
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                    }

                    RowLayout {
                        visible: Workspace.inspectorFileName.length > 0
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactFile
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        Text {
                            Layout.maximumWidth: Math.round(180 * root.px)
                            elide: Text.ElideMiddle
                            text: Workspace.inspectorFileName
                            color: ink.text
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }

                    RowLayout {
                        visible: Workspace.inspectorSizeWords.length > 0
                        Layout.fillWidth: true
                        Text {
                            Layout.fillWidth: true
                            text: Words.desktopFactSize
                            color: ink.dim
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                        }
                        Text {
                            text: Workspace.inspectorSizeWords
                            color: ink.text
                            font.family: Tokens.faceCode
                            font.pixelSize: Math.max(10, Math.round(11 * root.px))
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    radius: Math.round(8 * root.px)
                    color: ink.surfaceRaised
                    border.width: 1
                    border.color: ink.hairline
                    implicitHeight: contractNoteLabel.implicitHeight + Math.round(16 * root.px)

                    Text {
                        id: contractNoteLabel
                        anchors.fill: parent
                        anchors.margins: Math.round(8 * root.px)
                        wrapMode: Text.WordWrap
                        text: Words.desktopContractNote
                        color: ink.dim
                        font.family: Tokens.faceInterface
                        font.pixelSize: Math.max(10, Math.round(11 * root.px))
                    }
                }

                // Bottom secondary actions: Page, Favourite, Folder.
                RowLayout {
                    Layout.fillWidth: true
                    spacing: Math.round(6 * root.px)

                    Rectangle {
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopDetailsLabel
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(32 * root.px)
                        radius: Math.round(7 * root.px)
                        color: pageBtnHover.hovered ? ink.cardHover : ink.surfaceRaised
                        border.width: 1
                        border.color: ink.hairline

                        Text {
                            anchors.centerIn: parent
                            text: Words.desktopDetailsLabel
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Tokens.weightTitle
                        }

                        HoverHandler {
                            id: pageBtnHover
                        }

                        TapHandler {
                            onTapped: Actions.openPage(Workspace.inspectorKey)
                        }
                    }

                    Rectangle {
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Workspace.inspectorFavourite ? Words.desktopUnfavouriteLabel : Words.desktopFavouriteLabel
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(32 * root.px)
                        radius: Math.round(7 * root.px)
                        color: favBtnHover.hovered ? ink.cardHover : ink.surfaceRaised
                        border.width: 1
                        border.color: ink.hairline

                        Text {
                            anchors.centerIn: parent
                            text: Workspace.inspectorFavourite ? Words.desktopUnfavouriteLabel : Words.desktopFavouriteLabel
                            color: ink.textStrong
                            font.family: Tokens.faceInterface
                            font.pixelSize: Math.max(11, Math.round(12 * root.px))
                            font.weight: Tokens.weightTitle
                        }

                        HoverHandler {
                            id: favBtnHover
                        }

                        TapHandler {
                            onTapped: Actions.favourite(Workspace.inspectorKey)
                        }
                    }

                    Rectangle {
                        activeFocusOnTab: true
                        Accessible.role: Accessible.Button
                        Accessible.name: Words.desktopFolderLabel
                        Layout.fillWidth: true
                        Layout.preferredHeight: Math.round(32 * root.px)
                        radius: Math.round(7 * root.px)
                        color: folderBtnHover.hovered ? ink.cardHover : ink.surfaceRaised
                        border.width: 1
                        border.color: ink.hairline

                        RowLayout {
                            anchors.centerIn: parent
                            spacing: Math.round(4 * root.px)
                            DesktopGlyph {
                                Layout.preferredWidth: Math.round(12 * root.px)
                                Layout.preferredHeight: Layout.preferredWidth
                                kind: 9
                                color: ink.textStrong
                            }
                            Text {
                                text: Words.desktopFolderLabel
                                color: ink.textStrong
                                font.family: Tokens.faceInterface
                                font.pixelSize: Math.max(11, Math.round(12 * root.px))
                                font.weight: Tokens.weightTitle
                            }
                        }

                        HoverHandler {
                            id: folderBtnHover
                        }

                        TapHandler {
                            onTapped: Actions.workspaceOpenFolder(Workspace.inspectorKey)
                        }
                    }
                }
            }
        }
    }
}
