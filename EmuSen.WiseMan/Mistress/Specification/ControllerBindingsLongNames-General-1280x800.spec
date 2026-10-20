# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsLongNames-General-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 38.4,24,1203.2,668
title "Controller Settings"
case ControllerBindingsLongNames
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=38,124,1204,548 by=panel limit=38.4,124,1203.2,548 paints=#9C9CA0FF role=none seen=38.4,124,1203.2,548
    1 DockPanel at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
      2 Tabs at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=78,152,116,48 by=clip drawn focus heard=GENERAL limit=78,152,1124,504 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=90,158,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=78,152,116,48 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=194,152,65,48 by=clip drawn focus heard=NES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=206,158,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=194,152,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=259,152,51,48 by=clip drawn focus heard=GB limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=271,158,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=259,152,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=310,152,77,48 by=clip drawn focus heard=SNES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=322,158,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=310,152,77,48 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=387,152,64,48 by=clip drawn focus heard=N64 limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=399,158,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=387,152,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=451,152,107,48 by=clip drawn focus heard=SG-1000 limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=463,158,83,36 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=451,152,107,48 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=558,152,186,48 by=clip drawn focus heard="MASTER SYSTEM" limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=570,158,162,36 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=558,152,186,48 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=744,152,110,48 by=clip drawn focus heard=GENESIS limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=756,158,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=744,152,110,48 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=854,152,109,48 by=clip drawn focus heard="SEGA CD" limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=866,158,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=854,152,109,48 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=963,152,156,48 by=clip drawn focus heard=PLAYSTATION limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=975,158,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=963,152,156,48 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=90,202,1100,454 by=clip fades=b limit=78,202,1124,454 overflow=641 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=90,202,1100,454
          24 StackPanel at=90,210,1100,1087 by=scroll in=23 limit=90,202,1100,454 paints=#55555AFF,#6E6E6EFF,#86868AFF,#9C9CA0FF role=none seen=90,210,1100,446
            25 Grid at=90,210,1100,282 by=scroll in=23 limit=90,202,1100,454 name=GamepadSection role=none
              26 StackPanel at=90,210,516,282 by=scroll in=23 limit=90,202,1100,454 role=none
                27 TextBlock at=90,210,516,29 box=516,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Gamepad in=23 layout=76.39,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text size=24 text=Gamepad
                28 TextBlock at=90,249,516,29 box=516,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Connected: Simulated pad" in=23 layout=212.69,28.8 limit=90,202,1100,454 name=ControllerStatusText paints=#9C9CA0FF role=text size=24 text="Connected: Simulated pad"
                29 LunaSwitch at=90,290,287,32 by=scroll drawn focus heard="Use the left stick as a d-pad" in=23 limit=90,202,1100,454 name=AnalogStickAsDpadCheckBox op paints=#4C9AE8FF,#9C9CA0FF,#FFFFFFFF role=button
                  30 TextBlock at=142,292,229,29 box=229,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Use the left stick as a d-pad" in=23 layout=228.94,28.8 limit=90,290,287,32 owner=29 paints=#9C9CA0FF role=text size=24 text="Use the left stick as a d-pad"
                31 HintText at=114,328,492,48 box=492,48 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="On a console with no stick of its own, the left stick then presses the d-pad." in=23 layout=476.06,48 limit=90,202,1100,454 paints=#86868AFF role=text size=20 text="On a console with no stick of its own, the left stick then presses the d-pad."
                32 Grid at=114,388,492,50 by=scroll in=23 limit=90,202,1100,454 role=none
                  33 TextBlock at=114,398,126,29 box=126,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Stick deadzone" in=23 layout=124.01,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text size=24 text="Stick deadzone"
                  34 Slider at=250,388,201,50 by=scroll drawn focus heard="Stick deadzone" in=23 limit=90,202,1100,454 name=DeadzoneSlider op paints=#333337FF,#4C9AE8FF,#EEEEF0FF role=slider sideways
                    35 RepeatButton at=250,403,91,20 drawn in=23 name=PART_DecreaseButton paints=#4C9AE8FF part role=button
                    36 RepeatButton at=361,403,91,20 drawn in=23 name=PART_IncreaseButton paints=#333337FF,#4C9AE8FF part role=button seen=361,403,90,20
                  37 TextBlock at=461,398,41,29 box=41,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=50% in=23 layout=39.22,28.8 limit=90,202,1100,454 name=DeadzoneText paints=#9C9CA0FF role=text size=24 text=50%
                38 HintText at=114,444,492,48 box=492,48 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="How far the stick must travel before it counts as a direction; the dashed ring on each stick of the drawing. Raise it if your stick drifts." in=23 layout=487.76,48 limit=90,202,1100,454 paints=#86868AFF role=text size=20 text="How far the stick must travel before it counts as a direction; the dashed ring on each stick of the drawing. Raise it if your stick drifts."
              39 ControllerDiagram at=622,221,568,260 by=scroll heard="Your controller" in=23 limit=90,202,1100,454 name=RawDiagram role=group
            40 Separator at=90,510,1100,1 by=scroll in=23 limit=90,202,1100,454 paints=#6E6E6EFF,#9C9CA0FF role=none
            41 TextBlock at=90,525,1100,29 box=1100,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Emulator Hotkeys" in=23 layout=142.92,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text size=24 text="Emulator Hotkeys"
            42 HintText at=90,560,1100,24 box=1100,24 by=scroll caps=13.999 drawn face="Barlow Condensed" heard="Actions on the emulator itself, shared by every console. Fast Forward and Rewind apply while held; the rest fire once per press." in=23 layout=858.22,24 limit=90,202,1100,454 paints=#86868AFF role=text size=20 text="Actions on the emulator itself, shared by every console. Fast Forward and Rewind apply while held; the rest fire once per press."
            43 Grid at=90,594,1100,29 by=scroll in=23 limit=90,202,1100,454 name=HotkeyHeaderRow role=none
              44 TextBlock at=90,594,133,29 box=133,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Action in=23 layout=50.93,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text size=24 text=Action
              45 TextBlock at=235,594,78,29 box=78,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Keyboard in=23 layout=76.39,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text size=24 text=Keyboard
            46 StackPanel at=90,629,1100,444 by=scroll in=23 limit=90,202,1100,454 name=HotkeysPanel paints=#55555AFF,#9C9CA0FF role=none seen=90,629,1100,27
              47 Grid at=90,629,1100,44 by=scroll in=23 limit=90,202,1100,454 role=none seen=90,629,1100,27
                48 TextBlock at=90,637,133,29 box=133,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Fast Forward" in=23 layout=105.98,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text seen=90,637,133,19 size=24 text="Fast Forward"
                49 TextBlock at=235,637,78,29 box=78,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard=Tab in=23 layout=29.71,28.8 limit=90,202,1100,454 paints=#9C9CA0FF role=text seen=235,637,78,19 size=24 text=Tab
                50 Button at=325,629,138,44 by=scroll drawn focus heard="REBIND KEY" in=23 limit=90,202,1100,454 op paints=#55555AFF,#9C9CA0FF role=button seen=325,629,138,27 text="Rebind Key"
                  51 TextBlock at=342,635,104,32 box=104,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REBIND KEY" in=23 layout=103.97,31.2 limit=325,629,138,44 owner=50 paints=#9C9CA0FF role=text seen=342,635,104,21 size=26 text="REBIND KEY"
                52 Button at=480,629,90,44 by=scroll drawn focus heard=CLEAR in=23 limit=90,202,1100,454 op paints=#55555AFF,#9C9CA0FF role=button seen=480,629,90,27 text=Clear
                  53 TextBlock at=497,635,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLEAR in=23 layout=55.85,31.2 limit=480,629,90,44 owner=52 paints=#9C9CA0FF role=text seen=497,635,56,21 size=26 text=CLEAR
focus 3
edge 3 up same
edge 3 down 29
edge 29 up 3
edge 29 down 34
edge 29 left same
edge 29 right 3
edge 34 up 29
edge 34 down 50
edge 50 up 34
edge 50 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2
edge 50 left 3
edge 50 right 52
edge 52 up 34
edge 52 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/3
edge 52 left 50
edge 52 right 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 up 50
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/2/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 left 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/1/3
press pageup focus="DiagramLabel  (Up)" tab=PlayStation title="Controller Settings" top=InputSettingsWindow
press pagedown focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press first focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press last focus="LunaSwitch AnalogStickAsDpadCheckBox" tab=General title="Controller Settings" top=InputSettingsWindow
press back top=none
