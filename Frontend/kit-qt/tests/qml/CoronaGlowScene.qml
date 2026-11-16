import QtQuick
import EmuSen.Kit as Kit

// Eclipse's disc at the board's own measures (Main.dc.html:25): its rim, 2 px of #F6E7D8, a glow of 44 px blur and 8 px spread
// at rgba(246, 214, 200, 0.6), and a halo of 170 px and 46 px at rgba(236, 160, 186, 0.28), over Home's ground; the disc's
// place puts its edge at x 450 on the row y 400. The values are test inputs (EmuSen_Kit.md §7.116).
Window {
    objectName: "scene"
    visible: true
    width: 1200
    height: 800
    color: "#0A1024"
    Kit.CoronaDisc {
        objectName: "disc"
        x: -950
        y: -300
        width: 1400
        height: 1400
        fill: "#07080F"
        corona: "#F6E7D8"
        rim: 2
        glows: [
            Kit.CoronaGlow {
                blur: 44
                spread: 8
                color: Qt.rgba(246 / 255, 214 / 255, 200 / 255, 0.6)
            },
            Kit.CoronaGlow {
                blur: 170
                spread: 46
                color: Qt.rgba(236 / 255, 160 / 255, 186 / 255, 0.28)
            }
        ]
    }
}
