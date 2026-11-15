import QtQuick
import QtQuick.Shapes
import EmuSen.Kit as Kit

// The mark beside a core's name, from PlanetMarks, its one owner: a sky-named core's symbol or its console's body glyph stroked in the ink, or libretro's badge in the ink with the alien cut through it (EmuSen_Interface.md §24.6, EmuSen_Kit.md §7.115).
Item {
    id: root
    // The core's mark by name, as a console row's engineMark gives it.
    property string mark
    // Whether the core is a hosted libretro core, which its mark's name says unless set.
    property bool libretro: root.mark === Kit.PlanetMarks.libretro
    property color ink: Kit.Tokens.textStrong
    // The alien's colour, cut through the badge.
    property color cut: Kit.Tokens.surfaceRaised
    Accessible.role: Accessible.Graphic
    Accessible.name: root.mark
    Item {
        anchors.fill: parent
        visible: !root.libretro
        Accessible.role: Accessible.Graphic
        Shape {
            objectName: "planetMarkGlyph"
            width: Kit.PlanetMarks.boxSize
            height: Kit.PlanetMarks.boxSize
            scale: Math.min(root.width, root.height) / Kit.PlanetMarks.boxSize
            transformOrigin: Item.TopLeft
            preferredRendererType: Shape.CurveRenderer
            ShapePath {
                fillColor: "transparent"
                strokeColor: root.ink
                strokeWidth: Kit.PlanetMarks.stroke
                fillRule: ShapePath.OddEvenFill
                capStyle: ShapePath.RoundCap
                joinStyle: ShapePath.RoundJoin
                PathSvg {
                    objectName: "planetMarkPath"
                    path: Kit.PlanetMarks.path(root.mark)
                }
            }
        }
    }
    Shape {
        objectName: "planetMarkBadge"
        visible: root.libretro
        width: Kit.PlanetMarks.badgeBox
        height: Kit.PlanetMarks.badgeBox
        scale: Math.min(root.width, root.height) / Kit.PlanetMarks.badgeBox
        transformOrigin: Item.TopLeft
        ShapePath {
            fillColor: root.ink
            strokeColor: "transparent"
            PathSvg {
                objectName: "planetMarkBadgePath"
                path: Kit.PlanetMarks.badge
            }
        }
        ShapePath {
            fillColor: root.cut
            strokeColor: "transparent"
            PathSvg {
                objectName: "planetMarkAlienPath"
                path: Kit.PlanetMarks.alien
            }
        }
    }
}
