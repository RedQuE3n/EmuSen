pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A page on the frame: the hero across the top under the OnArt gradient, the cover over its foot, the title, facts, actions and section strip beside it, and the content below as tiles or rows, all from the host's models; it knows no game (EmuSen_Kit.md §7.44).
Kit.Frame {
    id: root
    property url hero
    // The hero's clip, which plays over the hero's picture while `clipPlaying` (Kit.VideoTile); silent unless `clipMuted` is false.
    property url clipSource
    property bool clipPlaying: false
    property bool clipMuted: true
    property url cover
    // The cover's width per unit of its height: 0.75 for a 3:4 box.
    property real coverAspect: 0.75
    property string coverName
    property string title
    property string facts
    // Roles `label`, `value`, `button` (a pad button, or below zero for none) and `filled` (the screen's main action).
    property alias actionModel: actionList.model
    // Roles `label`, `line` and `picture`.
    property alias tileModel: tileList.model
    // Roles `label`, `value`, `steps` and `opens`.
    property alias rowModel: rowList.model
    // Tiles where true, rows where false.
    property bool showsTiles: true
    // Words and a picture in place of tiles or rows, as a game's Overview shows them.
    property bool showsText: false
    property string contentWords
    property url contentPicture
    readonly property alias actionView: actionList
    readonly property alias tileView: tileList
    readonly property alias rowView: rowList
    readonly property alias videoTile: heroClip
    // The room a focused tile's ring takes outside it.
    readonly property real ringRoom: Kit.Tokens.ringOffset + Kit.Tokens.ringThickness
    signal acted(int index)
    signal tileChosen(int index)
    signal rowChosen(int index)
    // Left (-1) or Right (+1) on a stepped action that steps from where it stands (roles `stepLeft`, `stepRight`).
    signal actionStepped(int index, int by)
    // Left or Right on a row that steps.
    signal rowStepped(int index, int by)
    controller.navigationShown: false
    // Asked when a page opens, or opens on another game: the focus on the first action, as LunaP's page opens (EmuSen_Kit.md §7.58).
    signal focusFirstAction()
    onFocusFirstAction: {
        actionList.currentIndex = 0
        actionList.focus = true
    }
    // The content's kind changed with the focus in it: the focus to the list now shown, on its first item, or to the
    // actions where words are shown, as LunaP's FillContent hands it on (EmuSen_Kit.md §7.58).
    signal contentKindChanged()
    onShowsTilesChanged: root.contentKindChanged()
    onShowsTextChanged: root.contentKindChanged()
    onContentKindChanged: {
        if (!tileList.activeFocus && !rowList.activeFocus)
            return
        if (root.showsText) {
            actionList.forceActiveFocus()
            return
        }
        const list = root.showsTiles ? tileList : rowList
        list.currentIndex = 0
        list.forceActiveFocus()
    }
    Kit.PageLayout {
        id: places
        width: root.width
        height: root.height
        aspect: root.coverAspect
    }
    // The page's own places over the whole frame, inside its focus scope, so the frame's walk and help bar serve them.
    Item {
        objectName: "page"
        parent: root
        width: root.width
        height: root.height
        Rectangle {
            objectName: "pageHero"
            // Drawn behind the page, as a sky is: what stands over it is not an overlap.
            readonly property bool backdrop: true
            x: places.hero.x
            y: places.hero.y
            width: places.hero.width
            height: places.hero.height
            color: Kit.Tokens.voidFill
            clip: true
            Kit.VideoTile {
                id: heroClip
                objectName: "pageClip"
                anchors.fill: parent
                still: root.hero
                source: root.clipSource
                playing: root.clipPlaying
                muted: root.clipMuted
            }
            Rectangle {
                anchors.fill: parent
                gradient: Gradient {
                    GradientStop { position: 0; color: "transparent" }
                    GradientStop { position: 1; color: Kit.Tokens.overArtScrim }
                }
            }
        }
        // The cover is raised over the hero's foot, so it casts §5.4's shadow (EmuSen_Kit.md §7.55).
        Kit.RaisedShadow {
            x: places.cover.x
            y: places.cover.y
            width: places.cover.width
            height: places.cover.height
            radius: Kit.Tokens.rowRadius
        }
        Rectangle {
            objectName: "pageCover"
            x: places.cover.x
            y: places.cover.y
            width: places.cover.width
            height: places.cover.height
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.surfaceRaised
            border.width: Kit.Tokens.hoverBorder
            border.color: Kit.Tokens.border
            Accessible.role: Accessible.Graphic
            Accessible.name: root.coverName
            Image {
                anchors.fill: parent
                source: root.cover
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
        }
        Text {
            objectName: "pageTitle"
            x: places.title.x
            y: places.title.y
            width: places.title.width
            text: root.title
            color: Kit.Tokens.overArt
            font.pixelSize: Kit.Tokens.fontDisplay
            fontSizeMode: Text.HorizontalFit
            minimumPixelSize: Kit.Tokens.fontTitle
        }
        Text {
            objectName: "pageFacts"
            x: places.facts.x
            y: places.facts.y
            width: places.facts.width
            text: root.facts
            color: Kit.Tokens.overArt
            opacity: 0.7
            font.pixelSize: Kit.Tokens.fontBody
            fontSizeMode: Text.HorizontalFit
            minimumPixelSize: Kit.Tokens.fontCaption
        }
        ListView {
            id: actionList
            // A model that fills after the view was made leaves it with no current item, so no delegate would take the focus.
            onCountChanged: if (currentIndex < 0 && count > 0) currentIndex = 0
            objectName: "pageActions"
            x: places.actions.x
            y: places.actions.y
            width: places.actions.width
            height: places.actions.height
            orientation: ListView.Horizontal
            interactive: false
            focus: true
            keyNavigationEnabled: true
            spacing: Kit.Tokens.base * 3
            highlightMoveDuration: Kit.Tokens.instant
            KeyNavigation.down: root.showsText ? null : (root.showsTiles ? tileList : rowList)
            delegate: Kit.ActionButton {
                id: action
                required label
                required value
                required button
                required filled
                required stepped
                required property bool stepLeft
                required property bool stepRight
                required property int index
                controller: root.controller
                focus: ListView.isCurrentItem
                family: root.padFamily
                onActivated: root.acted(action.index)
                Keys.onLeftPressed: (event) => {
                    event.accepted = action.stepLeft
                    if (action.stepLeft)
                        root.actionStepped(action.index, -1)
                }
                Keys.onRightPressed: (event) => {
                    event.accepted = action.stepRight
                    if (action.stepRight)
                        root.actionStepped(action.index, 1)
                }
                onActiveFocusChanged: actionList.currentIndex = action.activeFocus ? action.index : actionList.currentIndex
            }
        }
        Kit.SectionStrip {
            objectName: "pageSections"
            controller: root.controller
            x: places.strip.x
            y: places.strip.y
            width: places.strip.width
            height: places.strip.height
        }
        Text {
            objectName: "pageWords"
            visible: root.showsText
            x: places.content.x
            y: places.content.y
            width: places.content.width - places.pictureWidth - places.pictureGap
            height: places.content.height
            text: root.contentWords
            color: Kit.Tokens.text
            font.pixelSize: Kit.Tokens.fontBody
            wrapMode: Text.Wrap
            elide: Text.ElideRight
        }
        Rectangle {
            objectName: "pagePicture"
            visible: root.showsText
            x: places.content.x + places.content.width - places.pictureWidth
            y: places.content.y
            width: places.pictureWidth
            height: places.pictureHeight
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.surfaceRaised
            clip: true
            Image {
                anchors.fill: parent
                source: root.contentPicture
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
        }
        Kit.EdgeFade {
            visible: root.showsTiles && !root.showsText
            x: places.content.x - root.ringRoom
            y: places.content.y - root.ringRoom
            width: places.content.width + 2 * root.ringRoom
            // The ring's room above and below the content's box, as at its sides.
            height: places.content.height + 2 * root.ringRoom
            fadeLeft: tileList.fadeLeft
            fadeRight: tileList.fadeRight
            ListView {
                id: tileList
                // The edges faded, declared on the scroller, where the fit audit reads them (EmuSen_Kit.md §7.50).
                readonly property bool fadeLeft: !tileList.atXBeginning
                readonly property bool fadeRight: !tileList.atXEnd
                // A model that fills after the view was made leaves it with no current item, so no delegate would take the focus.
                onCountChanged: if (currentIndex < 0 && count > 0) currentIndex = 0
                objectName: "pageTiles"
                anchors.fill: parent
                leftMargin: root.ringRoom
                rightMargin: root.ringRoom
                orientation: ListView.Horizontal
                clip: true
                keyNavigationEnabled: true
                boundsBehavior: Flickable.StopAtBounds
                spacing: places.tileGap
                highlightMoveDuration: Kit.Tokens.instant
                KeyNavigation.up: actionList
                delegate: Kit.Walkable {
                    id: tile
                    required property string label
                    required property string line
                    required property url picture
                    required property int index
                    width: places.tileWidth
                    // A horizontal ListView places its delegates at its top, so the tile's parts stand the ring's room below it, and it reaches the room below the content.
                    height: places.content.height + 2 * root.ringRoom
                    controller: root.controller
                    focus: ListView.isCurrentItem
                    Accessible.role: Accessible.ListItem
                    Accessible.name: tile.label
                    onActivated: root.tileChosen(tile.index)
                    onActiveFocusChanged: tileList.currentIndex = tile.activeFocus ? tile.index : tileList.currentIndex
                    Rectangle {
                        y: root.ringRoom
                        width: places.tileWidth
                        height: places.tileHeight
                        radius: Kit.Tokens.rowRadius
                        color: Kit.Tokens.surfaceRaised
                        clip: true
                        Image {
                            anchors.fill: parent
                            source: tile.picture
                            fillMode: Image.PreserveAspectCrop
                            asynchronous: true
                        }
                    }
                    Rectangle {
                        objectName: "ring"
                        visible: tile.activeFocus
                        x: -Kit.Tokens.ringOffset
                        y: root.ringRoom - Kit.Tokens.ringOffset
                        width: places.tileWidth + 2 * Kit.Tokens.ringOffset
                        height: places.tileHeight + 2 * Kit.Tokens.ringOffset
                        radius: Kit.Tokens.rowRadius + Kit.Tokens.ringOffset
                        color: "transparent"
                        border.width: Kit.Tokens.ringThickness
                        border.color: Kit.Tokens.accent
                    }
                    Text {
                        y: root.ringRoom + places.labelOffset
                        width: places.tileWidth
                        text: tile.label
                        // Lines at 1.2 of the size, Barlow's, as the recording's tiles set them, so two lines fit under the picture.
                        lineHeightMode: Text.FixedHeight
                        lineHeight: Math.round(Kit.Tokens.fontCaption * 1.2)
                        color: tile.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
                        font.pixelSize: Kit.Tokens.fontCaption
                        wrapMode: Text.Wrap
                    }
                    Text {
                        y: root.ringRoom + places.lineOffset
                        width: places.tileWidth
                        text: tile.line
                        // Lines at 1.2 of the size, Barlow's, as the recording's tiles set them, so two lines fit under the picture.
                        lineHeightMode: Text.FixedHeight
                        lineHeight: Math.round(Kit.Tokens.fontCaption * 1.2)
                        color: Kit.Tokens.muted
                        font.pixelSize: Kit.Tokens.fontCaption
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
        Kit.EdgeFade {
            visible: !root.showsTiles && !root.showsText
            x: places.content.x
            y: places.content.y
            width: places.content.width
            height: places.rows * Kit.Tokens.rowHeight
            fadeTop: rowList.fadeTop
            fadeBottom: rowList.fadeBottom
            ListView {
                id: rowList
                readonly property bool fadeTop: !rowList.atYBeginning
                readonly property bool fadeBottom: !rowList.atYEnd
                // A model that fills after the view was made leaves it with no current item, so no delegate would take the focus.
                onCountChanged: if (currentIndex < 0 && count > 0) currentIndex = 0
                objectName: "pageRows"
                anchors.fill: parent
                clip: true
                keyNavigationEnabled: true
                boundsBehavior: Flickable.StopAtBounds
                highlightMoveDuration: Kit.Tokens.instant
                highlightResizeDuration: Kit.Tokens.instant
                KeyNavigation.up: actionList
                delegate: Kit.MenuRow {
                    id: row
                    required label
                    required value
                    required steps
                    required opens
                    required property int index
                    width: ListView.view.width
                    controller: root.controller
                    ruled: true
                    focus: ListView.isCurrentItem
                    onActivated: root.rowChosen(row.index)
                    Keys.onLeftPressed: (event) => {
                        event.accepted = row.steps
                        if (row.steps)
                            root.rowStepped(row.index, -1)
                    }
                    Keys.onRightPressed: (event) => {
                        event.accepted = row.steps
                        if (row.steps)
                            root.rowStepped(row.index, 1)
                    }
                    onActiveFocusChanged: rowList.currentIndex = row.activeFocus ? row.index : rowList.currentIndex
                }
            }
        }
    }
}
