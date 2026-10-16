# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettingsNesEngine-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,65,1219.05,974
title "Graphics Settings"
case GraphicsSettingsNesEngine
entered shoulders
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,215,1221,795 by=panel limit=350.48,215,1219.05,794 paints=#9C9CA0FF role=none seen=350.48,215,1219.05,794
    1 DockPanel at=410,257,1101,729 by=panel limit=350.48,215,1219.05,794 role=none
      2 HintText at=410,257,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too." layout=709.76,48 limit=350.48,215,1219.05,794 paints=#86868AFF role=text size=20 text="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too."
      3 ButtonBar at=1034,920,477,66 by=panel limit=350.48,215,1219.05,794 paints=#9C9CA0FF role=toolbar
        4 Button at=1034,920,330,66 by=clip drawn focus heard="RESET THIS CONSOLE" limit=1034,920,477,66 op paints=#55555AFF,#9C9CA0FF role=button text="Reset This Console"
          5 TextBlock at=1059.5,929,279,48 box=186,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET THIS CONSOLE" layout=185.93,31.2 limit=1034,920,330,66 owner=4 paints=#9C9CA0FF role=text size=26 text="RESET THIS CONSOLE"
        6 Button at=1376,920,135,66 by=clip drawn focus heard=CLOSE limit=1034,920,477,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          7 TextBlock at=1401.5,929,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,920,135,66 owner=6 paints=#9C9CA0FF role=text size=26 text=CLOSE
      8 Tabs at=410,344,1101,561 by=panel limit=350.48,215,1219.05,794 name=ConsoleTabs paints=#9C9CA0FF role=tab
        9 TabItem at=410,344,97.5,72 by=clip drawn focus heard=NES limit=410,344,1101,561 op paints=#EEEEF0FF role=tabitem sideways text=NES
          10 TextBlock at=428,353,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=410,344,97.5,72 owner=9 paints=#EEEEF0FF role=text size=30 text=NES
        11 TabItem at=507.5,344,76.5,72 by=clip drawn focus heard=GB limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text=GB
          12 TextBlock at=525.5,353,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=507.5,344,76.5,72 owner=11 paints=#86868AFF role=text size=30 text=GB
        13 TabItem at=584,344,115.5,72 by=clip drawn focus heard=SNES limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text=SNES
          14 TextBlock at=602,353,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=584,344,115.5,72 owner=13 paints=#86868AFF role=text size=30 text=SNES
        15 TabItem at=699.5,344,96,72 by=clip drawn focus heard=N64 limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text=N64
          16 TextBlock at=717.5,353,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=699.5,344,96,72 owner=15 paints=#86868AFF role=text size=30 text=N64
        17 TabItem at=795.5,344,165,72 by=clip drawn focus heard=GENESIS limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=813.5,353,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=795.5,344,165,72 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=960.5,344,163.5,72 by=clip drawn focus heard="SEGA CD" limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=978.5,353,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=960.5,344,163.5,72 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=1124,344,234,72 by=clip drawn focus heard=PLAYSTATION limit=410,344,1101,561 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=1142,353,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=1124,344,234,72 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=428,419,1065,486 by=clip limit=410,419,1101,486 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=428,419,1065,486
          24 StackPanel at=428,431,1065,474 by=fixed-scroll in=23 limit=428,419,1065,486 role=none
            25 FieldRow at=428,431,1065,193.5 by=fixed-scroll heard="Shader and shape" in=23 limit=428,419,1065,486 paints=#9C9CA0FF role=group
              26 TextBlock at=428,431,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Shader and shape" in=23 layout=145.94,28.8 limit=428,431,1065,193.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Shader and shape"
              27 HintText at=428,480.5,1065,72 box=710,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square." in=23 layout=699.98,48 limit=428,431,1065,193.5 paints=#86868AFF part role=text size=20 text="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square."
              28 DockPanel at=428,558.5,1065,66 by=clip heard="Shader and shape" in=23 limit=428,431,1065,193.5 role=none
                29 Button at=428,558.5,192,66 by=clip drawn focus heard=SHADERS... in=23 limit=428,431,1065,193.5 name=NES.Shaders op paints=#55555AFF,#9C9CA0FF role=button text=Shaders...
                  30 TextBlock at=453.5,567.5,141,48 box=94,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SHADERS... in=23 layout=93.96,31.2 limit=428,558.5,192,66 owner=29 paints=#EEEEF0FF role=text size=26 text=SHADERS...
                31 Dropdown at=1305.5,558.5,187.5,66 by=clip drawn focus heard="Shader and shape" in=23 limit=428,431,1065,193.5 name=NES.PictureShape op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  32 TextBlock at=1323.5,566,121.5,48 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="TV shape" in=23 layout=80.81,31.2 limit=1323.5,566,121.5,48 owner=31 paints=#9C9CA0FF role=text size=26 text="TV shape"
                  33 ViewboxContainer at=1460,586.6,18,9.81 by=clip in=23 limit=1460,586.6,18,9.81 nohit role=none
                34 TextBlock at=638,570.5,649.5,43.5 box=433,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=23 layout=41.74,28.8 limit=428,431,1065,193.5 name=NES.ShaderInUse paints=#9C9CA0FF role=text size=24 text=None
            35 FieldRow at=428,642.5,1065,157.5 by=fixed-scroll heard=Overscan in=23 limit=428,419,1065,486 paints=#9C9CA0FF role=group
              36 TextBlock at=428,642.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Overscan in=23 layout=75.29,28.8 limit=428,642.5,1065,157.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Overscan
              37 HintText at=428,692,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The whole picture, the part a television showed, or a crop of your own." in=23 layout=474.92,24 limit=428,642.5,1065,157.5 paints=#86868AFF part role=text size=20 text="The whole picture, the part a television showed, or a crop of your own."
              38 StackPanel at=428,734,1065,66 by=clip heard=Overscan in=23 limit=428,642.5,1065,157.5 role=none
                39 Dropdown at=428,734,249,66 by=clip drawn focus heard=Overscan in=23 limit=428,642.5,1065,157.5 name=NES.Overscan op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  40 TextBlock at=446,741.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Whole picture" in=23 layout=121.45,31.2 limit=446,741.5,183,48 owner=39 paints=#9C9CA0FF role=text size=26 text="Whole picture"
                  41 ViewboxContainer at=644,762.1,18,9.81 by=clip in=23 limit=644,762.1,18,9.81 nohit role=none
            42 EmptyState at=572,818,778.5,85.5 by=fixed-scroll heard="No core settings yet" in=23 limit=428,419,1065,486 paints=#9C9CA0FF role=text
              43 TextBlock at=839,818,244.5,43.5 box=163,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No core settings yet" in=23 layout=162.74,28.8 limit=572,818,778.5,85.5 name=PART_Message paints=#86868AFF part role=text size=24 text="No core settings yet"
              44 TextBlock at=572,867.5,778.5,36 box=519,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The NES core has nothing else to offer here; what it draws, it draws one way." in=23 layout=518.94,24 limit=572,818,778.5,85.5 name=PART_Detail paints=#86868AFF part role=text size=20 text="The NES core has nothing else to offer here; what it draws, it draws one way."
focus 29
edge 4 up 31
edge 4 down same
edge 4 left 39
edge 4 right 6
edge 6 up 31
edge 6 down same
edge 6 left 4
edge 6 right same
edge 9 up same
edge 9 down 29
edge 29 up 9
edge 29 down 39
edge 29 left same
edge 29 right 31
edge 31 up 9
edge 31 down 39
edge 39 up 29
edge 39 down 4
press pageup focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
