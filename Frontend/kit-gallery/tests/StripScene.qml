import QtQuick
import EmuSen.Kit as Kit

// A section strip with more names than its width holds, and one whose names fit on a page that scrolls, for strip.rs (EmuSen_Kit.md §7.43, §7.105).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 240
    color: Kit.Tokens.voidFill
    Kit.FrameController {
        id: controller
        sections: ["All", "NES", "Super Nintendo", "Nintendo 64", "Game Boy", "Game Boy Color", "Game Boy Advance", "Genesis", "Sega CD", "Master System", "Saturn", "PlayStation", "PlayStation 2"]
        selectedSection: "All"
    }
    Kit.SectionStrip {
        objectName: "strip"
        x: 16
        y: 16
        width: 600
        height: 44
        controller: controller
    }
    Kit.FrameController {
        id: few
        sections: ["All", "NES", "Saturn"]
        selectedSection: "All"
    }
    Flickable {
        objectName: "page"
        x: 16
        y: 80
        width: 600
        height: 120
        clip: true
        contentWidth: 600
        contentHeight: 480
        Kit.SectionStrip {
            objectName: "fits"
            width: 600
            height: 44
            controller: few
        }
    }
    readonly property string chosen: few.selectedSection
}
