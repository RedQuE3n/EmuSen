# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettingsNesEngine-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Graphics Settings"
case GraphicsSettingsNesEngine
entered shoulders
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 HintText at=274,152,733,48 box=733,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too." layout=709.76,48 limit=234,124,812.35,548 paints=#86868AFF role=text size=20 text="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too."
      3 ButtonBar at=689,612,318,44 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=toolbar
        4 Button at=689,612,220,44 by=clip drawn focus heard="RESET THIS CONSOLE" limit=689,612,318,44 op paints=#55555AFF,#9C9CA0FF role=button text="Reset This Console"
          5 TextBlock at=706,618,186,32 box=186,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET THIS CONSOLE" layout=185.93,31.2 limit=689,612,220,44 owner=4 paints=#9C9CA0FF role=text size=26 text="RESET THIS CONSOLE"
        6 Button at=917,612,90,44 by=clip drawn focus heard=CLOSE limit=689,612,318,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          7 TextBlock at=934,618,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=917,612,90,44 owner=6 paints=#9C9CA0FF role=text size=26 text=CLOSE
      8 Tabs at=274,210,733,392 by=panel limit=234,124,812.35,548 name=ConsoleTabs paints=#9C9CA0FF role=tab
        9 TabItem at=274,210,65,48 by=clip drawn focus heard=NES limit=274,210,733,392 op paints=#EEEEF0FF role=tabitem sideways text=NES
          10 TextBlock at=286,216,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=274,210,65,48 owner=9 paints=#EEEEF0FF role=text size=30 text=NES
        11 TabItem at=339,210,51,48 by=clip drawn focus heard=GB limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=GB
          12 TextBlock at=351,216,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=339,210,51,48 owner=11 paints=#86868AFF role=text size=30 text=GB
        13 TabItem at=390,210,77,48 by=clip drawn focus heard=SNES limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=SNES
          14 TextBlock at=402,216,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=390,210,77,48 owner=13 paints=#86868AFF role=text size=30 text=SNES
        15 TabItem at=467,210,64,48 by=clip drawn focus heard=N64 limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=N64
          16 TextBlock at=479,216,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=467,210,64,48 owner=15 paints=#86868AFF role=text size=30 text=N64
        17 TabItem at=531,210,110,48 by=clip drawn focus heard=GENESIS limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=543,216,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=531,210,110,48 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=641,210,109,48 by=clip drawn focus heard="SEGA CD" limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=653,216,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=641,210,109,48 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=750,210,156,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=762,216,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=750,210,156,48 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=286,260,709,342 by=clip fades=b limit=274,260,733,342 overflow=125 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,260,709,342
          24 StackPanel at=286,268,709,459 by=scroll in=23 limit=286,260,709,342 role=none seen=286,268,709,334
            25 FieldRow at=286,268,709,129 by=scroll heard="Shader and shape" in=23 limit=286,260,709,342 paints=#9C9CA0FF role=group
              26 TextBlock at=286,268,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Shader and shape" in=23 layout=145.94,28.8 limit=286,268,709,129 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Shader and shape"
              27 HintText at=286,301,709,48 box=709,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square." in=23 layout=699.98,48 limit=286,268,709,129 paints=#86868AFF part role=text size=20 text="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square."
              28 DockPanel at=286,353,709,44 by=clip heard="Shader and shape" in=23 limit=286,268,709,129 role=none
                29 Button at=286,353,128,44 by=clip drawn focus heard=SHADERS... in=23 limit=286,268,709,129 name=NES.Shaders op paints=#55555AFF,#9C9CA0FF role=button text=Shaders...
                  30 TextBlock at=303,359,94,32 box=94,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SHADERS... in=23 layout=93.96,31.2 limit=286,353,128,44 owner=29 paints=#EEEEF0FF role=text size=26 text=SHADERS...
                31 Dropdown at=870,353,125,44 by=clip drawn focus heard="Shader and shape" in=23 limit=286,268,709,129 name=NES.PictureShape op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  32 TextBlock at=882,358,81,32 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="TV shape" in=23 layout=80.81,31.2 limit=882,358,81,32 owner=31 paints=#9C9CA0FF role=text size=26 text="TV shape"
                  33 ViewboxContainer at=973,371.73,12,6.54 by=clip in=23 limit=973,371.73,12,6.54 nohit role=none
                34 TextBlock at=426,361,432,29 box=432,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=23 layout=41.74,28.8 limit=286,268,709,129 name=NES.ShaderInUse paints=#9C9CA0FF role=text size=24 text=None
            35 FieldRow at=286,409,709,105 by=scroll heard=Overscan in=23 limit=286,260,709,342 paints=#9C9CA0FF role=group
              36 TextBlock at=286,409,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Overscan in=23 layout=75.29,28.8 limit=286,409,709,105 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Overscan
              37 HintText at=286,442,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The whole picture, the part a television showed, or a crop of your own." in=23 layout=474.92,24 limit=286,409,709,105 paints=#86868AFF part role=text size=20 text="The whole picture, the part a television showed, or a crop of your own."
              38 StackPanel at=286,470,709,44 by=clip heard=Overscan in=23 limit=286,409,709,105 role=none
                39 Dropdown at=286,470,166,44 by=clip drawn focus heard=Overscan in=23 limit=286,409,709,105 name=NES.Overscan op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  40 TextBlock at=298,475,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Whole picture" in=23 layout=121.45,31.2 limit=298,475,122,32 owner=39 paints=#9C9CA0FF role=text size=26 text="Whole picture"
                  41 ViewboxContainer at=430,488.73,12,6.54 by=clip in=23 limit=430,488.73,12,6.54 nohit role=none
            42 FieldRow at=286,526,709,201 by=scroll heard=Region in=23 limit=286,260,709,342 paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=group seen=286,526,709,76
              43 TextBlock at=286,526,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Region in=23 layout=55.58,28.8 limit=286,526,709,201 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Region
              44 HintText at=286,559,709,120 box=709,120 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The console a game runs on, which sets its speed and the picture's rate: the NTSC NES at 60 frames a second, the PAL NES at 50, and the Dendy, a famiclone, at 50 with the NTSC NES's sound. From the cartridge, a game runs as the console its NES 2.0 header names, and an older iNES header, which has no reliable mark, as the NTSC NES. A game made for one console runs too fast, too slow or wrongly on another. Takes effect when a game is next loaded." in=23 layout=707.02,120 limit=286,526,709,201 paints=#86868AFF part role=text seen=286,559,709,43 size=20 text="The console a game runs on, which sets its speed and the picture's rate: the NTSC NES at 60 frames a second, the PAL NES at 50, and the Dendy, a famiclone, at 50 with the NTSC NES's sound. From the cartridge, a game runs as the console its NES 2.0 header names, and an older iNES header, which has no reliable mark, as the NTSC NES. A game made for one console runs too fast, too slow or wrongly on another. Takes effect when a game is next loaded."
focus 29
edge 4 up 31
edge 4 down same
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
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
edge 39 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 up 39
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 down 4
press pageup focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
