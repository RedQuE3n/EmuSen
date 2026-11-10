import QtQuick
import QtQuick.Pdf
import EmuSen.Esde

// A page of the media viewer's PDF manual drawn to the box the scene placed, through Qt PDF; it tells the scene the page's size in points and the pages (EmuSen_Qt.md §4.13).
ElementBox {
    id: root
    required property url source
    required property real pageIndex
    PdfDocument {
        id: document
        source: root.source
        onPageCountChanged: root.esde?.manualPages(document.pageCount)
    }
    readonly property size pageSize: document.status === PdfDocument.Ready ? document.pagePointSize(root.pageIndex) : Qt.size(0, 0)
    onPageSizeChanged: root.esde?.measured(root.elementId, root.pageSize.width, root.pageSize.height)
    PdfPageImage {
        objectName: "manualPage"
        anchors.fill: parent
        document: document
        currentFrame: root.pageIndex
        sourceSize: Qt.size(root.width, root.height)
        fillMode: Image.Stretch
        asynchronous: false
    }
}
