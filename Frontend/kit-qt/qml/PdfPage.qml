pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Pdf
import EmuSen.Kit as Kit

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
    // The page is made only for a loaded document, so its decode size never asks a file of an empty source.
    Loader {
        anchors.fill: parent
        active: document.status === PdfDocument.Ready
        sourceComponent: PdfPageImage {
            document: document
            currentFrame: root.page
            sourceSize: pagePixels.pixels
            fillMode: Image.Stretch
            asynchronous: false
            // Rendered at the pixels the page covers, through any canvas's scale (EmuSen_Kit.md §7.111).
            Kit.DrawnPixels {
                id: pagePixels
                anchors.fill: parent
            }
        }
    }
}
