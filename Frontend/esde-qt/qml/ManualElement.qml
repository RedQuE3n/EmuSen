import QtQuick
import EmuSen.Kit as Kit
import EmuSen.Esde

// A page of the media viewer's PDF manual drawn to the box the scene placed, through the kit's page of Qt PDF; it tells the scene the page's size in points and the pages (EmuSen_Qt.md §4.13).
ElementBox {
    id: root
    required property url source
    required property real pageIndex
    readonly property size pageSize: manual.pageSize
    onPageSizeChanged: root.esde?.measured(root.elementId, root.pageSize.width, root.pageSize.height)
    Kit.PdfPage {
        id: manual
        objectName: "manualPage"
        anchors.fill: parent
        source: root.source
        page: root.pageIndex
        onPagesChanged: root.esde?.manualPages(manual.pages)
    }
}
