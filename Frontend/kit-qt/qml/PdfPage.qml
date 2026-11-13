import QtQuick
import QtQuick.Pdf

// A page of a PDF file drawn to its box through Qt PDF, with the file's pages and the page's size in points: the media viewers' manual (EmuSen_Kit.md §7.110).
Item {
    id: root
    property url source
    property int page
    readonly property int pages: document.pageCount
    readonly property size pageSize: document.status === PdfDocument.Ready ? document.pagePointSize(root.page) : Qt.size(0, 0)
    PdfDocument {
        id: document
        source: root.source
    }
    PdfPageImage {
        anchors.fill: parent
        document: document
        currentFrame: root.page
        sourceSize: Qt.size(root.width, root.height)
        fillMode: Image.Stretch
        asynchronous: false
    }
}
