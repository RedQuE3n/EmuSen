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
entered shoulders
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
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
        17 TabItem at=531,210,110,48 by=clip drawn focus heard=GENESIS limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=543,216,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=531,210,110,48 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=641,210,156,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=PlayStation
          20 TextBlock at=653,216,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=641,210,156,48 owner=19 paints=#86868AFF role=text size=30 text=PLAYSTATION
        21 ScrollViewer at=286,260,709,342 by=clip fades=tb limit=274,260,733,342 overflow=1341 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,260,709,342
          22 StackPanel at=286,-803,709,1675 by=scroll in=21 limit=286,260,709,342 paints=#0E0E10FF,#4C9AE8FF,#55555AFF,#86868AFF,#9C9CA0FF,#FFFFFFFF role=none seen=286,260,709,342
            23 FieldRow at=286,196,709,153 by=scroll heard="Internal resolution" in=21 limit=286,260,709,342 paints=#9C9CA0FF role=group seen=286,260,709,89
              24 HintText at=286,229,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen. Threads help; 2x is what most machines can hold at full speed." in=21 layout=694.68,72 limit=286,196,709,153 paints=#86868AFF part role=text seen=286,260,709,41 size=20 text="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen. Threads help; 2x is what most machines can hold at full speed."
              25 Dropdown at=286,305,120,44 by=clip drawn focus heard="Internal resolution" in=21 limit=286,196,709,153 name=N64.RenderScale op paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#FFFFFFCC,#FFFFFFFF role=combobox sideways
                26 TextBlock at=298,310,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3 in=21 layout=10.61,31.2 limit=298,310,76,32 owner=25 paints=#FFFFFFFF role=text size=26 text=3
                27 ViewboxContainer at=384,323.73,12,6.54 by=clip in=21 limit=384,323.73,12,6.54 nohit role=none
            28 FieldRow at=286,361,709,181 by=scroll heard=Antialiasing in=21 limit=286,260,709,342 paints=#9C9CA0FF role=group
              29 TextBlock at=286,361,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Antialiasing in=21 layout=94.63,28.8 limit=286,361,709,181 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Antialiasing
              30 HintText at=286,394,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. It multiplies the drawing's cost like the internal resolution, and the two together are held to four: at 2x resolution the most is 2x." in=21 layout=698.72,72 limit=286,361,709,181 paints=#86868AFF part role=text size=20 text="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. It multiplies the drawing's cost like the internal resolution, and the two together are held to four: at 2x resolution the most is 2x."
              31 StackPanel at=286,470,709,72 by=clip heard=Antialiasing in=21 limit=286,361,709,181 role=none
                32 Dropdown at=286,470,120,44 by=clip drawn focus in=21 limit=286,361,709,181 name=N64.Antialiasing op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  33 TextBlock at=298,475,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3x in=21 layout=20.38,31.2 limit=298,475,76,32 owner=32 paints=#9C9CA0FF role=text size=26 text=3x
                  34 ViewboxContainer at=384,488.73,12,6.54 by=clip in=21 limit=384,488.73,12,6.54 nohit role=none
                35 HintText at=286,518,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off at 3x resolution: the two together are held to four, so 3x has no effect." in=21 layout=499.08,24 limit=286,361,709,181 name=N64.Antialiasing.Note paints=#86868AFF role=text size=20 text="Off at 3x resolution: the two together are held to four, so 3x has no effect."
            36 FieldRow at=286,554,709,165 by=scroll heard="Draw the multiple on the graphics card" in=21 limit=286,260,709,342 paints=#86868AFF,#9C9CA0FF role=group seen=286,554,709,48
              37 TextBlock at=286,554,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Draw the multiple on the graphics card" in=21 layout=316.15,28.8 limit=286,554,709,165 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Draw the multiple on the graphics card"
              38 HintText at=286,587,709,96 box=709,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's." in=21 layout=707.46,96 limit=286,554,709,165 paints=#86868AFF part role=text seen=286,587,709,15 size=20 text="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's."
focus 25
edge 4 up 15
edge 4 down same
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0
edge 6 down same
edge 6 left 4
edge 6 right same
edge 15 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0
edge 15 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0
edge 25 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0
edge 25 down 32
edge 32 up 25
edge 32 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 up 32
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 right 15
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 down 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 right 15
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 right 15
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 right 15
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 down 25
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 right 15
press pageup focus="Button 'Shaders...'" tab=SNES title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Dropdown N64.RenderScale" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Dropdown N64.RenderScale" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Dropdown N64.RenderScale" tab=N64 title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
