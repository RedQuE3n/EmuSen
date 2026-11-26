import QtQuick
import EmuSen.Theme

// The help line's LB RB Sections on its glass at the line's right, Help.sections in the kit's HelpBar with the boards' boxed glyphs (Main:99-103; EmuSen_Qt.md §3.4.8), with an optional position suffix joined by Words.between.
Rectangle {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    // The words' colour, the help line's, and the entries: Help.sections, or a page's Section (EclipseGame:92-96).
    property color wordsColor: "transparent"
    property alias model: bar.model
    property string placeWords: ""
    property color placeColor: Tokens.muted
    property color borderColor: "transparent"
    property real borderWidth: 0
    // The glass's height and side padding, the words' size, the gap between the glyphs and the words, and a ringed glyph's side, in board pixels (Main:99-103).
    property real tall: 46
    property real pad: 18
    property real size: 21
    property real glyphGap: 10
    property real glyphSide: 34
    objectName: "helpSections"
    width: contentRow.width + 2 * root.pad * root.unit
    height: root.tall * root.unit
    radius: height / 2
    border.color: root.borderColor
    border.width: root.borderWidth * root.unit
    Row {
        id: contentRow
        x: root.pad * root.unit
        y: (root.height - height) / 2
        spacing: root.glyphGap * root.unit
        HelpBar {
            id: bar
            objectName: "helpSectionsBar"
            y: (contentRow.height - height) / 2
            model: Help.sections
            controller: root.controller
            family: Device.family
            fitted: true
            ruled: false
            color: "transparent"
            labelColor: root.wordsColor
            iconColor: Tokens.muted
            labelFont.family: Tokens.faceText
            label: Component {
                Line {
                    objectName: "helpSectionsWords"
                    unit: root.unit
                    size: root.size
                }
            }
            glyphGap: root.glyphGap * root.unit
            glyphSize: root.glyphSide * root.unit
            snapped: false
            glyph: Component {
                // No main action on the glass: LB is drawn in its box as RB is (Main:100-101).
                Glyph {
                    unit: root.unit
                    side: root.glyphSide
                    ink: root.wordsColor
                    leads: false
                }
            }
        }
        Line {
            objectName: "helpSectionsDivider"
            y: (contentRow.height - height) / 2
            visible: root.placeWords.length > 0
            unit: root.unit
            size: root.size
            color: root.placeColor
            text: Words.between
        }
        Line {
            objectName: "helpSectionsPlace"
            y: (contentRow.height - height) / 2
            visible: root.placeWords.length > 0
            unit: root.unit
            size: root.size
            color: root.placeColor
            text: root.placeWords
        }
    }
}
