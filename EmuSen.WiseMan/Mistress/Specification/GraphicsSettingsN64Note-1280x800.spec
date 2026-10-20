# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettingsN64Note-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Graphics Settings"
case GraphicsSettingsN64Note
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
        9 TabItem at=274,210,65,48 by=clip drawn focus heard=NES limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=NES
          10 TextBlock at=286,216,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=274,210,65,48 owner=9 paints=#86868AFF role=text size=30 text=NES
        11 TabItem at=339,210,51,48 by=clip drawn focus heard=GB limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=GB
          12 TextBlock at=351,216,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=339,210,51,48 owner=11 paints=#86868AFF role=text size=30 text=GB
        13 TabItem at=390,210,77,48 by=clip drawn focus heard=SNES limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=SNES
          14 TextBlock at=402,216,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=390,210,77,48 owner=13 paints=#86868AFF role=text size=30 text=SNES
        15 TabItem at=467,210,64,48 by=clip drawn focus heard=N64 limit=274,210,733,392 op paints=#EEEEF0FF role=tabitem sideways text=N64
          16 TextBlock at=479,216,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=467,210,64,48 owner=15 paints=#EEEEF0FF role=text size=30 text=N64
        17 TabItem at=531,210,107,48 by=clip drawn focus heard=SG-1000 limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=SG-1000
          18 TextBlock at=543,216,83,36 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=531,210,107,48 owner=17 paints=#86868AFF role=text size=30 text=SG-1000
        19 TabItem at=638,210,186,48 by=clip drawn focus heard="MASTER SYSTEM" limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text="Master System"
          20 TextBlock at=650,216,162,36 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=638,210,186,48 owner=19 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        21 TabItem at=824,210,110,48 by=clip drawn focus heard=GENESIS limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=Genesis
          22 TextBlock at=836,216,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=824,210,110,48 owner=21 paints=#86868AFF role=text size=30 text=GENESIS
        23 TabItem at=274,258,109,48 by=clip drawn focus heard="SEGA CD" limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text="Sega CD"
          24 TextBlock at=286,264,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=274,258,109,48 owner=23 paints=#86868AFF role=text size=30 text="SEGA CD"
        25 TabItem at=383,258,156,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=PlayStation
          26 TextBlock at=395,264,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=383,258,156,48 owner=25 paints=#86868AFF role=text size=30 text=PLAYSTATION
        27 ScrollViewer at=286,308,709,294 by=clip fades=tb limit=274,308,733,294 overflow=1588 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,308,709,294
          28 StackPanel at=286,-840,709,1874 by=scroll in=27 limit=286,308,709,294 paints=#0E0E10FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#B4B4B8FF,#EEEEF0FF,#FFFFFFFF role=none seen=286,308,709,294
            29 FieldRow at=286,205,709,153 by=scroll heard="Internal resolution" in=27 limit=286,308,709,294 paints=#9C9CA0FF role=group seen=286,308,709,50
              30 HintText at=286,238,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Threads help; 2x is what most machines can hold at full speed. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen." in=27 layout=695.92,72 limit=286,205,709,153 paints=#86868AFF part role=text seen=286,308,709,2 size=20 text="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Threads help; 2x is what most machines can hold at full speed. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen."
              31 StackPanel at=286,314,709,44 by=clip heard="Internal resolution" in=27 limit=286,205,709,153 role=none
                32 Dropdown at=286,314,120,44 by=clip drawn focus heard="Internal resolution" in=27 limit=286,205,709,153 name=N64.RenderScale op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  33 TextBlock at=298,319,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3 in=27 layout=10.61,31.2 limit=298,319,76,32 owner=32 paints=#9C9CA0FF role=text size=26 text=3
                  34 ViewboxContainer at=384,332.73,12,6.54 by=clip in=27 limit=384,332.73,12,6.54 nohit role=none
            35 FieldRow at=286,370,709,181 by=scroll heard=Antialiasing in=27 limit=286,308,709,294 paints=#9C9CA0FF role=group
              36 TextBlock at=286,370,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Antialiasing in=27 layout=94.63,28.8 limit=286,370,709,181 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Antialiasing
              37 HintText at=286,403,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. With the internal resolution it is held to four: at 2x resolution the most is 2x. It multiplies the drawing's cost like the internal resolution." in=27 layout=696.34,72 limit=286,370,709,181 paints=#86868AFF part role=text size=20 text="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. With the internal resolution it is held to four: at 2x resolution the most is 2x. It multiplies the drawing's cost like the internal resolution."
              38 StackPanel at=286,479,709,72 by=clip heard=Antialiasing in=27 limit=286,370,709,181 role=none
                39 Dropdown at=286,479,120,44 by=clip drawn focus heard=Antialiasing in=27 limit=286,370,709,181 name=N64.Antialiasing op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  40 TextBlock at=298,484,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3x in=27 layout=20.38,31.2 limit=298,484,76,32 owner=39 paints=#9C9CA0FF role=text size=26 text=3x
                  41 ViewboxContainer at=384,497.73,12,6.54 by=clip in=27 limit=384,497.73,12,6.54 nohit role=none
                42 HintText at=286,527,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off at 3x resolution: the two together are held to four, so 3x has no effect." in=27 layout=499.08,24 limit=286,370,709,181 name=N64.Antialiasing.Note paints=#86868AFF role=text size=20 text="Off at 3x resolution: the two together are held to four, so 3x has no effect."
            43 FieldRow at=286,563,709,165 by=scroll heard="Draw the multiple on the graphics card" in=27 limit=286,308,709,294 paints=#86868AFF,#9C9CA0FF role=group seen=286,563,709,39
              44 TextBlock at=286,563,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Draw the multiple on the graphics card" in=27 layout=316.15,28.8 limit=286,563,709,165 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Draw the multiple on the graphics card"
              45 HintText at=286,596,709,96 box=709,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's." in=27 layout=707.46,96 limit=286,563,709,165 paints=#86868AFF part role=text seen=286,596,709,6 size=20 text="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's."
focus path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0
edge 4 up 15
edge 4 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge 6 left 4
edge 6 right same
edge 15 up same
edge 15 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0
edge 32 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0
edge 32 down 39
edge 39 up 32
edge 39 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0 up same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/1 up 15
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0 right 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0 down 32
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0 right 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0 right 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0/0
press pageup focus="Button 'Shaders...'" tab=SNES title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
