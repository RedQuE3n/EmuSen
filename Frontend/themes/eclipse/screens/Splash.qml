pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Eclipse's splash as EclipseSplashPlay.dc.html draws it: the stars, the sun and the moon crossing it, the corona, a plain planet for each installed core resting on the orbit, the flare, the name and the counts, then the disc opening into Home's, on the program's clock (Splash.time) with the theme's sound (EmuSen_Qt.md §3.4.8, the splash).
ThemeScreen {
    id: screen
    objectName: "splashScreen"
    role: ThemeScreen.Splash
    entersHome: true
    // Home whole, ec-h4's 88 per cent of the board's 11 s loop (splash_times_original.json): the splash's length in ms, a constant of the design.
    length: 9680
    still: Tokens.reducedMotion
    // The sound the board was made with (splash-sound-chosen.wav, R-20261111-118), its samples as they are, in FLAC (EmuSen_Qt.md §3.5.3).
    sound.source: "../sounds/splash.flac"
    NormalizedCanvas {
        id: canvas
        objectName: "splashCanvas"
        Item {
            id: root
            anchors.fill: parent
            // The moment drawn as the board's per cent of its 11 s loop (EclipseSplashPlay:14-27), a constant of the design.
            readonly property real p: Splash.time / 110
            Board {
                id: design
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            readonly property real u: design.unit
            // The orbit, the board's ellipse of half axes 600 and 150 about (960, 480) turned -9 degrees (EclipseSplashPlay:45, 161): constants of the design.
            readonly property real orbitX: 960
            readonly property real orbitY: 480
            readonly property real orbitA: 600
            readonly property real orbitB: 150
            readonly property real tilt: -9
            // The ground the board draws everything on (EclipseSplashPlay:31), giving way to Home's as ec-home brings its backdrop in.
            Rectangle {
                objectName: "splashGround"
                anchors.fill: parent
                color: ink.splashGround
                opacity: root.p < 56 ? 1 : root.p < 76 ? 1 - (root.p - 56) / 20 : 0
            }
            // ec-stars: in by 6%, out from 54% to 64%.
            Item {
                objectName: "splashStars"
                anchors.fill: parent
                opacity: root.p < 6 ? root.p / 6 : root.p < 54 ? 1 : root.p < 64 ? (64 - root.p) / 10 : 0
                visible: opacity > 0
                // The board's 110 stars from its generator, seeded 0x4d6f6f6e, four draws a star (EclipseSplashPlay:153-159).
                Repeater {
                    id: starField
                    model: 110
                    Star {}
                }
                component Star: Rectangle {
                    id: star
                    required property int index
                    // The generator's state before this star's draws: the seed, else the star before's last.
                    readonly property real from: star.index === 0 ? 0x4d6f6f6e : (starField.itemAt(star.index - 1) as Star)?.last ?? 0
                    readonly property real d0: (Math.imul(star.from, 1664525) + 1013904223) >>> 0
                    readonly property real d1: (Math.imul(star.d0, 1664525) + 1013904223) >>> 0
                    readonly property real d2: (Math.imul(star.d1, 1664525) + 1013904223) >>> 0
                    readonly property real last: (Math.imul(star.d2, 1664525) + 1013904223) >>> 0
                    objectName: "splashStar." + star.index
                    x: Math.round(star.d1 / 4294967296 * design.width) * root.u
                    y: Math.round(star.d2 / 4294967296 * design.height) * root.u
                    width: (star.d0 / 4294967296 < 0.85 ? 2 : 3.5) * root.u
                    height: width
                    radius: width / 2
                    color: ink.star
                    opacity: Math.round(100 * (0.25 + star.last / 4294967296 * 0.6)) / 100
                }
            }
            // ec-orbit: in from 10% to 19%, out from 50% to 58%; its 1.5 px line inside its box.
            Shape {
                id: orbit
                objectName: "splashOrbit"
                readonly property real line: 1.5
                x: (root.orbitX - root.orbitA) * root.u
                y: (root.orbitY - root.orbitB) * root.u
                width: 2 * root.orbitA * root.u
                height: 2 * root.orbitB * root.u
                rotation: root.tilt
                opacity: root.p < 10 ? 0 : root.p < 19 ? (root.p - 10) / 9 : root.p < 50 ? 1 : root.p < 58 ? (58 - root.p) / 8 : 0
                visible: opacity > 0
                preferredRendererType: Shape.CurveRenderer
                ShapePath {
                    fillColor: "transparent"
                    strokeColor: ink.orbit
                    strokeWidth: orbit.line * root.u
                    PathAngleArc {
                        centerX: orbit.width / 2
                        centerY: orbit.height / 2
                        radiusX: orbit.width / 2 - orbit.line * root.u / 2
                        radiusY: orbit.height / 2 - orbit.line * root.u / 2
                        startAngle: 0
                        sweepAngle: 360
                    }
                }
            }
            component Planet: Item {
                id: planet
                required property int index
                // Whether this layer draws the planets in front of the disc or behind it.
                property bool front
                PlanetInk {
                    id: planetInk
                    place: planet.index
                }
                readonly property real theta: planetInk.angle * Math.PI / 180
                readonly property real ex: root.orbitA * Math.cos(planet.theta)
                readonly property real ey: -root.orbitB * Math.sin(planet.theta)
                readonly property real turn: root.tilt * Math.PI / 180
                readonly property real cx: root.orbitX + planet.ex * Math.cos(planet.turn) - planet.ey * Math.sin(planet.turn)
                readonly property real cy: root.orbitY + planet.ex * Math.sin(planet.turn) + planet.ey * Math.cos(planet.turn)
                readonly property bool inFront: Math.sin(planet.theta) < 0
                // The board's side: in front 26 and up to 14 more the nearer it comes, behind 18, rounded as the board writes it.
                readonly property real side: Math.round(planet.inFront ? 26 + 14 * -Math.sin(planet.theta) : 18)
                // Its three colours, as the tests read them.
                readonly property color light: planetInk.light
                readonly property color middle: planetInk.middle
                readonly property color shade: planetInk.shade
                // ec-planet, ease-out, 0.09 s later a planet: risen from 11% to 19%, gone from 49% to 57%.
                readonly property real q: root.p - planet.index * 9 / 11
                Ease {
                    id: rise
                    x2: 0.58
                    y2: 1
                    progress: (planet.q - 11) / 8
                }
                Ease {
                    id: set
                    x2: 0.58
                    y2: 1
                    progress: (planet.q - 49) / 8
                }
                objectName: "splashPlanet." + planet.index
                visible: planet.inFront === planet.front && planet.opacity > 0
                x: Math.round(planet.cx - planet.side / 2) * root.u
                y: Math.round(planet.cy - planet.side / 2) * root.u
                width: planet.side * root.u
                height: width
                opacity: planet.q < 19 ? rise.value : 1 - set.value
                transform: [
                    Scale {
                        origin.x: planet.width / 2
                        origin.y: planet.height / 2
                        xScale: 0.6 + 0.4 * rise.value
                        yScale: 0.6 + 0.4 * rise.value
                    },
                    Translate {
                        y: 26 * (1 - rise.value) * root.u
                    }
                ]
                // Its glow, 0 0 16px 3px of its middle colour at 0x88 (EclipseSplashPlay:169).
                Glow {
                    anchors.fill: parent
                    radius: width / 2
                    color: Qt.rgba(planet.middle.r, planet.middle.g, planet.middle.b, 0x88 / 255)
                    blur: 16 * root.u
                    spread: 3 * root.u
                    pixelScale: canvas.unit
                }
                // Its body, a radial gradient at 32% 30% to its farthest corner: light, middle at 52%, shade.
                Shape {
                    anchors.fill: parent
                    preferredRendererType: Shape.CurveRenderer
                    ShapePath {
                        strokeWidth: -1
                        fillGradient: RadialGradient {
                            centerX: 0.32 * planet.width
                            centerY: 0.30 * planet.width
                            centerRadius: Math.hypot(0.68, 0.70) * planet.width
                            focalX: 0.32 * planet.width
                            focalY: 0.30 * planet.width
                            focalRadius: 0
                            GradientStop { position: 0; color: planet.light }
                            GradientStop { position: 0.52; color: planet.middle }
                            GradientStop { position: 1; color: planet.shade }
                        }
                        PathAngleArc {
                            centerX: planet.width / 2
                            centerY: planet.width / 2
                            radiusX: planet.width / 2
                            radiusY: planet.width / 2
                            startAngle: 0
                            sweepAngle: 360
                        }
                    }
                }
            }
            Repeater {
                model: Library.cores
                Planet {
                    front: false
                }
            }
            // ec-sun: in by 4%, out from 20% to 25%; 0 0 60px 16px rgba(255, 240, 225, 0.6) round it (EclipseSplashPlay:51).
            Item {
                id: sun
                objectName: "splashSun"
                x: 750 * root.u
                y: 270 * root.u
                width: 420 * root.u
                height: width
                opacity: root.p < 4 ? root.p / 4 : root.p < 20 ? 1 : root.p < 25 ? (25 - root.p) / 5 : 0
                visible: opacity > 0
                Glow {
                    anchors.fill: parent
                    radius: width / 2
                    color: ink.sunGlow
                    blur: 60 * root.u
                    spread: 16 * root.u
                    pixelScale: canvas.unit
                }
                Rectangle {
                    anchors.fill: parent
                    radius: width / 2
                    color: ink.sun
                }
            }
            // The moon crossing the sun, then the disc Home is drawn on: ec-group, cubic-bezier(0.65, 0, 0.35, 1), opening from 52% to 76% to five times its size, moved -810, 120.
            Item {
                id: group
                objectName: "splashGroup"
                x: 750 * root.u
                y: 270 * root.u
                width: 420 * root.u
                height: width
                Ease {
                    id: opening
                    x1: 0.65
                    x2: 0.35
                    y2: 1
                    progress: (root.p - 52) / 24
                }
                transform: [
                    Scale {
                        origin.x: group.width / 2
                        origin.y: group.height / 2
                        xScale: 1 + 4 * opening.value
                        yScale: 1 + 4 * opening.value
                    },
                    Translate {
                        x: -810 * opening.value * root.u
                        y: 120 * opening.value * root.u
                    }
                ]
                // The corona, ec-corona: in from 17% to 26%; the rim and three glows round the disc, with no fill (EclipseSplashPlay:54).
                CoronaDisc {
                    objectName: "splashCorona"
                    anchors.fill: parent
                    fill: "transparent"
                    rim: 2 * root.u
                    pixelScale: canvas.unit
                    opacity: root.p < 17 ? 0 : root.p < 26 ? (root.p - 17) / 9 : root.p < 97 ? 1 : (100 - root.p) / 3
                    visible: opacity > 0
                    glows: [
                        CoronaGlow {
                            blur: 26 * root.u
                            spread: 6 * root.u
                            color: ink.coronaNear
                        },
                        CoronaGlow {
                            blur: 90 * root.u
                            spread: 22 * root.u
                            color: ink.coronaMiddle
                        },
                        CoronaGlow {
                            blur: 240 * root.u
                            spread: 80 * root.u
                            color: ink.coronaFar
                        }
                    ]
                }
                // ec-moon, cubic-bezier(0.3, 0, 0.2, 1): from -46, -12 to -40, -10 by 8%, home by 22%.
                Rectangle {
                    id: moon
                    objectName: "splashMoon"
                    anchors.fill: parent
                    radius: width / 2
                    color: ink.splashGround
                    Ease {
                        id: nearing
                        x1: 0.3
                        x2: 0.2
                        y2: 1
                        progress: root.p / 8
                    }
                    Ease {
                        id: covering
                        x1: 0.3
                        x2: 0.2
                        y2: 1
                        progress: (root.p - 8) / 14
                    }
                    transform: Translate {
                        x: (root.p < 8 ? -46 + 6 * nearing.value : -40 + 40 * covering.value) * root.u
                        y: (root.p < 8 ? -12 + 2 * nearing.value : -10 + 10 * covering.value) * root.u
                    }
                }
            }
            // The flare crossing the rim once, ec-flare, ease-out: in from 25% to 30% to 1.25 times, settling by 36% at 0.75, gone from 49% to 55% to 0.6 (EclipseSplashPlay:58-62).
            Item {
                id: flare
                objectName: "splashFlare"
                x: 1027 * root.u
                y: 253 * root.u
                width: 180 * root.u
                height: 160 * root.u
                Ease {
                    id: flash
                    x2: 0.58
                    y2: 1
                    progress: (root.p - 25) / 5
                }
                Ease {
                    id: settle
                    x2: 0.58
                    y2: 1
                    progress: (root.p - 30) / 6
                }
                Ease {
                    id: fade
                    x2: 0.58
                    y2: 1
                    progress: (root.p - 49) / 6
                }
                opacity: root.p < 30 ? flash.value : root.p < 49 ? 1 - 0.25 * settle.value : 0.75 * (1 - fade.value)
                visible: opacity > 0
                scale: root.p < 30 ? 0.2 + 1.05 * flash.value : root.p < 49 ? 1.25 - 0.25 * settle.value : 1 - 0.4 * fade.value
                Item {
                    x: 75 * root.u
                    y: 65 * root.u
                    width: 30 * root.u
                    height: width
                    Glow {
                        anchors.fill: parent
                        radius: width / 2
                        color: ink.flareFar
                        blur: 70 * root.u
                        spread: 26 * root.u
                        pixelScale: canvas.unit
                    }
                    Glow {
                        anchors.fill: parent
                        radius: width / 2
                        color: ink.flareNear
                        blur: 22 * root.u
                        spread: 10 * root.u
                        pixelScale: canvas.unit
                    }
                    Rectangle {
                        anchors.fill: parent
                        radius: width / 2
                        color: ink.flare
                    }
                }
                Rectangle {
                    y: 78 * root.u
                    width: 180 * root.u
                    height: 4 * root.u
                    radius: 2 * root.u
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0; color: ink.flareClear }
                        GradientStop { position: 0.5; color: ink.flareAcross }
                        GradientStop { position: 1; color: ink.flareClear }
                    }
                }
                Rectangle {
                    x: 88 * root.u
                    width: 4 * root.u
                    height: 160 * root.u
                    radius: 2 * root.u
                    gradient: Gradient {
                        GradientStop { position: 0; color: ink.flareClear }
                        GradientStop { position: 0.5; color: ink.flareDown }
                        GradientStop { position: 1; color: ink.flareClear }
                    }
                }
            }
            Repeater {
                model: Library.cores
                Planet {
                    front: true
                }
            }
            // The name, the line and the library's counts under the disc, centred, ec-word, ease-out: in from 32% to 40% from 18 below, out from 50% to 57% to 10 above (EclipseSplashPlay:68-72).
            Item {
                id: words
                anchors.fill: parent
                Ease {
                    id: arriving
                    x2: 0.58
                    y2: 1
                    progress: (root.p - 32) / 8
                }
                Ease {
                    id: leaving
                    x2: 0.58
                    y2: 1
                    progress: (root.p - 50) / 7
                }
                opacity: root.p < 40 ? arriving.value : 1 - leaving.value
                visible: opacity > 0
                transform: Translate {
                    y: (root.p < 40 ? 18 * (1 - arriving.value) : -10 * leaving.value) * root.u
                }
                // The name's line box, line-height 1 at 104 px, its words centred in it as CSS spreads the leading.
                Item {
                    id: nameLine
                    y: 800 * root.u
                    width: parent.width
                    height: 104 * root.u
                }
                Line {
                    objectName: "splashWordmark"
                    unit: root.u
                    x: (parent.width - width) / 2
                    y: nameLine.y + (nameLine.height - height) / 2
                    text: Words.name
                    color: Tokens.text
                    size: 104
                    weight: Tokens.weightWordmark
                    spacing: 0.06 * 104
                }
                Rectangle {
                    id: rule
                    objectName: "splashRule"
                    x: (parent.width - width) / 2
                    y: nameLine.y + nameLine.height + 22 * root.u
                    width: 360 * root.u
                    height: 2 * root.u
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0; color: ink.ruleClear }
                        GradientStop { position: 0.5; color: ink.rule }
                        GradientStop { position: 1; color: ink.ruleClear }
                    }
                }
                // Its line's top 22 under the rule, the face's own height under it, as Home's lines follow one another (Main:56).
                Line {
                    objectName: "splashCounts"
                    unit: root.u
                    x: (parent.width - width) / 2
                    y: rule.y + rule.height + 22 * root.u
                    text: Words.splashCounts(Library.gameCount, Library.cores.count)
                    color: ink.splashCounts
                    size: 24
                    spacing: 0.18 * 24
                }
            }
        }
    }
}
