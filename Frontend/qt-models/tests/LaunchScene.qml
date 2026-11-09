import QtQuick
import EmuSen.Models

// The harness's scene for LaunchModel: a game named prepares it, an answer set answers it, and the game it names once ready is read back (EmuSen_Qt.md §7.7).
Window {
    id: root
    objectName: "scene"
    width: 640
    height: 480
    property alias platformLibrary: launch.platformLibrary
    property alias cores: launch.cores
    property alias asking: launch.asking
    property alias title: launch.title
    property alias notice: launch.notice
    property string game
    property int answer: -1
    property string readied
    onGameChanged: if (root.game.length > 0) launch.prepare(root.game)
    onAnswerChanged: if (root.answer >= 0) launch.answer(root.answer)
    LaunchModel {
        id: launch
        onReady: (path) => root.readied = path
    }
}
