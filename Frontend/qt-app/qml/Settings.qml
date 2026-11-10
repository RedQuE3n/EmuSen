pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit
import EmuSen.Models

// Settings on the kit's settings frame: the six sections, the rows of the section or page shown, and the focused row's pane with its preview; every word and value is SettingsModel's (EmuSen_Qt.md §7.7).
Kit.NormalizedCanvas {
    id: screen
    required property SettingsModel settings
    property int padFamily
    // Whether it takes the keys: while it is shown and the splash has gone.
    property bool inputs: true
    // B on a section, and the declared actions the screens answer (Start, Home).
    signal backed()
    signal acted(string key)
    Connections {
        target: screen.settings
        function onLeft() { screen.backed() }
        // B closed a page: the row that opened it is focused again, as Mistress's pages return.
        function onRefocus(row) { frame.layout.focusRow(row) }
        // A declared action the page did not take is the screens' (Start, Home).
        function onPassed(key) { screen.acted(key) }
        function onTypeWords(start) { typist.setText(start, start.length) }
    }
    // The shoulders the frame declines under a trail: on a console's page, the console before or after it; on the
    // bindings page, the console's controller before or after it.
    Keys.onPressed: (event) => {
        if (event.key === Qt.Key_PageUp || event.key === Qt.Key_PageDown)
            event.accepted = screen.settings.shoulder(event.key === Qt.Key_PageDown ? 1 : -1)
    }
    Kit.SettingsFrame {
        id: frame
        objectName: "settingsFrame"
        anchors.fill: parent
        form: Kit.Forms.BigScreen
        focus: screen.inputs && !screen.typing && !listener.focus
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        model: screen.settings
        sections: screen.settings.sections
        section: screen.settings.section
        trail: screen.settings.trail
        sentenceTone: screen.settings.sentenceTone
        help: HelpModel {
            screen: screen.settings.help
        }
        pageShown: screen.settings.bindingsShown
        pageSentence: screen.settings.sentence
        onSectionChosen: (section) => screen.settings.section = section
        onStepped: (row, by) => screen.settings.step(row, by)
        onChosen: (row) => screen.settings.choose(row)
        preview: Item {
            objectName: "settingsPreview"
            width: parent ? parent.width : 0
            // Read from the model, not from the items' visible, which is false while the slot is empty and so would keep it so.
            height: screen.settings.preview === 1 ? picture.height : screen.settings.preview === 2 ? meter.height : screen.settings.preview === 3 ? variants.height : screen.settings.preview === 4 ? sizes.height : screen.settings.preview === 5 ? heading.height : 0
            // The bindings page's focused control, large, as Mistress's pane begins (EmuSen_Qt.md §7.7.H).
            Text {
                id: heading
                objectName: "SettingsLine"
                visible: screen.settings.preview === 5
                width: parent.width
                text: screen.settings.caption
                wrapMode: Text.Wrap
                color: Kit.Tokens.text
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontTitle
            }
            // The game's picture, named by its caption; the frame itself is drawn from piece J on (EmuSen_Qt.md §7.7).
            Rectangle {
                id: picture
                objectName: "SettingsPicture"
                visible: screen.settings.preview === 1
                width: parent.width
                height: parent.width * 3 / 4
                color: Kit.Tokens.voidFill
                Text {
                    objectName: "SettingsCaption"
                    anchors.centerIn: parent
                    width: Math.min(implicitWidth, parent.width - Kit.Tokens.base * 8)
                    horizontalAlignment: Text.AlignHCenter
                    wrapMode: Text.Wrap
                    text: screen.settings.caption
                    color: Kit.Tokens.overArt
                    font.family: faces.condensed
                    font.pixelSize: Kit.Tokens.fontTitle
                }
            }
            Rectangle {
                id: meter
                objectName: "SettingsMeter"
                visible: screen.settings.preview === 2
                width: parent.width
                height: Kit.Tokens.base * 4
                radius: meter.height / 2
                color: Kit.Tokens.inputSurface
                Rectangle {
                    width: meter.width * screen.settings.meter
                    height: meter.height
                    radius: meter.radius
                    color: Kit.Tokens.accent
                }
            }
            Row {
                id: variants
                objectName: "SettingsVariants"
                visible: screen.settings.preview === 3
                spacing: Kit.Tokens.base * 4
                Repeater {
                    model: screen.settings.samples
                    delegate: Rectangle {
                        id: tile
                        required property string modelData
                        required property int index
                        width: (variants.parent.width - variants.spacing) / 2
                        height: tile.width * 10 / 11
                        radius: Kit.Tokens.tileRadius
                        color: tile.index === 0 ? Kit.Tokens.surface : Kit.Tokens.inputSurface
                        border.width: 1
                        border.color: Kit.Tokens.border
                        Text {
                            x: Kit.Tokens.base * 1.5
                            y: Kit.Tokens.base * 1.5
                            text: tile.modelData
                            color: Kit.Tokens.text
                            font.family: faces.regular
                            font.pixelSize: Kit.Tokens.fontCaption
                        }
                    }
                }
            }
            Column {
                id: sizes
                objectName: "SettingsSizes"
                visible: screen.settings.preview === 4
                width: parent.width
                spacing: frame.layout.lineGap
                Repeater {
                    model: screen.settings.samples
                    delegate: Text {
                        id: sample
                        objectName: "SettingsLine"
                        required property string modelData
                        required property int index
                        width: sizes.width
                        text: sample.modelData
                        elide: Text.ElideRight
                        color: Kit.Tokens.text
                        font.family: faces.regular
                        font.pixelSize: Kit.Tokens.fontBody / Kit.Tokens.textScale * screen.settings.sampleScale(sample.index)
                    }
                }
            }
        }
        lines: Column {
            objectName: "settingsPaneLines"
            width: parent ? parent.width : 0
            spacing: frame.layout.lineGap
            Repeater {
                model: screen.settings.paneLines
                delegate: Kit.ToneText {
                    id: line
                    objectName: "SettingsLine"
                    required property string modelData
                    required property int index
                    width: parent.width
                    text: line.modelData
                    wrapMode: Text.Wrap
                    tone: screen.settings.lineTone(line.index)
                    font.family: faces.regular
                    font.pixelSize: Kit.Tokens.fontBody
                }
            }
            Repeater {
                model: screen.settings.paneMuted
                delegate: Text {
                    id: muted
                    objectName: "SettingsLine"
                    required property string modelData
                    width: parent.width
                    text: muted.modelData
                    wrapMode: Text.Wrap
                    color: Kit.Tokens.muted
                    font.family: faces.regular
                    font.pixelSize: Kit.Tokens.fontBody
                }
            }
        }
        Binding {
            target: screen.settings
            property: "focused"
            value: frame.currentRow
        }
        Connections {
            target: frame.controller
            function onActionInvoked(key) { screen.settings.act(key) }
        }
        // The bindings page (EmuSen_Input.md §10.6): the strip of the player and the consoles, the controller drawn with
        // each control's binding, and the caption under it; Left and Right walk the controls and A listens.
        pageContent: FocusScope {
            id: bindings
            objectName: "bindingsPage"
            width: parent ? parent.width : 0
            height: parent ? parent.height : 0
            focus: screen.settings.bindingsShown
            // The page holds the focus, a control's label the pointer's: a click on the page keeps the focus here.
            TapHandler {}
            Keys.onPressed: (event) => {
                if (event.key === Qt.Key_Left || event.key === Qt.Key_Right) {
                    screen.settings.bindingStep(event.key === Qt.Key_Right ? 1 : -1)
                    event.accepted = true
                } else if (event.key === Qt.Key_Return || event.key === Qt.Key_Enter) {
                    screen.settings.bindingAccept()
                    event.accepted = true
                }
            }
            Row {
                id: tabs
                objectName: "SettingsContentTabs"
                width: parent.width
                height: frame.layout.stripHeight
                Accessible.role: Accessible.PageTabList
                spacing: frame.layout.stripGap
                Repeater {
                    model: screen.settings.bindingTabs
                    delegate: TabButton {
                        id: tab
                        required property string modelData
                        required property int index
                        width: tab.implicitContentWidth + tab.leftPadding + tab.rightPadding
                        height: frame.layout.stripHeight
                        text: tab.modelData.toUpperCase()
                        font.pixelSize: Kit.Tokens.fontCaption
                        horizontalPadding: frame.layout.tabPadding
                        focusPolicy: Qt.NoFocus
                        checkable: false
                        checked: tab.index === 1
                        Accessible.name: tab.modelData
                        // The strip holds whole names only, the console shown first after the player, as Mistress's
                        // WholeRow does: a name that would run past the page is not drawn (EmuSen_Qt.md §7.7.H).
                        opacity: tab.x + tab.width <= bindings.width ? 1 : 0
                        enabled: tab.opacity > 0
                    }
                }
            }
            Kit.ControllerDrawing {
                id: drawing
                objectName: "BindingsDiagram"
                y: tabs.height
                width: parent.width
                height: 380
                drawing: screen.settings.drawing
                controls: screen.settings.drawingControls
                texts: screen.settings.drawingTexts
                focused: screen.settings.drawingFocus
                held: screen.settings.drawingHeld
                pushes: screen.settings.drawingPushes
                onChosen: (index) => screen.settings.bindingFocus(index)
            }
            Text {
                objectName: "SettingsContentCaption"
                y: drawing.y + drawing.height + 12
                width: parent.width
                text: screen.settings.bindingCaption
                wrapMode: Text.Wrap
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontBody
            }
        }
        Connections {
            target: frame.controller
            function onBackDeclined() { screen.settings.back() }
        }
    }
    // A key while a binding of the keyboard or a hotkey listens: the press listened for, Escape or the pad's B letting
    // it go (EmuSen_Input.md §10.6); while the tester runs, the keys held, and Escape stopping it. The frame hears
    // nothing meanwhile.
    FocusScope {
        id: listener
        objectName: "settingsListener"
        focus: screen.inputs && (screen.settings.testing || screen.settings.listening && !screen.settings.listeningPad)
        Keys.onPressed: (event) => event.accepted = screen.settings.heardKey(event.key, event.modifiers)
        Keys.onReleased: (event) => event.accepted = !event.isAutoRepeat && screen.settings.releasedKey(event.key, event.modifiers)
        Connections {
            target: Kit.PadRouter
            enabled: listener.activeFocus
            function onPressed(button) { screen.settings.padPressed(button) }
        }
        Timer {
            interval: 8
            repeat: true
            running: listener.activeFocus && Kit.PadRouter.clockRunning && Kit.PadRouter.pending
            onTriggered: Kit.PadRouter.pump()
        }
    }
    // B held for the tester's hold leaves it, since a short press of B is a button being tried.
    Timer {
        objectName: "testerHold"
        interval: screen.settings.testerHoldMs
        running: screen.settings.testing && screen.settings.testerBack
        onTriggered: screen.settings.back()
    }
    // Nothing pressed on the pad in five seconds leaves the binding as it was (Mistress's words on the pane).
    Timer {
        interval: 5000
        running: screen.settings.listeningPad
        onTriggered: screen.settings.back()
    }
    // A profile's name typed on the docked keyboard, as the page asks (EmuSen_Qt.md §7.7.H): Done gives it to the page.
    Kit.KeyboardModel {
        id: typist
        onClosed: screen.settings.typed(typist.text)
    }
    readonly property bool typing: screen.settings.typing
    Kit.Frame {
        id: typeFrame
        objectName: "settingsTyping"
        anchors.fill: parent
        visible: screen.typing
        focus: screen.inputs && screen.typing
        padFamily: screen.padFamily
        helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
        helpSpacing: 40
        helpGlyphGap: 10
        controller.navigationShown: false
        help: HelpModel {
            screen: HelpModel.ToolSearch
        }
        Connections {
            target: typeFrame.controller
            function onActionInvoked(key) { typist.act(key) }
        }
        Column {
            spacing: frame.layout.lineGap
            Text {
                text: screen.settings.typingTitle
                color: Kit.Tokens.textStrong
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontTitle
            }
            Text {
                text: screen.settings.typingPrompt
                color: Kit.Tokens.muted
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontBody
            }
            Text {
                objectName: "settingsTyped"
                text: typist.text
                color: Kit.Tokens.text
                font.family: faces.regular
                font.pixelSize: Kit.Tokens.fontRow
            }
        }
        Item {
            x: -typeFrame.safeX
            y: -typeFrame.safeY
            width: screen.width
            height: screen.height
            Kit.Keyboard {
                objectName: "settingsKeyboard"
                model: typist
                y: 398
                width: screen.width
                focus: true
                labelFont: Qt.font({ family: faces.regular, pixelSize: Kit.Tokens.fontBody })
            }
        }
    }
    Faces {
        id: faces
    }
}
