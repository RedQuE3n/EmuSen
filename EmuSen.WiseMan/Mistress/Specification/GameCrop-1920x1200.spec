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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,405,1221,471 by=panel limit=350.48,405,1219.05,471 paints=#D4D4D4FF role=none seen=350.48,405,1219.05,471
    1 StackPanel at=410,447,1101,405 by=panel limit=350.48,405,1219.05,471 role=none
      2 TextBlock at=410,447,1101,43.5 box=734,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=192.29,28.8 limit=350.48,405,1219.05,471 name=GameCropGame paints=#D4D4D4FF role=text size=24 text="Cobalt Harbor (Synthetic)"
      3 HintText at=410,505.5,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs." layout=733.32,48 limit=350.48,405,1219.05,471 paints=#8D8D91FF role=text size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs."
      4 FieldRow at=410,592.5,1101,118.5 by=panel heard="Crop, % of the picture" limit=350.48,405,1219.05,471 paints=#D4D4D4FF role=group
        5 TextBlock at=410,592.5,1101,43.5 box=734,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" layout=170.3,28.8 limit=410,592.5,1101,118.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Crop, % of the picture"
        6 WrapPanel at=410,642,1101,69 by=clip heard="Crop, % of the picture" limit=410,592.5,1101,118.5 role=none
          7 TextBlock at=410,654,46.5,43.5 box=31,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Left layout=30.1,28.8 limit=410,592.5,1101,118.5 paints=#D4D4D4FF role=text size=24 text=Left
          8 TextBox at=465.5,642,96,69 by=clip drawn focus heard="Left, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropLeft op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit text=12.5
            9 ScrollViewer allow="a text box scrolls its own text" at=482,652.5,63,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          10 TextBlock at=582.5,654,60,43.5 box=40,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Right layout=39.38,28.8 limit=410,592.5,1101,118.5 paints=#D4D4D4FF role=text size=24 text=Right
          11 TextBox at=651.5,642,96,69 by=clip drawn focus heard="Right, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropRight op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=12.5
            12 ScrollViewer allow="a text box scrolls its own text" at=668,652.5,63,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          13 TextBlock at=768.5,654,43.5,43.5 box=29,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Top layout=28.87,28.8 limit=410,592.5,1101,118.5 paints=#D4D4D4FF role=text size=24 text=Top
          14 TextBox at=821,642,96,69 by=clip drawn focus heard="Top, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropTop op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=12.5
            15 ScrollViewer allow="a text box scrolls its own text" at=837.5,652.5,63,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          16 TextBlock at=938,654,84,43.5 box=56,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Bottom layout=55.63,28.8 limit=410,592.5,1101,118.5 paints=#D4D4D4FF role=text size=24 text=Bottom
          17 TextBox at=1031,642,96,69 by=clip drawn focus heard="Bottom, percent" limit=410,592.5,1101,118.5 name=GameCrop.CropBottom op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=12.5
            18 ScrollViewer allow="a text box scrolls its own text" at=1047.5,652.5,63,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
      19 HintText at=410,726,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="This game has a crop of its own." layout=209.7,24 limit=350.48,405,1219.05,471 name=GameCropStatus paints=#8D8D91FF role=text size=20 text="This game has a crop of its own."
      20 ButtonBar at=959,786,552,66 by=panel limit=350.48,405,1219.05,471 paints=#D4D4D4FF role=toolbar
        21 Button at=959,786,409.5,66 by=clip drawn focus heard="USE THE CONSOLE'S SETTING" limit=959,786,552,66 name=GameCropUseConsole op paints=#707070FF,#D4D4D4FF role=button text="Use the Console's Setting"
          22 TextBlock at=984.5,795,358.5,48 box=239,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THE CONSOLE'S SETTING" layout=238.6,31.2 limit=959,786,409.5,66 owner=21 paints=#D4D4D4FF role=text size=26 text="USE THE CONSOLE'S SETTING"
        23 Button at=1380.5,786,130.5,66 by=clip drawn focus heard=CLOSE limit=959,786,552,66 op paints=#707070FF,#D4D4D4FF role=button text=Close
          24 TextBlock at=1406,795,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1380.5,786,130.5,66 owner=23 paints=#D4D4D4FF role=text size=26 text=CLOSE
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
