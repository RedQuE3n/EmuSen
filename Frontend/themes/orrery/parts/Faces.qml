import QtQuick

// Orrery's faces from EmuSen's fonts: Barlow Condensed Medium for names and titles, Barlow for words, Barlow Condensed Light for the wordmark (orrery.md §1.3).
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
