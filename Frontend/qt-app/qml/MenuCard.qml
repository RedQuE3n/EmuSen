pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Models

// Card host: reference layout when coronaRim is 0, shared Eclipse Cards frame when coronaRim > 0 (R-20261113-5, R-20261114-21).
FocusScope {
    id: root
    objectName: "PadCard"
    property var rows
    property string title
    property string breadcrumb
    property string subtitle
    property string footer
    property string cover
    property int current: 0
    property var paneLines
    property var paneMuted
    property string panePicture
    property string paneCaption
    property string paneCore
    property int panePlanet: -3
    property string paneStatus
    property bool paneStatusOk: false
    property real paneHeight: 0
    property color scrimColor: Kit.Tokens.scrim
    property bool menu: false
    property bool hasContinue: false
    property string continueTitle
    property string continueSubtitle
    property string continuePicture
    property string positionLine
    property int battery: -1
    property string clockText
    property real pixelScale: 1
    default property alias paneContent: paneArea.data
    property bool paneDrawn: false
    property HelpModel help
    property int padFamily: Kit.Families.Generic
    property bool open: false
    property bool eclipse: false
    property real slide: 1
    readonly property bool leaving: !root.open && root.slide < 1
    readonly property alias controller: frame.controller
    readonly property MenuCardReference refView: refLoader.item as MenuCardReference
    readonly property MenuCardEclipse eclipseView: eclipseLoader.item as MenuCardEclipse
    signal chosen(int index)
    signal stepped(int index, int by, bool held)
    signal focused(int index)
    signal backed()
    signal acted(string key)
    signal keyed(int key)
    signal gone()
    Accessible.role: Accessible.Pane
    Accessible.name: root.title
    onOpenChanged: {
        if (root.open) {
            exit.stop()
            enter.restart()
        } else {
            enter.stop()
            exit.restart()
        }
    }
    NumberAnimation {
        id: enter
        target: root
        property: "slide"
        to: 0
        duration: Kit.Tokens.enter
        easing.type: Kit.Tokens.enterCurve
    }
    NumberAnimation {
        id: exit
        target: root
        property: "slide"
        to: 1
        duration: Kit.Tokens.exit
        easing.type: Kit.Tokens.exitCurve
        onFinished: root.gone()
    }
    Kit.Frame {
        id: frame
        width: root.width
        height: root.height
        surfaceDrawn: false
        helpShown: false
        padFamily: root.padFamily
        focus: true
        controller.navigationShown: false
        help: root.help
        Connections {
            target: frame.controller
            function onBackDeclined() { root.backed() }
            function onActionInvoked(key) { root.acted(key) }
        }
        Loader {
            id: refLoader
            active: !root.eclipse
            focus: !root.eclipse
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            sourceComponent: MenuCardReference {
                focus: true
                controller: frame.controller
                rows: root.rows
                title: root.title
                breadcrumb: root.breadcrumb
                footer: root.footer
                cover: root.cover
                current: root.current
                paneLines: root.paneLines
                paneMuted: root.paneMuted
                panePicture: root.panePicture
                paneCaption: root.paneCaption
                paneDrawn: root.paneDrawn
                help: root.help
                padFamily: root.padFamily
                slide: root.slide
                onChosen: (index) => root.chosen(index)
                onStepped: (index, by, held) => root.stepped(index, by, held)
                onFocused: (index) => root.focused(index)
                onKeyed: (key) => root.keyed(key)
            }
        }
        Loader {
            id: eclipseLoader
            active: root.eclipse
            focus: root.eclipse
            x: -frame.safeX
            y: -frame.safeY
            width: root.width
            height: root.height
            sourceComponent: MenuCardEclipse {
                focus: true
                controller: frame.controller
                rows: root.rows
                title: root.title
                breadcrumb: root.breadcrumb
                subtitle: root.subtitle
                footer: root.footer
                current: root.current
                paneLines: root.paneLines
                paneMuted: root.paneMuted
                panePicture: root.panePicture
                paneCaption: root.paneCaption
                paneCore: root.paneCore
                panePlanet: root.panePlanet
                paneStatus: root.paneStatus
                paneStatusOk: root.paneStatusOk
                paneDrawn: root.paneDrawn
                paneHeight: root.paneHeight
                scrimColor: root.scrimColor
                menu: root.menu
                hasContinue: root.hasContinue
                continueTitle: root.continueTitle
                continueSubtitle: root.continueSubtitle
                continuePicture: root.continuePicture
                positionLine: root.positionLine
                battery: root.battery
                clockText: root.clockText
                help: root.help
                padFamily: root.padFamily
                slide: root.slide
                pixelScale: root.pixelScale
                onChosen: (index) => root.chosen(index)
                onStepped: (index, by, held) => root.stepped(index, by, held)
                onFocused: (index) => root.focused(index)
                onKeyed: (key) => root.keyed(key)
            }
        }
    }
    Item {
        id: paneArea
        parent: root.eclipseView ? root.eclipseView.paneSlot : (root.refView ? root.refView.paneSlot : root)
        anchors.fill: parent
    }
}
