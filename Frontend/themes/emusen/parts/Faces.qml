import QtQuick

// The screens' faces from EmuSen's fonts: Barlow Condensed Medium for names, Barlow for words, Barlow Condensed Light for the wordmark, Barlow Condensed regular for the worlds' counts and Home's places' lines (R-20261112-1; EmuSen_Qt.md §7.1, §3.4.1).
Item {
    readonly property string condensed: condensedFace.font.family
    readonly property string condensedRegular: condensedRegularFace.font.family
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
        id: condensedRegularFace
        source: "qrc:/emusen/fonts/BarlowCondensed-Regular.ttf"
    }
    FontLoader {
        id: regularFace
        source: "qrc:/emusen/fonts/Barlow-Regular.ttf"
    }
}
