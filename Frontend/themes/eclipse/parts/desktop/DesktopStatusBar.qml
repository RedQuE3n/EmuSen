pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import EmuSen.Theme

// Bottom status bar shared across Eclipse Desktop workspace views.
Rectangle {
    id: root
    required property real px

    objectName: "eclipseDesktopStatusBar"
    Layout.fillWidth: true
    Layout.preferredHeight: Math.round(28 * root.px)
    color: ink.glassStrong
    border.width: 1
    border.color: ink.hairline

    DesktopInk {
        id: ink
    }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Math.round(14 * root.px)
        anchors.rightMargin: Math.round(14 * root.px)
        spacing: Math.round(12 * root.px)

        Text {
            objectName: "eclipseDesktopStatusText"
            Layout.fillWidth: true
            elide: Text.ElideRight
            text: Workspace.statusText.length > 0 ? Workspace.statusText : Workspace.summaryLine
            color: ink.text
            font.family: Tokens.faceInterface
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
        }

        Text {
            visible: root.width >= 1360
            text: Workspace.statusHints
            color: ink.dim
            font.family: Tokens.faceInterface
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
        }

        Text {
            visible: Device.battery >= 0
            text: Words.percent(Device.battery)
            color: ink.dim
            font.family: Tokens.faceCode
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
        }

        Text {
            visible: Device.shortTime.length > 0
            text: Device.shortTime
            color: ink.text
            font.family: Tokens.faceCode
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
        }

        Text {
            objectName: "eclipseDesktopStatusSummary"
            text: Workspace.statusSummary
            color: ink.muted
            font.family: Tokens.faceInterface
            font.pixelSize: Math.max(10, Math.round(11 * root.px))
        }
    }
}
