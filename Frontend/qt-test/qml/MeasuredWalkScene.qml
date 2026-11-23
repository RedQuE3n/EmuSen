pragma ComponentBehavior: Bound

import QtQuick
import EmuSen.Kit as Kit

// Five rows of their own heights, each delegate telling the walk its height as drawn (EmuSen_Kit.md §7.122).
Window {
    id: scene
    objectName: "scene"
    visible: true
    width: 640
    height: 480
    // Each step down is one press of Down; `grown` draws the first row taller.
    property int presses: 0
    property bool grown: false
    // The row a second walk follows, as a screen binds its model's focus (1.11).
    property int followed: -1
    // Each change reads the follower's rows again, as a screen does when its model's rows change.
    property int reads: 0
    onReadsChanged: follower.read()
    onPressesChanged: walk.press(Kit.Pad.Down)
    ListModel {
        id: rows
        ListElement { kind: 1; tall: 84 }
        ListElement { kind: 1; tall: 120 }
        ListElement { kind: 1; tall: 84 }
        ListElement { kind: 1; tall: 160 }
        ListElement { kind: 1; tall: 84 }
    }
    Kit.MeasuredWalk {
        id: walk
        objectName: "walk"
        model: rows
        window: 300
        gap: 10
    }
    Kit.MeasuredWalk {
        id: follower
        objectName: "follower"
        model: rows
        window: 300
        gap: 10
        follows: scene.followed
    }
    Column {
        spacing: walk.gap
        y: -walk.offset
        Repeater {
            model: rows
            onItemAdded: (index, item) => { walk.measure(index, item.height); follower.measure(index, item.height) }
            delegate: Rectangle {
                required property int index
                required property real tall
                width: 200
                height: index === 0 && scene.grown ? 200 : tall
                color: index === walk.current ? "#ffffff" : "#404040"
                onHeightChanged: { walk.measure(index, height); follower.measure(index, height) }
            }
        }
    }
}
