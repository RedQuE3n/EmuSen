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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 HintText at=274,152,733,48 box=733,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too." layout=680.02,48 limit=234,124,812.35,548 paints=#8D8D91FF role=text size=20 text="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too."
      3 ButtonBar at=702,612,305,44 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=toolbar
        4 Button at=702,612,210,44 by=clip drawn focus heard="RESET THIS CONSOLE" limit=702,612,305,44 op paints=#707070FF,#D4D4D4FF role=button text="Reset This Console"
          5 TextBlock at=719,618,176,32 box=176,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET THIS CONSOLE" layout=175.14,31.2 limit=702,612,210,44 owner=4 paints=#D4D4D4FF role=text size=26 text="RESET THIS CONSOLE"
        6 Button at=920,612,87,44 by=clip drawn focus heard=CLOSE limit=702,612,305,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
          7 TextBlock at=937,618,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=920,612,87,44 owner=6 paints=#D4D4D4FF role=text size=26 text=CLOSE
      8 Tabs at=274,210,733,392 by=panel limit=234,124,812.35,548 name=ConsoleTabs paints=#D4D4D4FF role=tab
        9 TabItem at=274,210,62,48 by=clip drawn focus heard=NES limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=NES
          10 TextBlock at=286,216,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=37.71,36 limit=274,210,62,48 owner=9 paints=#8D8D91FF role=text size=30 text=NES
        11 TabItem at=336,210,50,48 by=clip drawn focus heard=GB limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=GB
          12 TextBlock at=348,216,26,36 box=26,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=25.77,36 limit=336,210,50,48 owner=11 paints=#8D8D91FF role=text size=30 text=GB
        13 TabItem at=386,210,74,48 by=clip drawn focus heard=SNES limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=SNES
          14 TextBlock at=398,216,50,36 box=50,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=49.53,36 limit=386,210,74,48 owner=13 paints=#8D8D91FF role=text size=30 text=SNES
        15 TabItem at=460,210,62,48 by=clip drawn focus heard=N64 limit=274,210,733,392 op paints=#EEEEF0FF role=tabitem sideways text=N64
          16 TextBlock at=472,216,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=460,210,62,48 owner=15 paints=#EEEEF0FF role=text size=30 text=N64
        17 TabItem at=522,210,104,48 by=clip drawn focus heard=SG-1000 limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          18 TextBlock at=534,216,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=79.8,36 limit=522,210,104,48 owner=17 paints=#8D8D91FF role=text size=30 text=SG-1000
        19 TabItem at=626,210,178,48 by=clip drawn focus heard="MASTER SYSTEM" limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text="Master System"
          20 TextBlock at=638,216,154,36 box=154,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=153.69,36 limit=626,210,178,48 owner=19 paints=#8D8D91FF role=text size=30 text="MASTER SYSTEM"
        21 TabItem at=804,210,104,48 by=clip drawn focus heard=GENESIS limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=Genesis
          22 TextBlock at=816,216,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=804,210,104,48 owner=21 paints=#8D8D91FF role=text size=30 text=GENESIS
        23 TabItem at=274,258,105,48 by=clip drawn focus heard="SEGA CD" limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          24 TextBlock at=286,264,81,36 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=274,258,105,48 owner=23 paints=#8D8D91FF role=text size=30 text="SEGA CD"
        25 TabItem at=379,258,147,48 by=clip drawn focus heard=PLAYSTATION limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          26 TextBlock at=391,264,123,36 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=379,258,147,48 owner=25 paints=#8D8D91FF role=text size=30 text=PLAYSTATION
        27 TabItem at=526,258,165,48 by=clip drawn focus heard="PLAYSTATION 2" limit=274,210,733,392 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          28 TextBlock at=538,264,141,36 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=526,258,165,48 owner=27 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        29 ScrollViewer at=286,308,709,294 by=clip fades=tb limit=274,308,733,294 overflow=1588 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=286,308,709,294
          30 StackPanel at=286,-840,709,1874 by=scroll in=29 limit=286,308,709,294 paints=#0E0E10FF,#4C9AE8FF,#707070FF,#8D8D91FF,#D4D4D4FF,#EEEEF0FF,#FFFFFFFF role=none seen=286,308,709,294
            31 FieldRow at=286,205,709,153 by=scroll heard="Internal resolution" in=29 limit=286,308,709,294 paints=#D4D4D4FF role=group seen=286,308,709,50
              32 HintText at=286,238,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Threads help; 2x is what most machines can hold at full speed. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen." in=29 layout=707.32,72 limit=286,205,709,153 paints=#8D8D91FF part role=text seen=286,308,709,2 size=20 text="The picture drawn at a multiple of the console's, beside the exact drawing games read back. Threads help; 2x is what most machines can hold at full speed. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen."
              33 StackPanel at=286,314,709,44 by=clip heard="Internal resolution" in=29 limit=286,205,709,153 role=none
                34 Dropdown at=286,314,120,44 by=clip drawn focus heard="Internal resolution" in=29 limit=286,205,709,153 name=N64.RenderScale op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=combobox sideways
                  35 TextBlock at=298,319,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3 in=29 layout=10.27,31.2 limit=298,319,76,32 owner=34 paints=#D4D4D4FF role=text size=26 text=3
                  36 ViewboxContainer at=384,332.73,12,6.54 by=clip in=29 limit=384,332.73,12,6.54 nohit role=none
            37 FieldRow at=286,370,709,181 by=scroll heard=Antialiasing in=29 limit=286,308,709,294 paints=#D4D4D4FF role=group
              38 TextBlock at=286,370,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Antialiasing in=29 layout=88.92,28.8 limit=286,370,709,181 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Antialiasing
              39 HintText at=286,403,709,72 box=709,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. With the internal resolution it is held to four: at 2x resolution the most is 2x. It multiplies the drawing's cost like the internal resolution." in=29 layout=681.44,72 limit=286,370,709,181 paints=#8D8D91FF part role=text size=20 text="Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. With the internal resolution it is held to four: at 2x resolution the most is 2x. It multiplies the drawing's cost like the internal resolution."
              40 StackPanel at=286,479,709,72 by=clip heard=Antialiasing in=29 limit=286,370,709,181 role=none
                41 Dropdown at=286,479,120,44 by=clip drawn focus heard=Antialiasing in=29 limit=286,370,709,181 name=N64.Antialiasing op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=combobox sideways
                  42 TextBlock at=298,484,76,32 box=76,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=3x in=29 layout=19.68,31.2 limit=298,484,76,32 owner=41 paints=#D4D4D4FF role=text size=26 text=3x
                  43 ViewboxContainer at=384,497.73,12,6.54 by=clip in=29 limit=384,497.73,12,6.54 nohit role=none
                44 HintText at=286,527,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off at 3x resolution: the two together are held to four, so 3x has no effect." in=29 layout=473.9,24 limit=286,370,709,181 name=N64.Antialiasing.Note paints=#8D8D91FF role=text size=20 text="Off at 3x resolution: the two together are held to four, so 3x has no effect."
            45 FieldRow at=286,563,709,165 by=scroll heard="Draw the multiple on the graphics card" in=29 limit=286,308,709,294 paints=#8D8D91FF,#D4D4D4FF role=group seen=286,563,709,39
              46 TextBlock at=286,563,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Draw the multiple on the graphics card" in=29 layout=302.38,28.8 limit=286,563,709,165 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Draw the multiple on the graphics card"
              47 HintText at=286,596,709,96 box=709,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's." in=29 layout=704.74,96 limit=286,563,709,165 paints=#8D8D91FF part role=text seen=286,596,709,6 size=20 text="The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's."
focus path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0
edge 4 up 15
edge 4 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0/0
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0/0
edge 6 left 4
edge 6 right same
edge 15 up same
edge 15 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/0/0/2/0/0
edge 34 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0
edge 34 down 41
edge 41 up 34
edge 41 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0/0
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
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0/0 down 34
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
