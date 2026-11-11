import QtQuick
import EmuSen.Kit as Kit

// A section strip with more names than its width holds, for strip.rs (EmuSen_Kit.md §7.43).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 120
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
}
