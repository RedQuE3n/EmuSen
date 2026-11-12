pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The gallery's screens: the tool screen with a tool's rows, and the page frame with tiles and with rows, the gate run on each at the four sizes (EmuSen_Kit.md §7.43, §7.44).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 1280
    height: 800
    color: Kit.Tokens.voidFill
    // 0 the tool screen; 1 the page with tiles; 2 the page with rows.
    property int shown: 0
    property int padFamily: Kit.Families.Generic
    property int activated: -1
    property int applied: 0
    Kit.NormalizedCanvas {
        id: canvas
        objectName: "canvas"
        Kit.ToolScreen {
            id: tool
            objectName: "tool"
            anchors.fill: parent
            visible: scene.shown === 0
            focus: scene.shown === 0
            padFamily: scene.padFamily
            controller.sections: ["All", "Super Nintendo", "PlayStation", "PlayStation 2"]
            controller.selectedSection: "PlayStation 2"
            title: "Cores"
            nameRole: "core"
            searchHint: "Search"
            facts: ListModel {
                ListElement { words: "9 cores"; tone: 0 }
                ListElement { words: "7 installed"; tone: 0 }
                ListElement { words: "1 to approve"; tone: 5 }
                ListElement { words: "catalogue read 31 Oct 2026"; tone: 1 }
            }
            columns: ListModel {
                ListElement { key: "core"; fill: 1; least: 120; trailing: false; toneKey: ""; shortKey: "" }
                ListElement { key: "console"; fill: 0; least: 0; trailing: false; toneKey: ""; shortKey: "consoleShort" }
                ListElement { key: "status"; fill: 0; least: 0; trailing: true; toneKey: "statusTone"; shortKey: "" }
            }
            rows: ListModel {
                ListElement { core: "PCEE2"; console: "PlayStation 2"; consoleShort: "PlayStation 2"; status: "Default"; statusTone: 0 }
                ListElement { core: "Sapphire"; console: "PlayStation 2"; consoleShort: "PlayStation 2"; status: "Built in"; statusTone: 1 }
                ListElement { core: "Genesis Plus GX"; console: "Genesis, Sega CD, Master System, SG-1000"; consoleShort: "Genesis +3"; status: "Installed"; statusTone: 4 }
                ListElement { core: "ares"; console: "Atari 2600, ColecoVision, Game Boy, Game Boy Color, Game Boy Advance, Game Gear, Genesis, Master System, 32X, Sega CD, MSX, Neo Geo Pocket, NES, Nintendo 64, PC Engine, PlayStation, SG-1000, Super Nintendo, WonderSwan, WonderSwan Color"; consoleShort: "Atari 2600 +19"; status: "Available"; statusTone: 1 }
                ListElement { core: "SameBoy"; console: "Game Boy, Game Boy Color"; consoleShort: "Game Boy +1"; status: "Available"; statusTone: 1 }
                ListElement { core: "Extra PS2 Core"; console: "PlayStation 2"; consoleShort: "PlayStation 2"; status: "To approve"; statusTone: 5 }
                ListElement { core: "LRPS2"; console: "PlayStation 2"; consoleShort: "PlayStation 2"; status: "Available"; statusTone: 1 }
                ListElement { core: "Diamond"; console: "PlayStation"; consoleShort: "PlayStation"; status: "Default"; statusTone: 0 }
                ListElement { core: "Beetle PSX HW"; console: "PlayStation"; consoleShort: "PlayStation"; status: "Installed"; statusTone: 4 }
                ListElement { core: "SwanStation"; console: "PlayStation"; consoleShort: "PlayStation"; status: "Update"; statusTone: 5 }
                ListElement { core: "Venus"; console: "Super Nintendo"; consoleShort: "Super Nintendo"; status: "Default"; statusTone: 0 }
                ListElement { core: "Snes9x"; console: "Super Nintendo"; consoleShort: "Super Nintendo"; status: "install"; statusTone: 3 }
                ListElement { core: "bsnes"; console: "Super Nintendo"; consoleShort: "Super Nintendo"; status: "Available"; statusTone: 1 }
            }
            // The core of twenty consoles, its pane's caption naming every one (EmuSen_Kit.md §7.106).
            paneTitle: "ares"
            paneCaption: "From libretro's buildbot · Atari 2600, ColecoVision, Game Boy, Game Boy Color, Game Boy Advance, Game Gear, Genesis, Master System, 32X, Sega CD, MSX, Neo Geo Pocket, NES, Nintendo 64, PC Engine, PlayStation, SG-1000, Super Nintendo, WonderSwan, WonderSwan Color"
            sentences: ListModel {
                ListElement { words: "Runs the games of twenty consoles through EmuSen's libretro host, in a sandbox of its own."; tone: 0 }
                ListElement { words: "By the ares team, under the ISC licence. Nightly build of 31 October 2026."; tone: 0 }
                ListElement { words: "Not tested by EmuSen."; tone: 1 }
            }
            summary: "1 marked: install Snes9x"
            applyLabel: "Apply…"
            applyEnabled: true
            meter: 0.3
            meterWords: "/Cores 51 MB · 410 GB free"
            help: ListModel {
                ListElement { button: 4; label: "Install" }
                ListElement { button: 5; label: "Back" }
                ListElement { button: 6; label: "Apply" }
            }
            onActivated: (index) => scene.activated = index
            onApplied: scene.applied += 1
        }
        Kit.PageFrame {
            id: page
            objectName: "page"
            anchors.fill: parent
            visible: scene.shown !== 0
            focus: scene.shown !== 0
            padFamily: scene.padFamily
            showsTiles: scene.shown === 1
            controller.sections: ["Overview", "States", "Options", "Media", "Details"]
            controller.selectedSection: scene.shown === 1 ? "States" : "Options"
            title: "Cobalt Harbor"
            facts: "Super Nintendo · 1994 · Lantern Works · Never played"
            coverName: "Cobalt Harbor, cover"
            actionModel: ListModel {
                ListElement { label: "Play"; value: ""; button: 4; filled: true; stepped: false; stepLeft: false; stepRight: false }
                ListElement { label: "Resume"; value: "Auto · 12 min ago"; button: -1; filled: false; stepped: true; stepLeft: false; stepRight: false }
                ListElement { label: "Favourite"; value: ""; button: 7; filled: false; stepped: false; stepLeft: false; stepRight: false }
            }
            tileModel: ListModel {
                ListElement { label: "Slot 1"; line: "2 h ago"; picture: "" }
                ListElement { label: "Slot 2"; line: "Yesterday"; picture: "" }
                ListElement { label: "Slot 3"; line: "3 days ago"; picture: "" }
                ListElement { label: "Auto"; line: "12 min ago"; picture: "" }
                ListElement { label: "Slot 4"; line: "Empty"; picture: "" }
            }
            rowModel: ListModel {
                ListElement { label: "Engine"; value: "Venus"; steps: true; opens: false }
                ListElement { label: "Cheats"; value: "1 of 3 on"; steps: false; opens: true }
                ListElement { label: "Display crop"; value: "Console's"; steps: false; opens: true }
                ListElement { label: "Television standard"; value: "The game's"; steps: true; opens: false }
                ListElement { label: "Controller profile"; value: "Default"; steps: true; opens: false }
            }
            help: ListModel {
                ListElement { button: 4; label: "Play" }
                ListElement { button: 7; label: "Favourite" }
                ListElement { button: 5; label: "Back" }
            }
            onActed: (index) => scene.activated = index
        }
    }
}
