pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Models
import EmuSen.App

// The grid's search: its words, which the keyboard types and the grid's model narrows by, the field over the one row of
// covers and the keyboard docked beneath, over the theme's Games screen (EmuSen_Interface.md §18.1; EmuSen_Qt.md §7.6).
Kit.NormalizedCanvas {
    id: searchCanvas
    objectName: "searchCanvas"
    required property Screens screens
    required property GameModel games
    // The reference's host, whose Games screen the search is over.
    required property ThemeHost host
    required property int padFamily
    required property bool gameShown
    required property bool splashShown
    visible: !searchCanvas.gameShown && searchCanvas.host.searching
    Kit.KeyboardModel {
        id: typist
        objectName: "typist"
        onTextChanged: searchCanvas.games.search = typist.text
        onClosed: searchCanvas.screens.back()
    }
    Connections {
        target: searchCanvas.games
        function onSearchChanged() { typist.setText(searchCanvas.games.search, searchCanvas.games.search.length) }
    }
    Connections {
        target: searchCanvas.host
        function onSearchingChanged() { typist.setText(searchCanvas.games.search, searchCanvas.games.search.length) }
    }
    Kit.Frame {
        id: searchFrame
        anchors.fill: parent
        surfaceDrawn: false
        padFamily: searchCanvas.padFamily
        helpFont: Qt.font({ family: searchFaces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        // Its help stands while the keyboard holds the focus; on the covers the theme's own does.
        helpShown: searchCanvas.screens.shown === Screens.Search
        focus: !searchCanvas.splashShown && searchCanvas.screens.shown === Screens.Search
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.Search
        }
        // The keyboard's own actions, Y Space and X Done, and the screen's, the shoulders to the covers; its close is the screen's way out.
        Connections {
            target: searchFrame.controller
            function onActionInvoked(key) { typist.act(key) }
        }
        Connections {
            target: searchFrame.controller
            function onActionInvoked(key) { searchCanvas.screens.act(key) }
        }
        Item {
            x: -searchFrame.safeX
            y: -searchFrame.safeY
            width: searchCanvas.width
            height: searchCanvas.height
            // The search's field over the one row of covers, never focused: the keyboard types into it (LunaP's GallerySearch).
            TextField {
                objectName: "GallerySearch"
                x: 64
                y: 100
                width: 1152
                height: 44
                focusPolicy: Qt.NoFocus
                activeFocusOnTab: false
                readOnly: true
                leftPadding: 16
                rightPadding: 16
                text: typist.text
                cursorPosition: typist.caret
                color: Kit.Tokens.textStrong
                font.family: searchFaces.regular
                font.pixelSize: 22
                Accessible.name: Words.search
            }
            Kit.Keyboard {
                id: keyboard
                objectName: "GalleryKeyboard"
                model: typist
                y: 398
                width: searchCanvas.width
                focus: true
                labelFont: Qt.font({ family: searchFaces.regular, pixelSize: Kit.Tokens.fontBody })
                onActiveFocusChanged: if (searchCanvas.host.searching && keyboard.activeFocus !== searchCanvas.host.typing) searchCanvas.screens.keysFocused(keyboard.activeFocus)
            }
        }
    }
    Faces {
        id: searchFaces
    }
}
