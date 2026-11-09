import QtQuick
import EmuSen.Theme
import EmuSen.Kit as Kit

// A theme's view of the module: every list read through its roles by name, the words, the tokens, a screen and each re-exported type placed.
Window {
    id: scene
    objectName: "scene"
    width: 1280
    height: 800
    color: Tokens.voidFill
    component ConsoleRow: Item {
        required property string key
        required property string name
        required property string displayName
        required property string maker
        required property int year
        required property int count
        required property real hue
        required property string body
        required property string engineName
        required property string engineKind
        required property string engineMark
        required property bool libretro
        required property int group
        required property int groupIndex
        required property int groupSize
        required property int groupStart
    }
    component PlaceRow: Item {
        required property string key
        required property string name
        required property string kind
        required property string line
        required property bool attention
        required property int count
        required property bool available
    }
    component GameRow: Item {
        required property string key
        required property string title
        required property string cover
        required property real coverShape
        required property bool favourite
        required property real lastPlayed
        required property bool played
        required property string consoleId
        required property real consoleHue
        required property int players
        required property int year
        required property string genre
        required property real rating
        required property string description
        required property string video
        required property string hero
        required property string lastPlayedWords
        required property string facts
    }
    readonly property int consoles: Library.consoles.count
    readonly property int places: Library.places.count
    readonly property int shelves: Library.shelves.count
    // The console whose games the scene lists, named by the test as a theme takes it from a row.
    property string consoleKey
    readonly property int consoleGames: Library.gamesOf(scene.consoleKey).count
    readonly property string wordmark: Words.title
    readonly property string counted: Words.games(2)
    readonly property string status: Words.statusLine(Device.controllers, Device.clock)
    readonly property string sessionTitle: Session.title
    readonly property string sessionKey: Session.key
    readonly property string sessionConsole: Session.console
    readonly property int shelf: Library.shelf.count
    readonly property string summary: Words.summary(2, 3)
    readonly property string browseHeading: Browse.heading
    readonly property string browseNext: Browse.next
    readonly property int browseGames: Browse.games.count
    readonly property int focusRow: Browse.focusRow
    readonly property bool marked: Marks.heart.length > 0 && Marks.path(Library.shelves.count > 0 ? "all" : "").length > 0
    readonly property int pageUp: PadGlyph.PageUp
    readonly property bool hasGame: Session.hasGame
    readonly property int gameHelp: gameHelpList.count
    readonly property string gameActions: Help.entries(Help.Section).actionLines
    readonly property color accent: Tokens.accent
    readonly property color kitAccent: Kit.Tokens.accent
    readonly property int dark: Tokens.Dark
    readonly property int screenRole: screen.role
    readonly property int screenFocus: screen.focusRole
    readonly property string firstConsole: (consoleList.count > 0 ? consoleList.itemAt(0) as ConsoleRow : null)?.key ?? ""
    // Home's places stand before the feed, so their probe is read again when the test names a console after the feed.
    readonly property string secondPlace: (scene.consoleKey.length >= 0 && placeList.count > 1 ? placeList.itemAt(1) as PlaceRow : null)?.name ?? ""
    readonly property bool secondAttention: (scene.consoleKey.length >= 0 && placeList.count > 1 ? placeList.itemAt(1) as PlaceRow : null)?.attention ?? false
    readonly property int firstGroupSize: (consoleList.count > 0 ? consoleList.itemAt(0) as ConsoleRow : null)?.groupSize ?? 0
    readonly property bool secondEnabled: (scene.consoleKey.length >= 0 && placeList.count > 1 ? placeList.itemAt(1) as PlaceRow : null)?.available ?? false
    readonly property int placeHelp: placeHelpList.count
    readonly property string placeGroup: (placeHelpList.count > 2 ? placeHelpList.itemAt(2) as HelpRow : null)?.label ?? ""
    readonly property string emptyWords: [Words.noGames, Words.noRecent, Words.noConsoles, Words.addFolder, Words.places, Words.continueEntry, Words.recentEntry].join("|")
    readonly property string paused: Words.pausedAgo(Session.pausedSince)
    property bool listEnabled: Actions.enabled("list")
    readonly property int rungs: Bells.ladder(3)
    readonly property string firstTitle: (gameList.count > 0 ? gameList.itemAt(0) as GameRow : null)?.title ?? ""
    component HelpRow: Item {
        required property int button
        required property string label
        required property int pair
    }
    Repeater {
        id: placeHelpList
        model: Help.entries(Help.Place, {group: true, back: true})
        delegate: HelpRow {}
    }
    Repeater {
        id: gameHelpList
        model: Help.entries(Help.Game)
        Item {
            required property int button
            required property string label
            required property int pair
        }
    }
    Repeater {
        id: consoleList
        model: Library.consoles
        delegate: ConsoleRow {}
    }
    Repeater {
        id: placeList
        model: Library.places
        delegate: PlaceRow {}
    }
    Repeater {
        model: Library.shelves
        Item {
            required property string key
            required property string name
            required property string line
            required property int count
        }
    }
    Repeater {
        id: gameList
        model: Library.gamesOf(scene.consoleKey)
        delegate: GameRow {}
    }
    Repeater {
        model: Browse.games
        delegate: GameRow {}
    }
    Repeater {
        model: Library.collections
        Item {
            required property string key
            required property string name
            required property int count
        }
    }
    ThemeScreen {
        id: screen
        objectName: "screen"
        anchors.fill: parent
        role: ThemeScreen.Games
        focusRole: ThemeScreen.Navigation
        focusItems: [walkable]
        Frame {
            id: frame
            anchors.fill: parent
            help: Help.entries(Help.Section)
            padFamily: Device.family
            Walkable {
                id: walkable
                objectName: "walkable"
                controller: frame.controller
                width: 120
                height: 160
            }
            Carousel {
                objectName: "carousel"
                controller: frame.controller
                y: 200
                width: 600
                height: 200
            }
        }
        Sky {
            objectName: "sky"
            anchors.fill: parent
            z: -1
        }
        Scene {
            objectName: "ownColours"
            Body {
                objectName: "body"
                width: 72
                height: 72
                hue: 200
            }
        }
        NormalizedCanvas {
            objectName: "canvas"
            width: 320
            height: 200
        }
        CoverTile {
            objectName: "coverTile"
            width: 120
            height: 160
            title: Session.title
            mark: Marks.heart
        }
        RoundPicture {
            objectName: "roundPicture"
            width: 40
            height: 40
        }
        PadGlyph {
            objectName: "padGlyph"
            button: PadGlyph.PageUp
        }
        PlanetLayout {
            objectName: "planets"
            width: 1280
            leading: 3
            bodies: 2
        }
        HelpBar {
            objectName: "helpBar"
            model: Help.entries(Help.Place)
            width: 600
        }
    }
}
