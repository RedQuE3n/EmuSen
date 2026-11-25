import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Themes' 424x265 preview card: rounded theme screenshot or board #61's #0C1030 four-tile swatch fallback (EclipseSettingsThemes:67-71).
Item {
    id: root
    required property real unit
    property real pixelScale: 1
    readonly property real radius: 22 * root.unit
    readonly property bool hasImage: Settings.previewImage.length > 0 && previewPhoto.status === Image.Ready
    objectName: "asideThemePreview"
    width: 424 * root.unit
    height: 265 * root.unit
    Ink {
        id: ink
    }
    SettingsInk {
        id: own
    }
    // Outer 2 px rim matching rgba(246, 231, 216, 0.5) (EclipseSettingsThemes:67).
    Glow {
        objectName: "asideThemePreviewRim"
        anchors.fill: parent
        radius: root.radius
        color: ink.resumeRim
        spread: 2 * root.unit
        pixelScale: root.pixelScale
    }
    // Stylized four-tile swatch card drawn when the theme has no preview PNG.
    Rectangle {
        objectName: "asideThemeSwatch"
        anchors.fill: parent
        radius: root.radius
        color: own.themePreviewGround
        visible: !root.hasImage
        Rectangle {
            objectName: "asideThemeBar0"
            x: 40 * root.unit
            y: 44 * root.unit
            width: 150 * root.unit
            height: 20 * root.unit
            radius: 10 * root.unit
            color: Tokens.text
        }
        Rectangle {
            objectName: "asideThemeBar1"
            x: 40 * root.unit
            y: 84 * root.unit
            width: 90 * root.unit
            height: 12 * root.unit
            radius: 6 * root.unit
            color: own.unsupportedName
        }
        Row {
            objectName: "asideThemeTiles"
            x: 40 * root.unit
            y: 133 * root.unit
            spacing: 14 * root.unit
            Rectangle {
                objectName: "asideThemeTile.0"
                width: 78 * root.unit
                height: 96 * root.unit
                radius: 10 * root.unit
                color: own.swatch0
            }
            Rectangle {
                objectName: "asideThemeTile.1"
                width: 78 * root.unit
                height: 96 * root.unit
                radius: 10 * root.unit
                color: own.swatch1
            }
            Rectangle {
                objectName: "asideThemeTile.2"
                width: 78 * root.unit
                height: 96 * root.unit
                radius: 10 * root.unit
                color: own.swatch2
            }
            Rectangle {
                objectName: "asideThemeTile.3"
                width: 78 * root.unit
                height: 96 * root.unit
                radius: 10 * root.unit
                color: own.swatch3
            }
        }
    }
    // Theme screenshot masked to the card's 22 px rounded rectangle when a preview URL is provided.
    Image {
        id: previewPhoto
        visible: false
        source: Settings.previewImage
        fillMode: Image.PreserveAspectCrop
        asynchronous: false
        width: Math.ceil(root.width * root.pixelScale)
        height: Math.ceil(root.height * root.pixelScale)
        sourceSize.width: width
        sourceSize.height: height
    }
    ShaderEffectSource {
        id: previewSource
        sourceItem: root.hasImage ? previewPhoto : null
        hideSource: false
        visible: false
        textureSize: Qt.size(Math.max(1, previewPhoto.width), Math.max(1, previewPhoto.height))
    }
    Shape {
        objectName: "asideThemeImage"
        anchors.fill: parent
        visible: root.hasImage
        preferredRendererType: Shape.CurveRenderer
        ShapePath {
            strokeColor: "transparent"
            fillColor: "transparent"
            fillItem: previewSource
            fillTransform: PlanarTransform.fromScale(root.width / Math.max(1, previewSource.textureSize.width), root.height / Math.max(1, previewSource.textureSize.height))
            PathRectangle {
                width: root.width
                height: root.height
                radius: root.radius
            }
        }
    }
}
