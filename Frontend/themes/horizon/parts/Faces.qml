import QtQuick

// Horizon's faces from EmuSen's fonts: Barlow Condensed Medium for names, Barlow for words, Barlow Medium for the splash's caption, Barlow Condensed Light for the wordmark (EmuSen_Qt.md §3.4.2).
Item {
    readonly property string condensed: condensedFace.font.family
    readonly property string regular: regularFace.font.family
    readonly property string medium: mediumFace.font.family
    readonly property string light: lightFace.font.family
    FontLoader {
        id: lightFace
        source: "qrc:/emusen/fonts/BarlowCondensed-Light.ttf"
    }
    FontLoader {
        id: condensedFace
        source: "qrc:/emusen/fonts/BarlowCondensed-Medium.ttf"
    }
    FontLoader {
        id: regularFace
        source: "qrc:/emusen/fonts/Barlow-Regular.ttf"
    }
    FontLoader {
        id: mediumFace
        source: "qrc:/emusen/fonts/Barlow-Medium.ttf"
    }
}
