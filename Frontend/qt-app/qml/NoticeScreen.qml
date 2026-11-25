pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Models

// Notice host: Kit.NoticeLayer under reference tokens, top-centered NoticeEclipse over Overlay when coronaRim > 0 (board #14).
Kit.NormalizedCanvas {
    id: noticeCanvas
    objectName: "noticeCanvas"
    required property Kit.NoticeModel notices
    required property NoticePresenter presenter
    property int padFamily: Kit.Families.Xbox
    readonly property bool eclipse: Kit.Tokens.coronaRim > 0
    Connections {
        target: noticeCanvas.notices
        function onWordsChanged() { noticeCanvas.presenter.sync(noticeCanvas.notices.words, noticeCanvas.notices.attention) }
        function onAttentionChanged() { noticeCanvas.presenter.sync(noticeCanvas.notices.words, noticeCanvas.notices.attention) }
        function onShownChanged() { noticeCanvas.presenter.clearWhenHidden(noticeCanvas.notices.shown) }
    }
    Connections {
        target: Kit.PadRouter
        enabled: noticeCanvas.eclipse && noticeCanvas.notices.shown && noticeCanvas.presenter.hasAction
        function onPressed(button) { noticeCanvas.presenter.handleButton(button) }
    }
    Loader {
        active: !noticeCanvas.eclipse
        anchors.fill: parent
        sourceComponent: Kit.NoticeLayer {
            model: noticeCanvas.notices
            width: noticeCanvas.width
            height: noticeCanvas.height
            labelFont: Qt.font({ family: Kit.Tokens.faceText, pixelSize: Kit.Tokens.fontBody })
        }
    }
    Item {
        id: eclipseOverlay
        parent: noticeCanvas.eclipse && Overlay.overlay ? Overlay.overlay : noticeCanvas
        anchors.fill: parent
        z: 100
        visible: noticeCanvas.eclipse
        Kit.NormalizedCanvas {
            id: eclipseCanvas
            objectName: "eclipseNoticeCanvas"
            Loader {
                active: noticeCanvas.eclipse
                width: eclipseCanvas.width
                height: eclipseCanvas.height
                sourceComponent: NoticeEclipse {
                    model: noticeCanvas.notices
                    presenter: noticeCanvas.presenter
                    padFamily: noticeCanvas.padFamily
                    pixelScale: eclipseCanvas.unit
                }
            }
        }
    }
}
