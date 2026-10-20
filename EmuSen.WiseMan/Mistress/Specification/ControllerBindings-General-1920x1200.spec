# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindings-General-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindings
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#9C9CA0FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=118,229,174,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=136,238,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=118,229,174,72 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=292,229,97.5,72 by=clip drawn focus heard=NES limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=310,238,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=292,229,97.5,72 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=389.5,229,76.5,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=407.5,238,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=389.5,229,76.5,72 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=466,229,115.5,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=484,238,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=466,229,115.5,72 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=581.5,229,96,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=599.5,238,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=581.5,229,96,72 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=677.5,229,160.5,72 by=clip drawn focus heard=SG-1000 limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=695.5,238,124.5,54 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=677.5,229,160.5,72 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=838,229,279,72 by=clip drawn focus heard="MASTER SYSTEM" limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=856,238,243,54 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=838,229,279,72 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=1117,229,165,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=1135,238,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=1117,229,165,72 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=1282,229,163.5,72 by=clip drawn focus heard="SEGA CD" limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=1300,238,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=1282,229,163.5,72 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=1445.5,229,234,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=1463.5,238,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=1445.5,229,234,72 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=961.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
          24 StackPanel at=136,316,1650,1630.5 by=scroll in=23 limit=136,304,1650,681 paints=#55555AFF,#6E6E6EFF,#86868AFF,#9C9CA0FF role=none seen=136,316,1650,669
            25 Grid at=136,316,1650,423 by=scroll in=23 limit=136,304,1650,681 name=GamepadSection role=none
              26 StackPanel at=136,316,774,423 by=scroll in=23 limit=136,304,1650,681 role=none
                27 TextBlock at=136,316,774,43.5 box=516,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Gamepad in=23 layout=76.39,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text size=24 text=Gamepad
                28 TextBlock at=136,374.5,774,43.5 box=516,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Connected: Simulated pad" in=23 layout=212.69,28.8 limit=136,304,1650,681 name=ControllerStatusText paints=#9C9CA0FF role=text size=24 text="Connected: Simulated pad"
                29 LunaSwitch at=136,436,430.5,48 by=scroll drawn focus heard="Use the left stick as a d-pad" in=23 limit=136,304,1650,681 name=AnalogStickAsDpadCheckBox op paints=#4C9AE8FF,#9C9CA0FF,#FFFFFFFF role=button
                  30 TextBlock at=214,439,343.5,43.5 box=229,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Use the left stick as a d-pad" in=23 layout=228.94,28.8 limit=136,436,430.5,48 owner=29 paints=#9C9CA0FF role=text size=24 text="Use the left stick as a d-pad"
                31 HintText at=172,493,738,72 box=492,48 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="On a console with no stick of its own, the left stick then presses the d-pad." in=23 layout=476.06,48 limit=136,304,1650,681 paints=#86868AFF role=text size=20 text="On a console with no stick of its own, the left stick then presses the d-pad."
                32 Grid at=172,583,738,75 by=scroll in=23 limit=136,304,1650,681 role=none
                  33 TextBlock at=172,598,189,43.5 box=126,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Stick deadzone" in=23 layout=124.01,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text size=24 text="Stick deadzone"
                  34 Slider at=376,583,301.5,75 by=scroll drawn focus heard="Stick deadzone" in=23 limit=136,304,1650,681 name=DeadzoneSlider op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                    35 RepeatButton at=376,605.5,136.5,30 drawn in=23 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                    36 RepeatButton at=542.5,605.5,136.5,30 drawn in=23 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button seen=542.5,605.5,135,30
                  37 TextBlock at=692.5,598,61.5,43.5 box=41,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=50% in=23 layout=39.22,28.8 limit=136,304,1650,681 name=DeadzoneText paints=#9C9CA0FF role=text size=24 text=50%
                38 HintText at=172,667,738,72 box=492,48 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="How far the stick must travel before it counts as a direction; the dashed ring on each stick of the drawing. Raise it if your stick drifts." in=23 layout=487.76,48 limit=136,304,1650,681 paints=#86868AFF role=text size=20 text="How far the stick must travel before it counts as a direction; the dashed ring on each stick of the drawing. Raise it if your stick drifts."
              39 ControllerDiagram at=934,332.5,852,390 by=scroll heard="Your controller" in=23 limit=136,304,1650,681 name=RawDiagram role=group
            40 Separator at=136,766,1650,1.5 by=scroll in=23 limit=136,304,1650,681 paints=#6E6E6EFF,#9C9CA0FF role=none
            41 TextBlock at=136,788.5,1650,43.5 box=1100,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Emulator Hotkeys" in=23 layout=142.92,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text size=24 text="Emulator Hotkeys"
            42 HintText at=136,841,1650,36 box=1100,24 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="Actions on the emulator itself, shared by every console. Fast Forward and Rewind apply while held; the rest fire once per press." in=23 layout=858.22,24 limit=136,304,1650,681 paints=#86868AFF role=text size=20 text="Actions on the emulator itself, shared by every console. Fast Forward and Rewind apply while held; the rest fire once per press."
            43 Grid at=136,892,1650,43.5 by=scroll in=23 limit=136,304,1650,681 name=HotkeyHeaderRow role=none
              44 TextBlock at=136,892,199.5,43.5 box=133,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Action in=23 layout=50.93,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text size=24 text=Action
              45 TextBlock at=353.5,892,117,43.5 box=78,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Keyboard in=23 layout=76.39,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text size=24 text=Keyboard
            46 StackPanel at=136,944.5,1650,666 by=scroll in=23 limit=136,304,1650,681 name=HotkeysPanel paints=#55555AFF,#9C9CA0FF role=none seen=136,944.5,1650,40.5
              47 Grid at=136,944.5,1650,66 by=scroll in=23 limit=136,304,1650,681 role=none seen=136,944.5,1650,40.5
                48 TextBlock at=136,956.5,199.5,43.5 box=133,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Fast Forward" in=23 layout=105.98,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text seen=136,956.5,199.5,28.5 size=24 text="Fast Forward"
                49 TextBlock at=353.5,956.5,117,43.5 box=78,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Tab in=23 layout=29.71,28.8 limit=136,304,1650,681 paints=#9C9CA0FF role=text seen=353.5,956.5,117,28.5 size=24 text=Tab
                50 Button at=488.5,944.5,207,66 by=scroll drawn focus heard="REBIND KEY" in=23 limit=136,304,1650,681 op paints=#55555AFF,#9C9CA0FF role=button seen=488.5,944.5,207,40.5 text="Rebind Key"
                  51 TextBlock at=514,953.5,156,48 box=104,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REBIND KEY" in=23 layout=103.97,31.2 limit=488.5,944.5,207,66 owner=50 paints=#9C9CA0FF role=text seen=514,953.5,156,31.5 size=26 text="REBIND KEY"
                52 Button at=721,944.5,135,66 by=scroll drawn focus heard=CLEAR in=23 limit=136,304,1650,681 op paints=#55555AFF,#9C9CA0FF role=button seen=721,944.5,135,40.5 text=Clear
                  53 TextBlock at=746.5,953.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR in=23 layout=55.85,31.2 limit=721,944.5,135,66 owner=52 paints=#9C9CA0FF role=text seen=746.5,953.5,84,31.5 size=26 text=CLEAR
focus 3
edge 3 up same
edge 3 down 29
edge 29 up 3
edge 29 down 34
edge 29 left same
edge 29 right 50
edge 34 up 29
edge 34 down 50
edge 50 up 34
edge 50 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2
edge 50 left 29
edge 50 right 52
edge 52 up 34
edge 52 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/3
edge 52 left 50
edge 52 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 up 50
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/2/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 left 29
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/3
press pageup focus="DiagramLabel  (Up)" tab=PlayStation title="Controller Settings" top=InputSettingsWindow
press pagedown focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press first focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press last focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press back top=none
