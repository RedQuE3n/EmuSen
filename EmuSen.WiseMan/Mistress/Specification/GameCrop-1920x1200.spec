# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GameCrop-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,255,1219.05,651
title "Crop This Game"
case GameCrop
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,405,1221,471 by=panel limit=350.48,405,1219.05,471 paints=#9C9CA0FF role=none seen=350.48,405,1219.05,471
    1 StackPanel at=410,447,1101,405 by=panel limit=350.48,405,1219.05,471 role=none
      2 TextBlock at=410,447,1101,43.5 box=734,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=203.04,28.8 limit=350.48,405,1219.05,471 name=GameCropGame paints=#9C9CA0FF role=text size=24 text="Cobalt Harbor (Synthetic)"
      3 HintText at=410,505.5,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs." layout=730.34,48 limit=350.48,405,1219.05,471 paints=#86868AFF role=text size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs."
      4 FieldRow at=410,592.5,1101,118.5 by=panel heard="Crop, % of the picture" limit=350.48,405,1219.05,471 paints=#9C9CA0FF role=group
        5 TextBlock at=410,592.5,1101,43.5 box=734,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" layout=180.77,28.8 limit=410,592.5,1101,118.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Crop, % of the picture"
        6 WrapPanel at=410,642,1101,69 by=clip heard="Crop, % of the picture" limit=410,592.5,1101,118.5 role=none
          7 TextBlock at=410,654,49.5,43.5 box=33,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Left layout=32.23,28.8 limit=410,592.5,1101,118.5 paints=#9C9CA0FF role=text size=24 text=Left
          8 TextBox at=468.5,642,96,69 by=clip drawn focus heard="Left, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropLeft op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit text=12.5
            9 ScrollViewer allow="a text box scrolls its own text" at=485,652.5,63,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          10 TextBlock at=585.5,654,64.5,43.5 box=43,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Right layout=42.12,28.8 limit=410,592.5,1101,118.5 paints=#9C9CA0FF role=text size=24 text=Right
          11 TextBox at=659,642,96,69 by=clip drawn focus heard="Right, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropRight op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=12.5
            12 ScrollViewer allow="a text box scrolls its own text" at=675.5,652.5,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          13 TextBlock at=776,654,45,43.5 box=30,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Top layout=29.14,28.8 limit=410,592.5,1101,118.5 paints=#9C9CA0FF role=text size=24 text=Top
          14 TextBox at=830,642,96,69 by=clip drawn focus heard="Top, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropTop op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=12.5
            15 ScrollViewer allow="a text box scrolls its own text" at=846.5,652.5,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          16 TextBlock at=947,654,90,43.5 box=60,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Bottom layout=59.06,28.8 limit=410,592.5,1101,118.5 paints=#9C9CA0FF role=text size=24 text=Bottom
          17 TextBox at=1046,642,96,69 by=clip drawn focus heard="Bottom, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropBottom op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=12.5
            18 ScrollViewer allow="a text box scrolls its own text" at=1062.5,652.5,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
      19 HintText at=410,726,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="This game has a crop of its own." layout=219.26,24 limit=350.48,405,1219.05,471 name=GameCropStatus paints=#86868AFF role=text size=20 text="This game has a crop of its own."
      20 ButtonBar at=933.5,786,577.5,66 by=panel limit=350.48,405,1219.05,471 paints=#9C9CA0FF role=toolbar
        21 Button at=933.5,786,430.5,66 by=clip drawn focus heard="USE THE CONSOLE'S SETTING" limit=933.5,786,577.5,66 name=GameCropUseConsole op paints=#55555AFF,#9C9CA0FF role=button text="Use the Console's Setting"
          22 TextBlock at=959,795,379.5,48 box=253,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THE CONSOLE'S SETTING" layout=252.23,31.2 limit=933.5,786,430.5,66 owner=21 paints=#9C9CA0FF role=text size=26 text="USE THE CONSOLE'S SETTING"
        23 Button at=1376,786,135,66 by=clip drawn focus heard=CLOSE limit=933.5,786,577.5,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          24 TextBlock at=1401.5,795,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,786,135,66 owner=23 paints=#9C9CA0FF role=text size=26 text=CLOSE
focus 8
edge 8 up same
edge 8 down 21
edge 8 left same
edge 8 right 11
edge 11 up same
edge 11 down 21
edge 11 left 8
edge 11 right 14
edge 14 up same
edge 14 down 21
edge 14 left 11
edge 14 right 17
edge 17 up same
edge 17 down 21
edge 17 left 14
edge 17 right 23
edge 21 up 17
edge 21 down same
edge 21 left 14
edge 21 right 23
edge 23 up 17
edge 23 down same
edge 23 left 21
edge 23 right same
press pageup focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press pagedown focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press first focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press last focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press back top=none
