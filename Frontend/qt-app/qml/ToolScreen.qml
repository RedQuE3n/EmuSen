pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Models

// A tool on the tool screen with its own ToolModel: Cores from Home's place, or a tool of jobs (Library Manager, Compressed
// Discs or Packages) as Screens names it (EmuSen_Interface.md §15; EmuSen_Qt.md §7.7; tools/jobs.rs).
Cores {
    id: toolScreen
    required property Screens screens
    // The screen Screens shows it as: Screens.Cores or Screens.Tool.
    required property int shownAs
    required property bool splashShown
    property alias kind: toolModel.tool
    property alias jobs: toolModel.jobs
    property alias platformLibrary: toolModel.platformLibrary
    property alias cores: toolModel.cores
    property alias world: toolModel.world
    // Open Folder on a firmware page, and what failed, which the window answers.
    signal openFolder(string path)
    signal said(string words)
    visible: toolScreen.screens.shown === toolScreen.shownAs
    tool: toolModel
    inputs: visible && !toolScreen.splashShown
    onBacked: toolScreen.screens.back()
    ToolModel {
        id: toolModel
        objectName: `${toolScreen.objectName}Model`
        open: toolScreen.screens.shown === toolScreen.shownAs
        onOpenFolder: (path) => toolScreen.openFolder(path)
    }
    Connections {
        target: toolModel
        function onProblemChanged() { toolScreen.said(toolModel.problem) }
    }
}
