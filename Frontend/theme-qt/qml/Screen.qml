import QtQuick

// The root of a theme's screen (EmuSen_Qt.md §3.2): which screen it is, the walk's first role, the items a free arrangement walks in order, and for the splash its length, whether it is drawn still, and its end.
FocusScope {
    enum Kind { Home, Consoles, Games, Splash }
    enum Walk { Navigation, Content, Detail, Actions, Search }
    // Home and Content by number: inside this file QtQuick's attached Screen shadows the type's own name.
    property int role: 0
    property int focusRole: 1
    property list<Item> focusItems
    property int length: 0
    property bool still: false
    signal finished()
}
