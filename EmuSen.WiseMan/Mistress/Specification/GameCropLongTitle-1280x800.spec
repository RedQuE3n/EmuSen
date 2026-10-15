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
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,268,840,343 by=panel limit=220,268,840,343 paints=#9C9CA0FF role=none
    1 StackPanel at=260,296,760,299 by=panel limit=220,268,840,343 role=none
      2 TextBlock at=260,296,760,58 box=760,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)" layout=756.79,57.6 limit=220,268,840,343 name=GameCropGame paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)"
      3 HintText at=260,364,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs." layout=730.34,48 limit=220,268,840,343 paints=#86868AFF role=text size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs."
      4 FieldRow at=260,422,760,79 by=panel heard="Crop, % of the picture" limit=220,268,840,343 paints=#9C9CA0FF role=group
        5 TextBlock at=260,422,760,29 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" layout=180.77,28.8 limit=260,422,760,79 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Crop, % of the picture"
        6 WrapPanel at=260,455,760,46 by=clip heard="Crop, % of the picture" limit=260,422,760,79 role=none
          7 TextBlock at=260,463,33,29 box=33,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Left layout=32.23,28.8 limit=260,422,760,79 paints=#9C9CA0FF role=text size=24 text=Left
          8 TextBox at=299,455,64,46 by=clip drawn focus heard="Left, percent" limit=260,422,760,79 name=GameCrop.CropLeft op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit text=0
            9 ScrollViewer allow="a text box scrolls its own text" at=310,462,42,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          10 TextBlock at=377,463,43,29 box=43,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Right layout=42.12,28.8 limit=260,422,760,79 paints=#9C9CA0FF role=text size=24 text=Right
          11 TextBox at=426,455,64,46 by=clip drawn focus heard="Right, percent" limit=260,422,760,79 name=GameCrop.CropRight op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            12 ScrollViewer allow="a text box scrolls its own text" at=437,462,42,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          13 TextBlock at=504,463,30,29 box=30,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Top layout=29.14,28.8 limit=260,422,760,79 paints=#9C9CA0FF role=text size=24 text=Top
          14 TextBox at=540,455,64,46 by=clip drawn focus heard="Top, percent" limit=260,422,760,79 name=GameCrop.CropTop op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            15 ScrollViewer allow="a text box scrolls its own text" at=551,462,42,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          16 TextBlock at=618,463,60,29 box=60,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Bottom layout=59.06,28.8 limit=260,422,760,79 paints=#9C9CA0FF role=text size=24 text=Bottom
          17 TextBox at=684,455,64,46 by=clip drawn focus heard="Bottom, percent" limit=260,422,760,79 name=GameCrop.CropBottom op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            18 ScrollViewer allow="a text box scrolls its own text" at=695,462,42,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
      19 HintText at=260,511,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="This game is drawn with the SNES's setting." layout=294.14,24 limit=220,268,840,343 name=GameCropStatus paints=#86868AFF role=text size=20 text="This game is drawn with the SNES's setting."
      20 ButtonBar at=635,551,385,44 by=panel limit=220,268,840,343 paints=#9C9CA0FF role=toolbar
        21 Button at=635,551,287,44 by=clip drawn heard="USE THE CONSOLE'S SETTING" limit=635,551,385,44 name=GameCropUseConsole off paints=#55555AFF,#9C9CA0FF role=button text="Use the Console's Setting"
          22 TextBlock at=652,557,253,32 box=253,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THE CONSOLE'S SETTING" layout=252.23,31.2 limit=635,551,287,44 off owner=21 paints=#55555AFF role=text size=26 text="USE THE CONSOLE'S SETTING"
        23 Button at=930,551,90,44 by=clip drawn focus heard=CLOSE limit=635,551,385,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          24 TextBlock at=947,557,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=930,551,90,44 owner=23 paints=#9C9CA0FF role=text size=26 text=CLOSE
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
