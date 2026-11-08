import QtQuick

// A stub screen of the host's tests: plain QtQuick, no EmuSen.Theme yet.
Item {
    objectName: "stubHome"
    Rectangle { objectName: "stubFill"; anchors.fill: parent; color: "#204060" }
    Image { objectName: "stubPicture"; source: "../assets/dot.png" }
}
