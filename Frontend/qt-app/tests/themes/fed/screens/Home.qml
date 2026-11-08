import QtQuick
import EmuSen.Theme

// The feed's test theme: what the program hands a theme's engine, shown as text, and one item that opens Home's Games place.
ThemeScreen {
    id: root
    objectName: "fedHome"
    role: ThemeScreen.Home
    Text { objectName: "fedConsoles"; text: Library.consoles.count }
    Text { objectName: "fedPlaces"; y: 20; text: Library.places.count }
    Text { objectName: "fedClock"; y: 40; text: Device.clock }
    Text { objectName: "fedSession"; y: 60; text: Session.hasGame }
    Text { objectName: "fedMotion"; y: 80; text: Tokens.reducedMotion }
    Frame {
        id: frame
        anchors.fill: parent
        focus: true
        Walkable {
            objectName: "fedGames"
            controller: frame.controller
            focus: true
            width: 100
            height: 100
            onActivated: taken.text = Actions.openPlace("games")
        }
    }
    Text { id: taken; objectName: "fedTaken"; y: 100 }
}
