pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Orrery's Home: the paused game under the moon, the recent games on one wide orbit with the focus at its apex, and the dial of the seven places along the foot (orrery.md §2).
ThemeScreen {
    id: screen
    objectName: "homeScreen"
    role: ThemeScreen.Home
    // Where the focus starts and where Start on Home takes it back: the panel, else the apex cover, else the first place.
    focusItems: Session.hasGame ? [panel] : (orbit.count > 0 ? [orbit] : [root.firstPlace ?? orbit])
    // A mote on the orbit's lower arc, `at` of the way from 150° to 30°, carried on by the resting phase.
    component Mote: Rectangle {
        id: mote
        property real at
        readonly property real theta: (150 - 120 * ((mote.at + root.phase) % 1)) * Math.PI / 180
        x: root.width / 2 + 620 * Math.cos(mote.theta) - 1.5
        y: 540 * Math.sin(mote.theta) - 1.5
        width: 3
        height: 3
        radius: 1.5
        color: root.lineInk
    }
    NormalizedCanvas {
        id: canvas
        objectName: "homeCanvas"
        FocusScope {
            id: root
            // First, so the parts read the faces from their first binding.
            Faces {
                id: faces
            }
            objectName: "home"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            readonly property real ox: (root.width - 1280) / 2
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property bool resting: !Tokens.reducedMotion && !(Device.battery >= 0 && !Device.charging)
            readonly property color lineInk: root.light ? Qt.rgba(74 / 255, 58 / 255, 40 / 255, 0.5) : Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.32)
            readonly property real pitch: (root.width - 128) / 7
            readonly property bool coverHeld: orbit.activeFocus
            readonly property var apex: orbit.currentItem
            readonly property Item firstPlace: root.made > 0 ? places.itemAt(0) : null
            property var focusedPlace: null
            // How many places are made, so a neighbour looked up by index is read again as each is.
            property int made: 0
            property real phase: 0
            NumberAnimation on phase {
                running: root.resting && root.visible
                from: 0
                to: 1
                duration: 60000
                loops: Animation.Infinite
            }
            Backdrop {
                objectName: "homeSky"
                anchors.fill: parent
                glowHue: panel.activeFocus ? Session.hue : (root.coverHeld && root.apex ? root.apex.consoleHue : -1)
                glowX: panel.activeFocus ? (640 + root.ox) / root.width : 0.5
                glowY: panel.activeFocus ? 216 / 800 : 540 / 800
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
                helpSpacing: 40
                helpGlyphGap: 10
                focus: true
                controller.navigationShown: false
                help: panel.activeFocus ? Help.entries(Help.Continue) : (root.coverHeld ? Help.entries(Help.Game) : Help.entries(Help.Place, { accept: root.focusedPlace ? root.focusedPlace.available : true }))
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Text {
                        objectName: "status"
                        x: 64
                        y: 40
                        text: Words.statusLine(Device.controllers, Device.clock)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "wordmark"
                        x: root.width - 64 - width
                        y: 37.6
                        text: Words.title
                        color: Tokens.textStrong
                        font.family: faces.light
                        font.weight: Font.Light
                        font.pixelSize: 24
                        font.letterSpacing: 6
                        lineHeight: 28.8
                        lineHeightMode: Text.FixedHeight
                    }
                    Moon {
                        objectName: "moon"
                        x: 346 + root.ox
                        y: 44
                        width: 26
                        height: 26
                    }
                    // The orbit's lower arc from 150° to 30° about (W/2, 0), and the four motes resting along it.
                    Scene {
                        objectName: "orbitLine"
                        anchors.fill: parent
                        Shape {
                            anchors.fill: parent
                            preferredRendererType: Shape.CurveRenderer
                            ShapePath {
                                strokeColor: root.lineInk
                                strokeWidth: 1
                                fillColor: "transparent"
                                PathAngleArc {
                                    centerX: root.width / 2
                                    centerY: 0
                                    radiusX: 620
                                    radiusY: 540
                                    startAngle: 150
                                    sweepAngle: -120
                                }
                            }
                        }
                        Mote { at: 0.2 }
                        Mote { at: 0.45 }
                        Mote { at: 0.7 }
                        Mote { at: 0.9 }
                    }
                    // Continue: the glass panel under the moon holding the paused frame, and its words beside it.
                    Walkable {
                        id: panel
                        objectName: "continuePanel"
                        controller: frame.controller
                        Accessible.name: Session.title
                        visible: Session.hasGame
                        focus: Session.hasGame
                        activeFocusOnTab: Session.hasGame
                        x: 400 + root.ox
                        y: 76
                        // Over the orbit's box, which reaches under its foot, so the pointer has all of it; no cover comes under it.
                        z: 1
                        width: 480
                        height: 280
                        onActivated: Actions.play(Session.key)
                        KeyNavigation.down: orbit.count > 0 ? orbit : (root.made > 3 ? places.itemAt(3) : null)
                        Rectangle {
                            objectName: "continueGlass"
                            anchors.fill: parent
                            radius: 12
                            color: Qt.rgba(Tokens.surfaceRaised.r, Tokens.surfaceRaised.g, Tokens.surfaceRaised.b, 0.5)
                            border.width: 1
                            border.color: root.light ? Qt.rgba(74 / 255, 58 / 255, 40 / 255, 0.25) : Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.18)
                        }
                        Item {
                            id: pausedFrame
                            objectName: "continueFrame"
                            x: 12
                            y: 12
                            width: 456
                            height: 256
                            Rectangle {
                                anchors.fill: parent
                                radius: 8
                                color: Tokens.voidFill
                            }
                            // The game's own picture drawn into a texture at the window's resolution, cut to 16:9 at its centre, its corners rounded as the frame's.
                            ShaderEffectSource {
                                id: pausedTexture
                                readonly property Item frame: Session.frame
                                readonly property real cutWidth: pausedTexture.frame ? Math.min(pausedTexture.frame.width, pausedTexture.frame.height * 16 / 9) : 0
                                readonly property real cutHeight: pausedTexture.cutWidth * 9 / 16
                                sourceItem: Session.hasGame && root.visible ? pausedTexture.frame : null
                                hideSource: true
                                visible: false
                                sourceRect: pausedTexture.frame ? Qt.rect((pausedTexture.frame.width - pausedTexture.cutWidth) / 2, (pausedTexture.frame.height - pausedTexture.cutHeight) / 2, pausedTexture.cutWidth, pausedTexture.cutHeight) : Qt.rect(0, 0, 0, 0)
                                textureSize: Qt.size(Math.ceil(456 * root.pixelScale), Math.ceil(256 * root.pixelScale))
                            }
                            Shape {
                                objectName: "continuePicture"
                                anchors.fill: parent
                                visible: Session.hasGame && Session.frame !== null
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: "transparent"
                                    fillColor: "transparent"
                                    fillItem: pausedTexture
                                    fillTransform: PlanarTransform.fromScale(1 / root.pixelScale, 1 / root.pixelScale)
                                    PathRectangle {
                                        width: 456
                                        height: 256
                                        radius: 8
                                    }
                                }
                            }
                            Rectangle {
                                objectName: "continueOnArt"
                                x: 0
                                y: 146
                                width: 456
                                height: 110
                                bottomLeftRadius: 8
                                bottomRightRadius: 8
                                gradient: Gradient {
                                    GradientStop { position: 0; color: "transparent" }
                                    GradientStop { position: 1; color: Tokens.overArtScrim }
                                }
                            }
                            Chip {
                                objectName: "continueChip"
                                x: 14
                                y: 256 - 12 - 26
                                kind: Help.Continue
                                onArt: true
                                faces: faces
                            }
                        }
                        Rectangle {
                            objectName: "continueRing"
                            visible: panel.activeFocus
                            x: -Tokens.ringOffset - Tokens.ringThickness
                            y: -Tokens.ringOffset - Tokens.ringThickness
                            width: parent.width + 2 * (Tokens.ringOffset + Tokens.ringThickness)
                            height: parent.height + 2 * (Tokens.ringOffset + Tokens.ringThickness)
                            radius: 16
                            color: "transparent"
                            border.width: Tokens.ringThickness
                            border.color: Tokens.accent
                        }
                    }
                    Column {
                        objectName: "continueWords"
                        visible: Session.hasGame
                        x: 904 + root.ox
                        y: 100
                        width: root.width - 968 - root.ox
                        spacing: 8
                        Text {
                            objectName: "continueEyebrow"
                            width: parent.width
                            text: Words.continuePlaying
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            font.letterSpacing: 2
                            font.capitalization: Font.AllUppercase
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "continueTitle"
                            width: parent.width
                            text: Session.title
                            wrapMode: Text.Wrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 32
                            lineHeight: 38.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Row {
                            spacing: 10
                            Scene {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: Session.hue >= 0
                                width: 14
                                height: 14
                                Rectangle {
                                    anchors.fill: parent
                                    radius: 7
                                    gradient: Gradient {
                                        GradientStop { position: 0; color: Qt.hsla(Math.max(0, Session.hue) / 360, 0.85, 0.8, 1) }
                                        GradientStop { position: 1; color: Qt.hsla(Math.max(0, Session.hue) / 360, 0.65, 0.45, 1) }
                                    }
                                }
                            }
                            Text {
                                objectName: "continueConsole"
                                width: Math.min(implicitWidth, root.width - 968 - root.ox - 24)
                                text: Session.console
                                elide: Text.ElideRight
                                color: Tokens.text
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                        Text {
                            objectName: "continuePaused"
                            width: parent.width
                            text: Session.pausedSince > 0 && Device.clock.length >= 0 ? Words.pausedAgo(Session.pausedSince) : ""
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                    // The recent games on the orbit about (W/2, 0), 620 x 540, 18° apart, the focus at the apex; its box clears a wide cover two from the apex (EmuSen_Qt.md §3.4.6).
                    Orbit {
                        id: orbit
                        objectName: "orbit"
                        x: 0
                        y: 316
                        width: root.width
                        height: 426
                        focus: !Session.hasGame && orbit.count > 0
                        model: Library.shelf
                        KeyNavigation.up: Session.hasGame ? panel : null
                        KeyNavigation.down: root.made > 3 ? places.itemAt(3) : null
                        delegate: OrbitCover {
                            id: cover
                            objectName: "orbitCover." + cover.index
                            orbit: orbit
                            controller: frame.controller
                            rx: 620
                            ry: 540
                            cy: 0
                            step: 18
                            coverHeight: 170
                            scaleStep: 0.1
                            minScale: 0.7
                            pixelScale: root.pixelScale
                            onActivated: Actions.play(cover.key)
                            onActiveFocusChanged: if (cover.activeFocus) Actions.focus(cover.key)
                        }
                    }
                    Text {
                        objectName: "noRecent"
                        visible: orbit.count === 0
                        x: root.width / 2 - 200
                        y: 528
                        width: 400
                        horizontalAlignment: Text.AlignHCenter
                        text: Words.noRecent
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Column {
                        objectName: "apexCaption"
                        visible: root.coverHeld && root.apex !== null
                        x: 64
                        y: 556
                        width: 440
                        spacing: 5
                        Text {
                            objectName: "apexTitle"
                            width: parent.width
                            text: root.apex ? root.apex.title : ""
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 26
                            lineHeight: 31
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "apexFacts"
                            width: parent.width
                            text: root.apex ? root.apex.facts : ""
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                    // The dial: each place a small wheel with eight teeth and its mark at the hub, its name and its line, in sevenths of the safe width.
                    Repeater {
                        id: places
                        model: Library.places
                        onItemAdded: root.made = root.made + 1
                        Walkable {
                            id: place
                            required property int index
                            required property string key
                            required property string name
                            required property string line
                            required property bool attention
                            required property bool available
                            readonly property color wheelInk: place.activeFocus ? Tokens.textStrong : root.lineInk
                            objectName: "place." + place.key
                            controller: frame.controller
                            Accessible.name: place.name
                            focus: !Session.hasGame && orbit.count === 0 && place.index === 0
                            x: 64 + place.index * root.pitch
                            y: 644
                            width: root.pitch
                            height: 92
                            onActiveFocusChanged: if (place.activeFocus) root.focusedPlace = place
                            onActivated: Actions.openPlace(place.key)
                            KeyNavigation.left: root.made > 6 && place.index > 0 ? places.itemAt(place.index - 1) : null
                            KeyNavigation.right: root.made > 6 && place.index < 6 ? places.itemAt(place.index + 1) : null
                            KeyNavigation.up: orbit.count > 0 ? orbit : (Session.hasGame ? panel : null)
                            Item {
                                anchors.fill: parent
                                opacity: place.available ? 1 : 0.38
                                DialWheel {
                                    objectName: "placeWheel." + place.key
                                    x: (root.pitch - 44) / 2
                                    y: 0
                                    teeth: 8
                                    inner: 17
                                    outer: 21
                                    first: 90
                                    ink: place.wheelInk
                                    Glyph {
                                        x: 14
                                        y: 14
                                        width: 16
                                        height: 16
                                        path: Marks.path(place.key)
                                        ink: Tokens.textStrong
                                        strokeWidth: 2
                                    }
                                }
                                Text {
                                    objectName: "placeName." + place.key
                                    x: 8
                                    y: 48
                                    width: root.pitch - 16
                                    horizontalAlignment: Text.AlignHCenter
                                    text: place.name
                                    elide: Text.ElideRight
                                    color: Tokens.textStrong
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 20
                                    font.letterSpacing: 1.2
                                    font.capitalization: Font.AllUppercase
                                    lineHeight: 24
                                    lineHeightMode: Text.FixedHeight
                                }
                                Row {
                                    x: (root.pitch - width) / 2
                                    y: 72
                                    spacing: 4
                                    Glyph {
                                        anchors.verticalCenter: parent.verticalCenter
                                        visible: place.attention
                                        width: 16
                                        height: 16
                                        path: Marks.attention
                                        filled: true
                                        ink: Tokens.attention
                                    }
                                    Text {
                                        objectName: "placeLine." + place.key
                                        width: Math.min(implicitWidth, root.pitch - 16 - (place.attention ? 20 : 0))
                                        text: place.line
                                        elide: Text.ElideRight
                                        color: place.attention ? Tokens.attention : Tokens.muted
                                        font.family: faces.regular
                                        font.pixelSize: 19
                                        lineHeight: 22.8
                                        lineHeightMode: Text.FixedHeight
                                    }
                                }
                            }
                            Rectangle {
                                objectName: "placeRing." + place.key
                                visible: place.activeFocus
                                x: (root.pitch - 58) / 2
                                y: -7
                                width: 58
                                height: 58
                                radius: 29
                                color: "transparent"
                                border.width: Tokens.ringThickness
                                border.color: Tokens.accent
                            }
                        }
                    }
                }
            }
        }
    }
}
