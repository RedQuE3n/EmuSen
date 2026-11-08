import QtQuick

// The screens' two faces from the kit's fonts, as the mockups set them: Barlow Condensed Medium for names, Barlow for words (EmuSen_Qt.md §7.1).
Item {
    readonly property string condensed: condensedFace.font.family
    readonly property string regular: regularFace.font.family
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
}
