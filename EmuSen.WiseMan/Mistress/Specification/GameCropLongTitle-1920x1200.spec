# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GameCropLongTitle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,252.5,1260,695
title "Crop This Game"
case GameCropLongTitle
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,402,1260,516 by=panel limit=330,402,1260,515 paints=#9C9CA0FF role=none seen=330,402,1260,515
    1 StackPanel at=390,444,1140,450 by=panel limit=330,402,1260,515 role=none
      2 TextBlock at=390,444,1140,87 box=760,58 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)" layout=756.79,57.6 limit=330,402,1260,515 name=GameCropGame paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta)"
      3 HintText at=390,546,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs." layout=730.34,48 limit=330,402,1260,515 paints=#86868AFF role=text size=20 text="How much of the picture's width or height to hide at each edge, up to 25. The rest fills the screen. A crop set here is this game's alone and replaces the SNES's Overscan setting while it runs."
      4 FieldRow at=390,633,1140,118.5 by=panel heard="Crop, % of the picture" limit=330,402,1260,515 paints=#9C9CA0FF role=group
        5 TextBlock at=390,633,1140,43.5 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Crop, % of the picture" layout=180.77,28.8 limit=390,633,1140,118.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Crop, % of the picture"
        6 WrapPanel at=390,682.5,1140,69 by=clip heard="Crop, % of the picture" limit=390,633,1140,118.5 role=none
          7 TextBlock at=390,694.5,49.5,43.5 box=33,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Left layout=32.23,28.8 limit=390,633,1140,118.5 paints=#9C9CA0FF role=text size=24 text=Left
          8 TextBox at=448.5,682.5,96,69 by=clip drawn focus heard="Left, percent" limit=390,633,1140,118.5 name=GameCrop.CropLeft op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit text=0
            9 ScrollViewer allow="a text box scrolls its own text" at=465,693,63,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
          10 TextBlock at=565.5,694.5,64.5,43.5 box=43,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Right layout=42.12,28.8 limit=390,633,1140,118.5 paints=#9C9CA0FF role=text size=24 text=Right
          11 TextBox at=639,682.5,96,69 by=clip drawn focus heard="Right, percent" limit=390,633,1140,118.5 name=GameCrop.CropRight op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            12 ScrollViewer allow="a text box scrolls its own text" at=655.5,693,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          13 TextBlock at=756,694.5,45,43.5 box=30,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Top layout=29.14,28.8 limit=390,633,1140,118.5 paints=#9C9CA0FF role=text size=24 text=Top
          14 TextBox at=810,682.5,96,69 by=clip drawn focus heard="Top, percent" limit=390,633,1140,118.5 name=GameCrop.CropTop op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            15 ScrollViewer allow="a text box scrolls its own text" at=826.5,693,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
          16 TextBlock at=927,694.5,90,43.5 box=60,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Bottom layout=59.06,28.8 limit=390,633,1140,118.5 paints=#9C9CA0FF role=text size=24 text=Bottom
          17 TextBox at=1026,682.5,96,69 by=clip drawn focus heard="Bottom, percent" limit=390,633,1140,118.5 name=GameCrop.CropBottom op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit text=0
            18 ScrollViewer allow="a text box scrolls its own text" at=1042.5,693,63,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
      19 HintText at=390,766.5,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="This game is drawn with the SNES's setting." layout=294.14,24 limit=330,402,1260,515 name=GameCropStatus paints=#86868AFF role=text size=20 text="This game is drawn with the SNES's setting."
      20 ButtonBar at=952.5,826.5,577.5,66 by=panel limit=330,402,1260,515 paints=#9C9CA0FF role=toolbar
        21 Button at=952.5,826.5,430.5,66 by=clip drawn heard="USE THE CONSOLE'S SETTING" limit=952.5,826.5,577.5,66 name=GameCropUseConsole off paints=#55555AFF,#9C9CA0FF role=button text="Use the Console's Setting"
          22 TextBlock at=978,835.5,379.5,48 box=253,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE THE CONSOLE'S SETTING" layout=252.23,31.2 limit=952.5,826.5,430.5,66 off owner=21 paints=#55555AFF role=text size=26 text="USE THE CONSOLE'S SETTING"
        23 Button at=1395,826.5,135,66 by=clip drawn focus heard=CLOSE limit=952.5,826.5,577.5,66 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          24 TextBlock at=1420.5,835.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,826.5,135,66 owner=23 paints=#9C9CA0FF role=text size=26 text=CLOSE
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
