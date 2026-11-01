# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GameCropLongTitle-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,168.5,840,463
title "Crop This Game"
case GameCropLongTitle
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,268,840,343 by=panel limit=220,268,840,343 paints=#D4D4D4FF role=none
    1 StackPanel at=260,296,760,299 by=panel limit=220,268,840,343 role=none
      2 TextBlock at=260,296,760,58 box=760,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)" layout=740.78,57.6 limit=220,268,840,343 name=GameCropGame paints=#D4D4D4FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)"
      3 HintText at=260,364,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs." layout=747.88,48 limit=220,268,840,343 paints=#8D8D91FF role=text size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs."
      4 FieldRow at=260,422,760,79 by=panel heard="Crop, % of the picture" limit=220,268,840,343 paints=#D4D4D4FF role=group
        5 TextBlock at=260,422,760,29 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" layout=170.3,28.8 limit=260,422,760,79 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Crop, % of the picture"
        6 WrapPanel at=260,455,760,46 by=clip heard="Crop, % of the picture" limit=260,422,760,79 role=none
          7 TextBlock at=260,463,31,29 box=31,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Left layout=30.1,28.8 limit=260,422,760,79 paints=#D4D4D4FF role=text size=24 text=Left
          8 TextBox at=297,455,64,46 by=clip drawn focus heard="Left, percent" limit=260,422,760,79 name=GameCrop.CropLeft op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit text=0
            9 ScrollViewer allow="a text box scrolls its own text" at=308,462,42,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          10 TextBlock at=375,463,40,29 box=40,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Right layout=39.38,28.8 limit=260,422,760,79 paints=#D4D4D4FF role=text size=24 text=Right
          11 TextBox at=421,455,64,46 by=clip drawn focus heard="Right, percent" limit=260,422,760,79 name=GameCrop.CropRight op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=0
            12 ScrollViewer allow="a text box scrolls its own text" at=432,462,42,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          13 TextBlock at=499,463,29,29 box=29,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Top layout=28.87,28.8 limit=260,422,760,79 paints=#D4D4D4FF role=text size=24 text=Top
          14 TextBox at=534,455,64,46 by=clip drawn focus heard="Top, percent" limit=260,422,760,79 name=GameCrop.CropTop op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=0
            15 ScrollViewer allow="a text box scrolls its own text" at=545,462,42,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          16 TextBlock at=612,463,56,29 box=56,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Bottom layout=55.63,28.8 limit=260,422,760,79 paints=#D4D4D4FF role=text size=24 text=Bottom
          17 TextBox at=674,455,64,46 by=clip drawn focus heard="Bottom, percent" limit=260,422,760,79 name=GameCrop.CropBottom op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=0
            18 ScrollViewer allow="a text box scrolls its own text" at=685,462,42,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
      19 HintText at=260,511,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="This game is drawn with the SNES's setting." layout=280.34,24 limit=220,268,840,343 name=GameCropStatus paints=#8D8D91FF role=text size=20 text="This game is drawn with the SNES's setting."
      20 ButtonBar at=652,551,368,44 by=panel limit=220,268,840,343 paints=#D4D4D4FF role=toolbar
        21 Button at=652,551,273,44 by=clip drawn heard="USE THE CONSOLE'S SETTING" limit=652,551,368,44 name=GameCropUseConsole off paints=#707070FF,#D4D4D4FF role=button text="Use the Console's Setting"
          22 TextBlock at=669,557,239,32 box=239,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THE CONSOLE'S SETTING" layout=238.6,31.2 limit=652,551,273,44 off owner=21 paints=#707070FF role=text size=26 text="USE THE CONSOLE'S SETTING"
        23 Button at=933,551,87,44 by=clip drawn focus heard=CLOSE limit=652,551,368,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
          24 TextBlock at=950,557,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,551,87,44 owner=23 paints=#D4D4D4FF role=text size=26 text=CLOSE
focus 8
edge 8 up same
edge 8 down 23
edge 8 left same
edge 8 right 11
edge 11 up same
edge 11 down 23
edge 11 left 8
edge 11 right 14
edge 14 up same
edge 14 down 23
edge 14 left 11
edge 14 right 17
edge 17 up same
edge 17 down 23
edge 17 left 14
edge 17 right 23
edge 23 up 17
edge 23 down same
edge 23 left 17
edge 23 right same
press pageup focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press pagedown focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press first focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press last focus="TextBox GameCrop.CropLeft (Left, percent)" title="Crop This Game" top=GameCropWindow
press back top=none
