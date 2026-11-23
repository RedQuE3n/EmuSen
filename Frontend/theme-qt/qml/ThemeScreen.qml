import QtQuick

// The root of a theme's screen (EmuSen_Qt.md §3.2): which screen it is, the walk's first role, the items a free arrangement walks in order, for the splash its length, whether it is drawn still, its end, and from 1.10 whether it enters Home and its sound, and for Home where such a splash is placed.
FocusScope {
    enum Kind { Home, Consoles, Games, Splash, Settings, Game }
    enum Walk { Navigation, Content, Detail, Actions, Search }
    property int role: ThemeScreen.Home
    property int focusRole: ThemeScreen.Content
    property list<Item> focusItems
    property int length: 0
    property bool still: false
    // 1.10: the splash enters Home on the program's clock (Splash.time), with no Moonrise under it (EmuSen_Qt.md §3.2).
    property bool entersHome: false
    // 1.10: the splash's sound, its `source` a WAV file of the theme's own as its screen's file names it, played from the first frame.
    component Sound: QtObject {
        property url source
    }
    readonly property Sound sound: Sound {}
    // 1.10: on Home, the item a splash that enters Home is placed in, between what Home draws under it and over it.
    property Item splashPlace: null
    signal finished()
}
