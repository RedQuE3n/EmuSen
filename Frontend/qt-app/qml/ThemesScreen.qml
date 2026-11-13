pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models
import EmuSen.App

// The Themes tab, under Settings ▸ Library ▸ Themes (EmuSen_Qt.md §3.8): Use stores a theme's id and the chosen
// theme's host switches at once; an ES-DE theme's Use and its choices restyle the library at once.
Themes {
    id: screen
    required property Screens screens
    required property LibraryModel library
    required property string platformLibrary
    required property bool splashShown
    // A theme used, by its folder, and a look stored, ES-DE's or not, which the window answers.
    signal used(string folder)
    signal restyled(bool esde)
    visible: screen.screens.shown === Screens.Themes
    themes: themesModel
    inputs: visible && !screen.splashShown
    onBacked: screen.screens.back()
    onActed: (key) => screen.screens.act(key)
    ThemesModel {
        id: themesModel
        objectName: "themesModel"
        platformLibrary: screen.platformLibrary
        reference: ReferenceTheme.manifest
        referenceFolder: ReferenceTheme.folder
        open: screen.screens.shown === Screens.Themes
        onUsed: (folder) => screen.used(folder)
        // The window lets go of the start's choice of an ES-DE theme first; the library's read decides the rest.
        onRestyled: (esde) => {
            screen.restyled(esde)
            screen.library.restyle()
        }
    }
}
