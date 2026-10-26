# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettings-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Graphics Settings"
case GraphicsSettings
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
        13 TabItem at=390,210,77,48 by=clip drawn focus heard=SNES limit=274,210,733,392 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          14 TextBlock at=402,216,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=390,210,77,48 owner=13 paints=#EEEEF0FF role=text size=30 text=SNES
        15 TabItem at=467,210,64,48 by=clip drawn focus heard=N64 limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text=N64
          16 TextBlock at=479,216,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=467,210,64,48 owner=15 paints=#86868AFF role=text size=30 text=N64
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
        27 TabItem at=539,258,174,48 by=clip drawn focus heard="PLAYSTATION 2" limit=274,210,733,392 paints=#86868AFF role=tabitem sideways text="PlayStation 2"
          28 TextBlock at=551,264,150,36 box=150,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=149.97,36 limit=539,258,174,48 owner=27 paints=#86868AFF role=text size=30 text="PLAYSTATION 2"
        29 ScrollViewer at=286,308,709,294 by=clip fades=b limit=274,308,733,294 overflow=245 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=286,308,709,294
          30 StackPanel at=286,316,709,531 by=scroll in=29 limit=286,308,709,294 role=none seen=286,316,709,286
            31 FieldRow at=286,316,709,129 by=scroll heard="Shader and shape" in=29 limit=286,308,709,294 paints=#9C9CA0FF role=group
              32 TextBlock at=286,316,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Shader and shape" in=29 layout=145.94,28.8 limit=286,316,709,129 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Shader and shape"
              33 HintText at=286,349,709,48 box=709,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square." in=29 layout=699.98,48 limit=286,316,709,129 paints=#86868AFF part role=text size=20 text="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square."
              34 DockPanel at=286,401,709,44 by=clip heard="Shader and shape" in=29 limit=286,316,709,129 role=none
                35 Button at=286,401,128,44 by=clip drawn focus heard=SHADERS... in=29 limit=286,316,709,129 name=SNES.Shaders op paints=#55555AFF,#9C9CA0FF role=button text=Shaders...
                  36 TextBlock at=303,407,94,32 box=94,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SHADERS... in=29 layout=93.96,31.2 limit=286,401,128,44 owner=35 paints=#EEEEF0FF role=text size=26 text=SHADERS...
                37 Dropdown at=870,401,125,44 by=clip drawn focus heard="Shader and shape" in=29 limit=286,316,709,129 name=SNES.PictureShape op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  38 TextBlock at=882,406,81,32 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="TV shape" in=29 layout=80.81,31.2 limit=882,406,81,32 owner=37 paints=#9C9CA0FF role=text size=26 text="TV shape"
                  39 ViewboxContainer at=973,419.73,12,6.54 by=clip in=29 limit=973,419.73,12,6.54 nohit role=none
                40 TextBlock at=426,409,432,29 box=432,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=41.74,28.8 limit=286,316,709,129 name=SNES.ShaderInUse paints=#9C9CA0FF role=text size=24 text=None
            41 FieldRow at=286,457,709,105 by=scroll heard=Overscan in=29 limit=286,308,709,294 paints=#9C9CA0FF role=group
              42 TextBlock at=286,457,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Overscan in=29 layout=75.29,28.8 limit=286,457,709,105 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Overscan
              43 HintText at=286,490,709,24 box=709,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The whole picture, the part a television showed, or a crop of your own." in=29 layout=474.92,24 limit=286,457,709,105 paints=#86868AFF part role=text size=20 text="The whole picture, the part a television showed, or a crop of your own."
              44 StackPanel at=286,518,709,44 by=clip heard=Overscan in=29 limit=286,457,709,105 role=none
                45 Dropdown at=286,518,166,44 by=clip drawn focus heard=Overscan in=29 limit=286,457,709,105 name=SNES.Overscan op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  46 TextBlock at=298,523,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Whole picture" in=29 layout=121.45,31.2 limit=298,523,122,32 owner=45 paints=#9C9CA0FF role=text size=26 text="Whole picture"
                  47 ViewboxContainer at=430,536.73,12,6.54 by=clip in=29 limit=430,536.73,12,6.54 nohit role=none
            48 FieldRow at=286,574,709,273 by=scroll heard="SA-1 catch-up" in=29 limit=286,308,709,294 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF role=group seen=286,574,709,28
              49 TextBlock at=286,574,709,29 box=709,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="SA-1 catch-up" in=29 layout=112.01,28.8 limit=286,574,709,273 name=PART_Label paints=#9C9CA0FF part role=text seen=286,574,709,28 size=24 text="SA-1 catch-up"
focus 35
edge 4 up 13
edge 4 down same
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0/0
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0/0
edge 6 down same
edge 6 left 4
edge 6 right same
edge 13 up same
edge 13 down 35
edge 35 up 13
edge 35 down 45
edge 35 left same
edge 35 right 37
edge 37 up 13
edge 37 down 45
edge 45 up 35
edge 45 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0/0 up 45
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0/0 down 4
press pageup focus="Button 'Shaders...'" tab=GB title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=SNES title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=SNES title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=SNES title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
