pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Eclipse's Home as its board draws it (Main.dc.html): the disc, the header, the focused game's name and buttons, the Recent row and the help line, every measure the board's in canvas units (EmuSen_Qt.md §3.4.8).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    // Where the splash is placed while it enters Home: over the backdrop, under the disc and everything Home draws on it (1.10).
    splashPlace: splashPlace
    // Where the focus starts, and where Start on Home takes it back: the first game of Recent, else the search.
    focusItems: [
        root.firstTile ?? status.search
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            objectName: "home"
            anchors.fill: parent
            focus: true
            // The board's three buttons, the entries that act on the game (Main:61-71): a constant of the design.
            readonly property int gameButtons: 3
            // The tile whose game the top shows: the focused one, else the last focused.
            property int current: 0
            property Item firstTile: null
            // The first button, read again whenever its Repeater's count moves.
            readonly property Item firstButton: buttons.count > 0 ? buttons.itemAt(0) : null
            readonly property Item currentTile: tiles.count > 0 ? tiles.itemAt(Math.min(root.current, tiles.count - 1)) : null
            readonly property bool resuming: Session.hasGame && root.currentTile !== null && root.currentTile.key === Session.key
            readonly property int helpKind: root.resuming ? Help.Continue : Help.Game
            Board {
                id: design
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            // Home arriving as the splash's board times it, whole once the splash has gone.
            Encore {
                id: encore
            }
            // The shown game's hues for the board's fields (parts/ArtHues.qml).
            ArtHues {
                id: shownHues
                key: root.currentTile ? root.currentTile.key : ""
                consoleHue: root.currentTile ? root.currentTile.consoleHue : NaN
            }
            readonly property var artHues: shownHues.art
            readonly property var hues: shownHues.hues
            // Each board field's colour with the shown game's hue; the streaks take the hues of the two fields they are pale tints of
            // (Main:22 of 19, Main:23 of 20).
            FieldInk { id: field0; board: ink.field0; hue: root.hues.length > 0 ? root.hues[0] : NaN }
            FieldInk { id: field1; board: ink.field1; hue: root.hues.length > 0 ? root.hues[1] : NaN }
            FieldInk { id: field2; board: ink.field2; hue: root.hues.length > 0 ? root.hues[2] : NaN }
            FieldInk { id: field3; board: ink.field3; hue: root.hues.length > 0 ? root.hues[0] : NaN }
            FieldInk { id: field4; board: ink.field4; hue: root.hues.length > 0 ? root.hues[1] : NaN }
            // The ground and the board's five blurred colour fields, each at its place, size, turn, blur and opacity (Main:17-23).
            Backdrop {
                objectName: "backdrop"
                anchors.fill: parent
                ground: ink.ground
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: 1060 * design.unit; y: -140 * design.unit; width: 900 * design.unit; height: 760 * design.unit; color: field0.color; blur: 90 * design.unit; opacity: 0.85 },
                    BackdropField { x: 1380 * design.unit; y: 420 * design.unit; width: 760 * design.unit; height: 760 * design.unit; color: field1.color; blur: 110 * design.unit; opacity: 0.9 },
                    BackdropField { x: 900 * design.unit; y: -260 * design.unit; width: 700 * design.unit; height: 520 * design.unit; color: field2.color; blur: 100 * design.unit; opacity: 0.8 },
                    BackdropField { x: 1240 * design.unit; y: 250 * design.unit; width: 620 * design.unit; height: 46 * design.unit; corner: 40 * design.unit; color: field3.color; blur: 22 * design.unit; opacity: 0.55 * encore.handed; rotation: -18 },
                    BackdropField { x: 1330 * design.unit; y: 380 * design.unit; width: 520 * design.unit; height: 30 * design.unit; corner: 40 * design.unit; color: field4.color; blur: 18 * design.unit; opacity: 0.5 * encore.handed; rotation: -14 }
                ]
            }
            // The splash, while it enters Home: its ground, stars, planets and the disc that becomes Home's (EclipseSplashPlay.dc.html).
            Item {
                id: splashPlace
                objectName: "splashPlace"
                anchors.fill: parent
            }
            CoronaDisc {
                id: disc
                objectName: "disc"
                // The splash's own disc stands for it until the splash has gone, and the hand-over's until Home is whole.
                visible: !encore.splashing && !encore.easing
                // The shown game's first hue at the look's rim's own saturation and lightness; its glows take it at their own strengths (R-20261114-38, R-20261115-27).
                corona: root.hues.length > 0 ? Qt.hsla((((root.hues[0] % 360) + 360) % 360) / 360, Tokens.coronaColor.hslSaturation, Tokens.coronaColor.hslLightness, 1) : "transparent"
                pixelScale: canvas.unit
                x: -900 * design.unit
                y: -450 * design.unit
                width: 2100 * design.unit
                height: width
            }
            // The hand-over (R-20261123-9): the splash's opened disc, its rim, three glows and ground, eased into Home's disc, which it gives way to once equal.
            SplashCorona {
                id: splashCorona
            }
            Blend { id: rimInk; from: Tokens.coronaColor; to: disc.rimColor; t: encore.handed }
            Blend { id: fillInk; from: ink.splashGround; to: disc.fill; t: encore.handed }
            Blend { id: nearInk; from: ink.coronaNear; to: disc.glowColor; t: encore.handed }
            Blend { id: middleInk; from: ink.coronaMiddle; to: disc.haloColor; t: encore.handed }
            Blend { id: farInk; from: ink.coronaFar; to: Qt.rgba(ink.coronaFar.r, ink.coronaFar.g, ink.coronaFar.b, 0); t: encore.handed }
            CoronaDisc {
                objectName: "handoverDisc"
                visible: encore.easing
                fill: fillInk.value
                corona: rimInk.value
                rim: splashCorona.rim * splashCorona.opened * design.unit * (1 - encore.handed) + disc.rim * encore.handed
                pixelScale: canvas.unit
                x: disc.x
                y: disc.y
                width: disc.width
                height: disc.height
                glows: [
                    CoronaGlow {
                        blur: splashCorona.nearBlur * splashCorona.opened * design.unit * (1 - encore.handed) + disc.glows[0].blur * encore.handed
                        spread: splashCorona.nearSpread * splashCorona.opened * design.unit * (1 - encore.handed) + disc.glows[0].spread * encore.handed
                        color: nearInk.value
                    },
                    CoronaGlow {
                        blur: splashCorona.middleBlur * splashCorona.opened * design.unit * (1 - encore.handed) + disc.glows[1].blur * encore.handed
                        spread: splashCorona.middleSpread * splashCorona.opened * design.unit * (1 - encore.handed) + disc.glows[1].spread * encore.handed
                        color: middleInk.value
                    },
                    CoronaGlow {
                        blur: splashCorona.farBlur * splashCorona.opened * design.unit * (1 - encore.handed)
                        spread: splashCorona.farSpread * splashCorona.opened * design.unit * (1 - encore.handed)
                        color: farInk.value
                    }
                ]
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                helpShown: false
                // The help's entries are the frame's actions, which the pad's X, Y, Select and Start invoke; the help line draws them.
                help: Help.entries(root.helpKind)
                padFamily: Device.family
                focus: true
                controller.navigationShown: false
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    // The header, arriving from 16 above (ec-h1).
                    Item {
                        objectName: "arrivalHeader"
                        anchors.fill: parent
                        opacity: encore.headerOpacity
                        transform: Translate { y: encore.headerY * design.unit }
                        Brand {
                            id: brand
                            unit: design.unit
                            x: 96 * design.unit
                            y: 80 * design.unit - height / 2
                        }
                        // The sections after the brand, the board's 64 px gap between them (Main:27).
                        Nav {
                            unit: design.unit
                            controller: frame.controller
                            x: brand.x + brand.width + 64 * design.unit
                            y: 80 * design.unit - height / 2
                        }
                        Status {
                            id: status
                            unit: design.unit
                            controller: frame.controller
                            glassColor: ink.glass
                            pixelScale: canvas.unit
                            battery: true
                            batteryPicture: true
                            x: parent.width - 96 * design.unit - width
                            y: 80 * design.unit - height / 2
                            below: root.firstButton ?? root.currentTile
                        }
                        // The art behind the disc named on the glass under the status, while the shown game's art gives its colours (Main:75).
                        Rectangle {
                            id: artLabel
                            objectName: "artLabel"
                            // The splash's board draws no label, so none while the splash shows.
                            visible: root.artHues.length > 0 && !encore.splashing
                            opacity: encore.handed
                            x: parent.width - 96 * design.unit - width
                            y: 150 * design.unit
                            width: artWords.width + 2 * 16 * design.unit
                            height: artWords.height + 2 * 8 * design.unit
                            radius: 18 * design.unit
                            color: ink.glass
                            Line {
                                id: artWords
                                objectName: "artWords"
                                unit: design.unit
                                x: 16 * design.unit
                                y: 8 * design.unit
                                text: Words.gameArt(root.currentTile?.title ?? "")
                                capitalization: Font.AllUppercase
                                color: ink.help
                                size: 16
                                spacing: 0.14 * 16
                            }
                        }
                        // With no game on the row the search is where the focus is, so the pad reaches it.
                        Binding {
                            target: status.search
                            property: "focus"
                            value: tiles.count === 0
                        }
                    }
                    // The shown game's words and buttons, arriving from 30 to the left (ec-h2).
                    Item {
                        objectName: "arrivalMain"
                        anchors.fill: parent
                        opacity: encore.mainOpacity
                        transform: Translate { x: encore.mainX * design.unit }
                        Line {
                            id: eyebrow
                            objectName: "eyebrow"
                            unit: design.unit
                            x: 96 * design.unit
                            y: 250 * design.unit
                            width: 980 * design.unit
                            // "Continue · Super Nintendo" in capitals, as the board draws it (Main:54).
                            text: root.resuming ? Words.continueLine(Session.console) : ""
                            capitalization: Font.AllUppercase
                            color: ink.rose
                            elide: Text.ElideRight
                            size: 20
                            weight: Font.DemiBold
                            spacing: 0.22 * 20
                        }
                        // The title's line box, the board's line height of 1.02 (Main:55): the next line follows it, as the board's column does.
                        Item {
                            id: titleLine
                            objectName: "titleLine"
                            x: 96 * design.unit
                            y: eyebrow.y + eyebrow.height + 26 * design.unit
                            width: 980 * design.unit
                            height: title.lineCount * title.lineHeight * design.unit
                        }
                        Line {
                            id: title
                            objectName: "title"
                            unit: design.unit
                            x: 96 * design.unit
                            // The face's own height about the line box's middle, as CSS spreads a line's leading equally above and below.
                            y: titleLine.y + (titleLine.height - height) / 2
                            width: 980 * design.unit
                            text: root.resuming ? Session.title : root.currentTile?.title ?? (Library.consoles.count === 0 ? Words.addFolder : Words.noRecent)
                            color: Tokens.textStrong
                            // A game's name keeps the board's one line; the program's own words in its place (Words.addFolder, Words.noRecent) take a second.
                            wrapMode: root.currentTile === null ? Text.WordWrap : Text.NoWrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                            lineHeightMode: Text.FixedHeight
                            lineHeight: 1.02 * 112
                            size: 112
                            weight: Tokens.weightTitle
                            spacing: -0.02 * 112
                        }
                        Line {
                            id: facts
                            objectName: "facts"
                            unit: design.unit
                            x: 96 * design.unit
                            y: titleLine.y + titleLine.height + 26 * design.unit
                            width: 980 * design.unit
                            // The board's line (Main:56): when it was last played and the time played; it follows the clock, which ticks each minute.
                            text: root.resuming ? (Device.clock.length >= 0 ? Words.playedFacts(Session.pausedSince, Session.played) : "") : root.currentTile?.facts ?? ""
                            color: Tokens.muted
                            elide: Text.ElideRight
                            size: 26
                            weight: Tokens.weightSentence
                        }
                        // The resume point (Main:57-60): framed in a rounded inputSurface card while the suspended game is shown.
                        Rectangle {
                            id: resume
                            objectName: "resume"
                            x: 96 * design.unit
                            y: facts.y + facts.height + 26 * design.unit
                            width: Math.min(980 * design.unit, Math.max(560 * design.unit, resumeWords.x + resumeWords.implicitWidth + 24 * design.unit))
                            height: 84 * design.unit
                            radius: 18 * design.unit
                            color: Tokens.inputSurface
                            border.width: 1 * design.unit
                            border.color: ink.plateEdge
                            visible: root.resuming
                            // The game's own picture at its centre, cut to the board's 96 by 60.
                            ShaderEffectSource {
                                id: pausedTexture
                                readonly property Item frame: Session.frame
                                readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 96 / 60) : 0
                                readonly property real cutHeight: pausedTexture.cutWidth * 60 / 96
                                sourceItem: root.resuming && resume.visible ? pausedTexture.frame : null
                                hideSource: false
                                visible: false
                                sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                textureSize: Qt.size(Math.ceil(resumePicture.width * canvas.unit), Math.ceil(resumePicture.height * canvas.unit))
                            }
                            Shape {
                                id: resumePicture
                                objectName: "resumePicture"
                                x: 12 * design.unit
                                y: (resume.height - height) / 2
                                width: 96 * design.unit
                                height: 60 * design.unit
                                readonly property real radius: 10 * design.unit
                                visible: Session.frame !== null
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: "transparent"
                                    fillColor: "transparent"
                                    fillItem: pausedTexture
                                    fillTransform: PlanarTransform.fromScale(resumePicture.width / Math.max(1, pausedTexture.textureSize.width), resumePicture.height / Math.max(1, pausedTexture.textureSize.height))
                                    PathRectangle {
                                        width: resumePicture.width
                                        height: resumePicture.height
                                        radius: resumePicture.radius
                                    }
                                }
                            }
                            // The picture's inset rim, 2 px of rgba(246, 231, 216, 0.5) (Main:58).
                            Rectangle {
                                objectName: "resumeRim"
                                // The rim's colour, which the test reads.
                                readonly property color paint: ink.resumeRim
                                anchors.fill: resumePicture
                                radius: resumePicture.radius
                                color: "transparent"
                                border.color: paint
                                border.width: 2 * design.unit
                            }
                            Line {
                                id: resumeWords
                                objectName: "resumeWords"
                                unit: design.unit
                                x: resumePicture.x + resumePicture.width + 16 * design.unit
                                y: (resume.height - height) / 2
                                text: root.resuming ? Words.resumePoint(Session.pausedSince) : ""
                                color: ink.help
                                size: 22
                            }
                        }
                        Row {
                            objectName: "buttons"
                            x: 96 * design.unit
                            // The board's column gap and the buttons' own margin above them (Main:53, 61), under the resume point when it is drawn.
                            y: (resume.visible ? resume.y + resume.height : facts.y + facts.height) + (26 + 18) * design.unit
                            spacing: 18 * design.unit
                            visible: root.currentTile !== null
                            Repeater {
                                id: buttons
                                model: Help.entries(root.helpKind, { card: true })
                                Pill {
                                    id: pill
                                    required property int index
                                    required property var model
                                    unit: design.unit
                                    controller: frame.controller
                                    objectName: "button." + pill.index
                                    place: pill.index
                                    pixelScale: canvas.unit
                                    visible: pill.index < root.gameButtons
                                    live: pill.model.enabled ?? true
                                    filled: pill.index === 0 && pill.activeFocus
                                    fill: Tokens.surfaceRaised
                                    borderWidth: pill.index === 0 ? 1.5 * design.unit : Tokens.hoverBorder
                                    outline: pill.index === 0 ? "#94F6E7D8" : ink.pillLine
                                    leadGlyph: pill.index === 0
                                    labelWeight: pill.index === 0 ? Font.DemiBold : Font.Normal
                                    glyphSide: pill.filled ? 46 : (pill.index === 0 ? 38 : 44)
                                    leadLetterSize: pill.filled ? 22 : 18
                                    labelSize: pill.filled ? 26 : 24
                                    trail: pill.filled ? 34 : 30
                                    gap: pill.filled ? 16 : 14
                                    // A plays the game; the others are their buttons' presses, which the host acts on as the help line does.
                                    onActivated: pill.index === 0 ? Actions.play(root.currentTile?.key ?? "") : frame.controller.offer(pill.button)
                                    KeyNavigation.left: pill.index > 0 ? buttons.itemAt(pill.index - 1) : null
                                    KeyNavigation.right: pill.index + 1 < Math.min(root.gameButtons, buttons.count) ? buttons.itemAt(pill.index + 1) : null
                                    KeyNavigation.up: status.search
                                    KeyNavigation.down: root.currentTile
                                }
                            }
                        }
                    }
                    // Recent, arriving from 36 below (ec-h3).
                    Item {
                        objectName: "arrivalRecent"
                        anchors.fill: parent
                        opacity: encore.recentOpacity
                        transform: Translate { y: encore.recentY * design.unit }
                        Line {
                            id: recentHeading
                            objectName: "recentHeading"
                            unit: design.unit
                            x: 96 * design.unit
                            y: 790 * design.unit
                            text: Words.recentEntry
                            color: Tokens.text
                            size: 28
                            weight: Font.DemiBold
                        }
                        Item {
                            id: row
                            objectName: "recent"
                            x: 96 * design.unit
                            y: recentHeading.y + recentHeading.height + 22 * design.unit
                            width: parent.width - x
                            height: 196 * design.unit
                            Repeater {
                                id: tiles
                                // Every game played, the suspended one first among them as the board puts it.
                                model: Library.recent
                                onItemAdded: (index, item) => root.firstTile = index === 0 ? item : root.firstTile
                                Walkable {
                                    id: tile
                                    required property int index
                                    required property string key
                                    required property string title
                                    required property string hero
                                    required property string consoleName
                                    required property real consoleHue
                                    readonly property bool big: tile.index === root.current
                                    readonly property real radius: 18 * design.unit
                                    controller: frame.controller
                                    objectName: "tile." + tile.index
                                    focus: tile.index === 0
                                    Accessible.name: tile.title
                                    // The board's tiles, 264 by 165 and the shown one 312 by 196, 24 apart, their feet on the row's foot.
                                    x: tile.index * (264 + 24) * design.unit + (tile.index > root.current ? (312 - 264) * design.unit : 0)
                                    width: (tile.big ? 312 : 264) * design.unit
                                    height: (tile.big ? 196 : 165) * design.unit
                                    y: row.height - height
                                    visible: tile.x + tile.width <= row.width
                                    onActivated: Actions.play(tile.key)
                                    onActiveFocusChanged: {
                                        if (!tile.activeFocus)
                                            return
                                        root.current = tile.index
                                        Actions.focus(tile.key)
                                    }
                                    KeyNavigation.left: tile.index > 0 ? tiles.itemAt(tile.index - 1) : null
                                    KeyNavigation.right: tiles.count > tile.index + 1 ? tiles.itemAt(tile.index + 1) : null
                                    KeyNavigation.up: root.firstButton ?? status.search
                                    // A resting tile's shadow (Main:83): 0 10 px 30 px rgba(0, 0, 0, 0.45); the focused one has its ring instead.
                                    Glow {
                                        objectName: "tileShadow." + tile.index
                                        anchors.fill: parent
                                        visible: tile.visible && !tile.activeFocus
                                        radius: tile.radius
                                        color: "#73000000"
                                        blur: 30 * design.unit
                                        offsetY: 10 * design.unit
                                        pixelScale: canvas.unit
                                    }
                                    RoundPicture {
                                        objectName: "tileArt." + tile.index
                                        anchors.fill: parent
                                        source: tile.visible ? tile.hero : ""
                                        radius: tile.radius
                                        crop: true
                                        pixelScale: canvas.unit
                                    }
                                    TileWords {
                                        unit: design.unit
                                        place: tile.index
                                        // The tile's corners, which clip the shade on the board (Main:83).
                                        radius: 18
                                        title: tile.title
                                        // The console alone, as the board's tile line draws {{t.system}} (Main:86).
                                        facts: tile.consoleName
                                        titleInk: ink.tileTitle
                                        factsInk: ink.tileFacts
                                        shadeInk: ink.shade
                                        shadeClear: ink.shadeClear
                                    }
                                    // Tile focus ring: 4 px gap, 3 px accent, 40 px glow 10 px out at rgba(240, 180, 196, 0.5) (Main:113).
                                    FocusRing {
                                        objectName: "tileRing"
                                        anchors.fill: parent
                                        radius: tile.radius
                                        gap: 4 * design.unit
                                        thickness: 3 * design.unit
                                        glowColor: ink.ringGlow
                                        glowBlur: 40 * design.unit
                                        glowSpread: 10 * design.unit
                                        bandRadius: tile.radius + 7 * design.unit
                                        pixelScale: canvas.unit
                                        visible: tile.visible && tile.activeFocus
                                        opacity: encore.handed
                                    }
                                }
                            }
                        }
                    }
                    HelpLine {
                        // The help line, fading in last (ec-h4).
                        opacity: encore.footerOpacity
                        unit: design.unit
                        controller: frame.controller
                        // The board's words for the frame's entries: X Options, Start the Home card, no Select search (Main:94-97).
                        model: Help.entries(root.helpKind, { card: true })
                        ink: ink.help
                        SectionsHelp {
                            unit: design.unit
                            controller: frame.controller
                            color: ink.glass
                            borderColor: "#38F6E7D8"
                            borderWidth: 1
                            wordsColor: ink.help
                            placeWords: tiles.count > 0 ? Words.place(root.current, tiles.count) : ""
                        }
                    }
                }
            }
        }
    }
}
