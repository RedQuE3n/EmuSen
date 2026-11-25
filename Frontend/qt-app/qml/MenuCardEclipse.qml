pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit
import EmuSen.Models

// Shared Eclipse Cards frame: left corona disc, Menu or standard header + nav, detail card, status pill, and footer bar (boards #5, #44-#46, Pause Card).
FocusScope {
    id: root
    required property var controller
    required property var rows
    property string title
    property string breadcrumb
    property string footer
    property int current: 0
    property var paneLines
    property var paneMuted
    property string panePicture
    property string paneCaption
    property string paneCore
    property int panePlanet: -3
    property string paneStatus
    property bool paneStatusOk: false
    property bool paneDrawn: false
    property bool menu: false
    property bool hasContinue: false
    property string continueTitle
    property string continueSubtitle
    property string continuePicture
    property string positionLine
    property int battery: -1
    property string clockText
    property HelpModel help
    property int padFamily: Kit.Families.Xbox
    property real slide: 0
    property real pixelScale: 1
    readonly property real boardHeight: 1200
    readonly property real unit: Kit.CanvasUnits.designHeight / root.boardHeight
    readonly property bool paneShown: !root.menu && ((root.paneLines?.length ?? 0) > 0 || root.paneCore.length > 0 || root.paneStatus.length > 0 || root.panePicture.length > 0 || root.paneCaption.length > 0 || root.paneDrawn)
    readonly property alias paneSlot: paneArea
    readonly property alias list: list
    signal chosen(int index)
    signal stepped(int index, int by, bool held)
    signal focused(int index)
    signal keyed(int key)
    onCurrentChanged: if (list.currentIndex !== root.current) list.currentIndex = root.current
    Rectangle {
        objectName: "AdaptiveCardScrim"
        anchors.fill: parent
        color: Kit.Tokens.scrim
    }
    Item {
        id: leftSlide
        x: -(760 * root.unit) * root.slide
        width: root.width
        height: root.height
        Kit.CoronaDisc {
            id: disc
            objectName: "cardDisc"
            x: (root.menu ? -2440 : -2400) * root.unit
            y: -960 * root.unit
            width: 3120 * root.unit
            height: 3120 * root.unit
            fill: Kit.Tokens.surface
            pixelScale: root.pixelScale
        }
        Item {
            id: section
            objectName: "cardSection"
            x: 72 * root.unit
            y: (root.menu ? 72 : 64) * root.unit
            width: (root.menu ? 520 : 560) * root.unit
            height: root.height - (root.menu ? 144 : 174) * root.unit
            Item {
                id: menuHeader
                objectName: "menuHeader"
                visible: root.menu
                x: 0
                y: 0
                width: section.width
                height: 43 * root.unit
                Item {
                    id: menuMark
                    objectName: "menuMark"
                    readonly property color ink: Kit.Tokens.accent
                    x: 0
                    y: (menuHeader.height - height) / 2
                    width: 34 * root.unit
                    height: 34 * root.unit
                    Accessible.role: Accessible.Graphic
                    Accessible.name: Words.name
                    Shape {
                        width: 34
                        height: 34
                        scale: parent.width / 34
                        transformOrigin: Item.TopLeft
                        preferredRendererType: Shape.CurveRenderer
                        ShapePath {
                            fillColor: menuMark.ink
                            strokeColor: "transparent"
                            fillRule: ShapePath.OddEvenFill
                            PathSvg { path: Marks.menuCrescent }
                        }
                    }
                }
                CardLine {
                    objectName: "cardTitle"
                    visible: root.menu
                    unit: root.unit
                    boardLine: true
                    x: 48 * root.unit
                    y: 0
                    text: root.title
                    color: Kit.Tokens.textStrong
                    size: 34
                    weight: Font.DemiBold
                }
                CardLine {
                    objectName: "menuStatus"
                    visible: root.menu
                    unit: root.unit
                    boardLine: true
                    x: menuHeader.width - width
                    y: (menuHeader.height - height) / 2
                    text: Words.statusClock(root.battery, root.clockText)
                    color: "#D8DBE6"
                    size: 22
                    weight: Font.Normal
                }
            }
            Column {
                id: standardHeader
                objectName: "cardHeader"
                visible: !root.menu
                x: 0
                y: 0
                width: section.width
                spacing: 8 * root.unit
                CardLine {
                    objectName: "cardBreadcrumb"
                    visible: !root.menu && root.breadcrumb.length > 0
                    unit: root.unit
                    boardLine: true
                    width: parent.width
                    text: root.breadcrumb
                    capitalization: Font.AllUppercase
                    color: "#F3C9D2"
                    size: 18
                    weight: Font.DemiBold
                    spacing: 0.22 * 18
                }
                CardLine {
                    objectName: "cardTitle"
                    visible: !root.menu
                    unit: root.unit
                    boardLine: true
                    boardLineHeight: 53
                    width: parent.width
                    text: root.title
                    color: Kit.Tokens.textStrong
                    size: 50
                    weight: Font.DemiBold
                    spacing: -0.01 * 50
                    wrapMode: Text.WordWrap
                    lineHeight: 53
                    lineHeightMode: Text.FixedHeight
                }
            }
            Item {
                id: rowsArea
                objectName: "AdaptiveCardRows"
                x: 0
                y: root.menu ? menuHeader.height + 34 * root.unit : standardHeader.height + 26 * root.unit
                width: section.width
                height: section.height - y - (root.menu ? 56 * root.unit : 0)
                ListView {
                    id: list
                    objectName: "PadMenuList"
                    anchors.fill: parent
                    model: root.rows
                    focus: true
                    interactive: false
                    clip: list.contentHeight > list.height
                    spacing: 6 * root.unit
                    keyNavigationEnabled: true
                    keyNavigationWraps: false
                    highlightMoveDuration: Kit.Tokens.instant
                    highlightResizeDuration: Kit.Tokens.instant
                    Accessible.role: Accessible.List
                    onCurrentIndexChanged: if (list.currentIndex !== root.current) root.focused(list.currentIndex)
                    delegate: MenuRowEclipse {
                        id: row
                        required property int index
                        required property var model
                        objectName: "cardRow." + row.index
                        width: ListView.view.width
                        unit: root.unit
                        pixelScale: root.pixelScale
                        rowIndex: row.index
                        padFamily: root.padFamily
                        controller: root.controller
                        menu: root.menu
                        continueCard: root.menu && root.hasContinue && row.index === 0
                        continueTitle: root.continueTitle
                        continueSubtitle: root.continueSubtitle
                        continuePicture: root.continuePicture
                        label: row.model.label
                        value: row.model.value
                        icon: row.model.icon ?? ""
                        steps: row.model.steps ?? false
                        opens: row.model.opens ?? false
                        sectionEnd: row.model.sectionEnd ?? false
                        current: ListView.isCurrentItem
                        focus: ListView.isCurrentItem
                        onActivated: root.chosen(row.index)
                        onActiveFocusChanged: list.currentIndex = row.activeFocus ? row.index : list.currentIndex
                        Keys.onLeftPressed: (event) => root.stepped(row.index, -1, event.isAutoRepeat)
                        Keys.onRightPressed: (event) => root.stepped(row.index, 1, event.isAutoRepeat)
                        Keys.onPressed: (event) => root.keyed(event.key)
                    }
                }
            }
            CardLine {
                objectName: "cardFooter"
                visible: root.footer.length > 0
                unit: root.unit
                boardLine: true
                x: 0
                y: rowsArea.y + list.contentHeight + 20 * root.unit
                width: section.width
                text: root.footer
                color: Kit.Tokens.muted
                size: 19
                wrapMode: Text.WordWrap
            }
            Kit.HelpBar {
                id: menuHelp
                objectName: "cardHelp"
                visible: root.menu
                x: 0
                y: section.height - height
                controller: root.controller
                model: root.help
                family: root.padFamily
                fitted: true
                ruled: false
                color: "transparent"
                labelColor: "#D8DBE6"
                iconColor: Kit.Tokens.muted
                leadStyle: Kit.GlyphStyles.Filled
                leadColor: Kit.Tokens.accent
                entrySpacing: 34 * root.unit
                glyphGap: 10 * root.unit
                glyphSize: 34 * root.unit
                snapped: false
                label: Component {
                    CardLine {
                        objectName: "helpWords-" + text
                        unit: root.unit
                        boardLine: true
                        size: 21
                    }
                }
                glyph: Component {
                    CardGlyph {
                        unit: root.unit
                        side: 34
                        ink: "#D8DBE6"
                    }
                }
            }
        }
    }
    Rectangle {
        id: detailCard
        objectName: "AdaptiveCardPane"
        visible: root.paneShown
        opacity: 1 - root.slide
        x: 800 * root.unit
        y: 210 * root.unit
        width: 760 * root.unit
        height: root.paneDrawn ? Math.max(detailColumn.implicitHeight + 72 * root.unit, paneArea.implicitHeight + 72 * root.unit) : detailColumn.implicitHeight + 72 * root.unit
        radius: 28 * root.unit
        color: Qt.alpha(Kit.Tokens.surface, 0.95)
        border.width: 1 * root.unit
        border.color: "#262A3A"
        Column {
            id: detailColumn
            objectName: "AdaptiveDetail"
            x: 36 * root.unit
            y: 36 * root.unit
            width: detailCard.width - 72 * root.unit
            spacing: 22 * root.unit
            Item {
                id: coreRow
                objectName: "paneCoreRow"
                visible: root.paneCore.length > 0
                width: parent.width
                height: 25 * root.unit
                CardPlanet {
                    objectName: "panePlanet"
                    unit: root.unit
                    planet: root.panePlanet
                    side: 22
                    glowBlur: 10
                    glowSpread: 2
                    glowAlpha: 0.55
                    pixelScale: root.pixelScale
                    x: 0
                    y: (coreRow.height - height) / 2
                }
                CardLine {
                    objectName: "paneCoreName"
                    unit: root.unit
                    boardLine: true
                    x: 34 * root.unit
                    y: (coreRow.height - height) / 2
                    text: root.paneCore
                    color: "#D8DBE6"
                    size: 20
                    weight: Font.Normal
                }
            }
            Image {
                objectName: "PanePicture"
                width: parent.width
                height: 444 * root.unit
                visible: root.panePicture.length > 0
                source: root.panePicture
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
            CardLine {
                objectName: "PaneCaption"
                width: parent.width
                visible: root.paneCaption.length > 0
                unit: root.unit
                boardLine: true
                text: root.paneCaption
                color: Kit.Tokens.muted
                size: 19
                elide: Text.ElideRight
            }
            Repeater {
                model: root.paneLines
                Column {
                    id: said
                    required property int index
                    required property string modelData
                    width: detailColumn.width
                    spacing: 22 * root.unit
                    CardLine {
                        objectName: "paneLead." + said.index
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 34.8
                        width: parent.width
                        visible: said.modelData.length > 0
                        text: said.modelData
                        color: Kit.Tokens.textStrong
                        size: 24
                        weight: Font.Normal
                        wrapMode: Text.WordWrap
                        lineHeight: 34.8
                        lineHeightMode: Text.FixedHeight
                    }
                    CardLine {
                        objectName: "paneMuted." + said.index
                        unit: root.unit
                        boardLine: true
                        boardLineHeight: 27.55
                        width: parent.width
                        visible: text.length > 0
                        text: root.paneMuted?.[said.index] ?? ""
                        color: Kit.Tokens.muted
                        size: 19
                        weight: Font.Normal
                        wrapMode: Text.WordWrap
                        lineHeight: 27.55
                        lineHeightMode: Text.FixedHeight
                    }
                }
            }
            CardLine {
                objectName: "paneStatus"
                visible: root.paneStatus.length > 0
                unit: root.unit
                boardLine: true
                width: parent.width
                text: root.paneStatus
                color: root.paneStatusOk ? "#C8F0DA" : "#F3C9D2"
                size: 20
                weight: Font.Normal
                wrapMode: Text.WordWrap
            }
        }
        Item {
            id: paneArea
            anchors.fill: parent
        }
    }
    Rectangle {
        id: statusPill
        objectName: "cardStatus"
        visible: !root.menu && (root.clockText.length > 0 || root.battery >= 0)
        opacity: 1 - root.slide
        readonly property string chargeText: Words.percent(root.battery)
        y: 48 * root.unit
        height: 56 * root.unit
        width: statusClock.x + statusClock.width + 22 * root.unit
        x: root.width - 200 * root.unit - width
        radius: 28 * root.unit
        color: Qt.alpha(Kit.Tokens.surface, 0.82)
        CardLine {
            id: statusBattery
            objectName: "cardBattery"
            visible: statusPill.chargeText.length > 0
            unit: root.unit
            boardLine: true
            x: 22 * root.unit
            y: (statusPill.height - height) / 2
            text: statusPill.chargeText
            color: Kit.Tokens.textStrong
            size: 22
            weight: Font.Normal
        }
        CardLine {
            id: statusClock
            objectName: "cardClock"
            unit: root.unit
            boardLine: true
            x: statusBattery.visible ? statusBattery.x + statusBattery.width + 22 * root.unit : 22 * root.unit
            y: (statusPill.height - height) / 2
            text: root.clockText
            color: Kit.Tokens.textStrong
            size: 22
            weight: Font.DemiBold
        }
    }
    Rectangle {
        id: footerBar
        objectName: "cardFooterBar"
        visible: !root.menu
        x: 0
        y: root.height - height + height * root.slide
        width: root.width
        height: 76 * root.unit
        color: Kit.Tokens.surface
        Rectangle {
            objectName: "cardFooterRule"
            x: 0
            y: 0
            width: parent.width
            height: 2 * root.unit
            color: Kit.Tokens.rule
        }
        Kit.HelpBar {
            id: barHelp
            objectName: "cardHelp"
            visible: !root.menu
            x: 72 * root.unit
            y: (footerBar.height - height) / 2
            controller: root.controller
            model: root.help
            family: root.padFamily
            fitted: true
            ruled: false
            color: "transparent"
            labelColor: "#D8DBE6"
            iconColor: Kit.Tokens.muted
            leadStyle: Kit.GlyphStyles.Filled
            leadColor: Kit.Tokens.accent
            entrySpacing: 34 * root.unit
            glyphGap: 10 * root.unit
            glyphSize: 34 * root.unit
            snapped: false
            label: Component {
                CardLine {
                    objectName: "helpWords-" + text
                    unit: root.unit
                    boardLine: true
                    size: 21
                }
            }
            glyph: Component {
                CardGlyph {
                    unit: root.unit
                    side: 34
                    ink: "#D8DBE6"
                    boxMenu: true
                }
            }
        }
        CardLine {
            objectName: "cardPosition"
            visible: !root.menu && root.positionLine.length > 0
            unit: root.unit
            boardLine: true
            x: footerBar.width - 72 * root.unit - width
            y: (footerBar.height - height) / 2
            text: root.positionLine
            color: Kit.Tokens.muted
            size: 21
            weight: Font.Normal
        }
    }
}
