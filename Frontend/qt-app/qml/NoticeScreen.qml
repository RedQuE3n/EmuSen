import QtQuick
import EmuSen.Kit as Kit

// The session's notices (saved, loaded, the screenshot, what failed), at the top right over everything; over running
// gameplay they are the game's own, so they show (EmuSen_Interface.md §18.6).
Kit.NormalizedCanvas {
    id: noticeCanvas
    objectName: "noticeCanvas"
    required property Kit.NoticeModel notices
    Kit.NoticeLayer {
        model: noticeCanvas.notices
        width: noticeCanvas.width
        height: noticeCanvas.height
        labelFont: Qt.font({ family: noticeFaces.regular, pixelSize: Kit.Tokens.fontBody })
    }
    Faces {
        id: noticeFaces
    }
}
