# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettingsCustomCrop-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Graphics Settings"
case GraphicsSettingsCustomCrop
entered shoulders
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 HintText at=410,229,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too." layout=709.76,48 limit=350.48,187,1219.05,822 paints=#86868AFF role=text size=20 text="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too."
      3 ButtonBar at=1034,919,477,66 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=toolbar
        4 Button at=1034,919,330,66 by=clip drawn focus heard="RESET THIS CONSOLE" limit=1034,919,477,66 op paints=#55555AFF,#9C9CA0FF role=button text="Reset This Console"
          5 TextBlock at=1059.5,928,279,48 box=186,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET THIS CONSOLE" layout=185.93,31.2 limit=1034,919,330,66 owner=4 paints=#9C9CA0FF role=text size=26 text="RESET THIS CONSOLE"
        6 Button at=1376,919,135,66 by=clip drawn focus heard=CLOSE limit=1034,919,477,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          7 TextBlock at=1401.5,928,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,919,135,66 owner=6 paints=#9C9CA0FF role=text size=26 text=CLOSE
      8 Tabs at=410,316,1101,588 by=panel limit=350.48,187,1219.05,822 name=ConsoleTabs paints=#9C9CA0FF role=tab
        9 TabItem at=410,316,97.5,72 by=clip drawn focus heard=NES limit=410,316,1101,588 op paints=#EEEEF0FF role=tabitem sideways text=NES
          10 TextBlock at=428,325,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=410,316,97.5,72 owner=9 paints=#EEEEF0FF role=text size=30 text=NES
        11 TabItem at=507.5,316,76.5,72 by=clip drawn focus heard=GB limit=410,316,1101,588 paints=#86868AFF role=tabitem sideways text=GB
          12 TextBlock at=525.5,325,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=507.5,316,76.5,72 owner=11 paints=#86868AFF role=text size=30 text=GB
        13 TabItem at=584,316,115.5,72 by=clip drawn focus heard=SNES limit=410,316,1101,588 paints=#86868AFF role=tabitem sideways text=SNES
          14 TextBlock at=602,325,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=584,316,115.5,72 owner=13 paints=#86868AFF role=text size=30 text=SNES
        15 TabItem at=699.5,316,96,72 by=clip drawn focus heard=N64 limit=410,316,1101,588 paints=#86868AFF role=tabitem sideways text=N64
          16 TextBlock at=717.5,325,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=699.5,316,96,72 owner=15 paints=#86868AFF role=text size=30 text=N64
        17 TabItem at=795.5,316,165,72 by=clip drawn focus heard=GENESIS limit=410,316,1101,588 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=813.5,325,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=795.5,316,165,72 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=960.5,316,234,72 by=clip drawn focus heard=PLAYSTATION limit=410,316,1101,588 paints=#86868AFF role=tabitem sideways text=PlayStation
          20 TextBlock at=978.5,325,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=960.5,316,234,72 owner=19 paints=#86868AFF role=text size=30 text=PLAYSTATION
        21 ScrollViewer at=428,391,1065,513 by=clip fades=b limit=410,391,1101,513 overflow=505.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=428,391,1065,513
          22 StackPanel at=428,403,1065,1006.5 by=scroll in=21 limit=428,391,1065,513 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF role=none seen=428,403,1065,501
            23 FieldRow at=428,403,1065,193.5 by=scroll heard="Shader and shape" in=21 limit=428,391,1065,513 paints=#9C9CA0FF role=group
              24 TextBlock at=428,403,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Shader and shape" in=21 layout=145.94,28.8 limit=428,403,1065,193.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Shader and shape"
              25 HintText at=428,452.5,1065,72 box=710,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square." in=21 layout=699.98,48 limit=428,403,1065,193.5 paints=#86868AFF part role=text size=20 text="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square."
              26 DockPanel at=428,530.5,1065,66 by=clip heard="Shader and shape" in=21 limit=428,403,1065,193.5 role=none
                27 Button at=428,530.5,192,66 by=clip drawn focus heard=SHADERS... in=21 limit=428,403,1065,193.5 name=NES.Shaders op paints=#55555AFF,#9C9CA0FF role=button text=Shaders...
                  28 TextBlock at=453.5,539.5,141,48 box=94,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SHADERS... in=21 layout=93.96,31.2 limit=428,530.5,192,66 owner=27 paints=#EEEEF0FF role=text size=26 text=SHADERS...
                29 Dropdown at=1305.5,530.5,187.5,66 by=clip drawn focus heard="Shader and shape" in=21 limit=428,403,1065,193.5 name=NES.PictureShape op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  30 TextBlock at=1323.5,538,121.5,48 box=81,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="TV shape" in=21 layout=80.81,31.2 limit=1323.5,538,121.5,48 owner=29 paints=#9C9CA0FF role=text size=26 text="TV shape"
                  31 ViewboxContainer at=1460,558.6,18,9.81 by=clip in=21 limit=1460,558.6,18,9.81 nohit role=none
                32 TextBlock at=638,542.5,649.5,43.5 box=433,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=21 layout=41.74,28.8 limit=428,403,1065,193.5 name=NES.ShaderInUse paints=#9C9CA0FF role=text size=24 text=None
            33 FieldRow at=428,614.5,1065,157.5 by=scroll heard=Overscan in=21 limit=428,391,1065,513 paints=#9C9CA0FF role=group
              34 TextBlock at=428,614.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Overscan in=21 layout=75.29,28.8 limit=428,614.5,1065,157.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Overscan
              35 HintText at=428,664,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The whole picture, the part a television showed, or a crop of your own." in=21 layout=474.92,24 limit=428,614.5,1065,157.5 paints=#86868AFF part role=text size=20 text="The whole picture, the part a television showed, or a crop of your own."
              36 StackPanel at=428,706,1065,66 by=clip heard=Overscan in=21 limit=428,614.5,1065,157.5 role=none
                37 Dropdown at=428,706,234,66 by=clip drawn focus heard=Overscan in=21 limit=428,614.5,1065,157.5 name=NES.Overscan op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
                  38 TextBlock at=446,713.5,168,48 box=112,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Custom crop" in=21 layout=111.98,31.2 limit=446,713.5,168,48 owner=37 paints=#9C9CA0FF role=text size=26 text="Custom crop"
                  39 ViewboxContainer at=629,734.1,18,9.81 by=clip in=21 limit=629,734.1,18,9.81 nohit role=none
            40 FieldRow at=428,790,1065,196.5 by=scroll heard="Crop, % of the picture" in=21 limit=428,391,1065,513 name=NES.Crop paints=#0E0E10FF,#55555AFF,#9C9CA0FF,#D2D2D6FF role=group seen=428,790,1065,114
              41 TextBlock at=428,790,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" in=21 layout=180.77,28.8 limit=428,790,1065,196.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Crop, % of the picture"
              42 HintText at=428,839.5,1065,72 box=710,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen, the same for every game of this console in every mode." in=21 layout=684.3,48 limit=428,790,1065,196.5 paints=#86868AFF part role=text seen=428,839.5,1065,64.5 size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen, the same for every game of this console in every mode."
focus 27
edge 4 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7
edge 4 down same
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge 6 down same
edge 6 left 4
edge 6 right same
edge 9 up same
edge 9 down 27
edge 27 up 9
edge 27 down 37
edge 27 left same
edge 27 right 29
edge 29 up 9
edge 29 down 37
edge 37 up 27
edge 37 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1 up 37
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1 left 9
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3 up 37
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5 up 37
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7 up 37
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/7 right 6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/2/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 down 4
press pageup focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=NES title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
