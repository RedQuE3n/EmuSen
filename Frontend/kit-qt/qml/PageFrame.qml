pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// A page on the frame: the hero across the top under the OnArt gradient, the cover over its foot, the title, facts, actions and section strip beside it, and the content below as tiles or rows, all from the host's models; it knows no game (EmuSen_Kit.md §7.40).
Kit.Frame {
    id: root
    property url hero
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
    readonly property alias actionView: actionList
    readonly property alias tileView: tileList
    readonly property alias rowView: rowList
    // The room a focused tile's ring takes outside it.
    readonly property real ringRoom: Kit.Tokens.ringOffset + Kit.Tokens.ringThickness
    signal acted(int index)
    signal tileChosen(int index)
    signal rowChosen(int index)
    controller.navigationShown: false
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
            Image {
                anchors.fill: parent
                source: root.hero
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }
            Rectangle {
                anchors.fill: parent
                gradient: Gradient {
                    GradientStop { position: 0; color: "transparent" }
                    GradientStop { position: 1; color: Kit.Tokens.overArtScrim }
                }
            }
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
            KeyNavigation.down: root.showsTiles ? tileList : rowList
            delegate: Kit.ActionButton {
                id: action
                required label
                required value
                required button
                required filled
                required property int index
                controller: root.controller
                focus: ListView.isCurrentItem
                family: root.padFamily
                onActivated: root.acted(action.index)
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
        Kit.EdgeFade {
            visible: root.showsTiles
            x: places.content.x - root.ringRoom
            y: places.content.y - root.ringRoom
            width: places.content.width + 2 * root.ringRoom
            height: places.content.height + root.ringRoom
            fadeLeft: !tileList.atXBeginning
            fadeRight: !tileList.atXEnd
            ListView {
                id: tileList
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
                    height: places.content.height
                    y: root.ringRoom
                    controller: root.controller
                    focus: ListView.isCurrentItem
                    Accessible.role: Accessible.ListItem
                    Accessible.name: tile.label
                    onActivated: root.tileChosen(tile.index)
                    onActiveFocusChanged: tileList.currentIndex = tile.activeFocus ? tile.index : tileList.currentIndex
                    Rectangle {
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
                        y: -Kit.Tokens.ringOffset
                        width: places.tileWidth + 2 * Kit.Tokens.ringOffset
                        height: places.tileHeight + 2 * Kit.Tokens.ringOffset
                        radius: Kit.Tokens.rowRadius + Kit.Tokens.ringOffset
                        color: "transparent"
                        border.width: Kit.Tokens.ringThickness
                        border.color: Kit.Tokens.accent
                    }
                    Text {
                        y: places.labelOffset
                        width: places.tileWidth
                        text: tile.label
                        color: tile.activeFocus ? Kit.Tokens.textStrong : Kit.Tokens.text
                        font.pixelSize: Kit.Tokens.fontCaption
                        wrapMode: Text.Wrap
                    }
                    Text {
                        y: places.lineOffset
                        width: places.tileWidth
                        text: tile.line
                        color: Kit.Tokens.muted
                        font.pixelSize: Kit.Tokens.fontCaption
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
        Kit.EdgeFade {
            visible: !root.showsTiles
            x: places.content.x
            y: places.content.y
            width: places.content.width
            height: places.rows * Kit.Tokens.rowHeight
            fadeTop: !rowList.atYBeginning
            fadeBottom: !rowList.atYEnd
            ListView {
                id: rowList
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
                    onActiveFocusChanged: rowList.currentIndex = row.activeFocus ? row.index : rowList.currentIndex
                }
            }
        }
    }
}
