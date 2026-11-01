# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state InsertDiscPlayStation-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,283.5,1219.05,594
title "Insert Disc"
case InsertDiscPlayStation
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,434,1221,414 by=panel limit=350.48,434,1219.05,414 paints=#D4D4D4FF role=none seen=350.48,434,1219.05,414
    1 StackPanel at=410,476,1101,348 by=panel limit=350.48,434,1219.05,414 role=none
      2 HintText at=410,476,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later." layout=636.26,24 limit=350.48,434,1219.05,414 name=InsertDiscAbout paints=#8D8D91FF role=text size=20 text="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later."
      3 ScrollViewer at=410,530,1101,210 by=panel limit=350.48,434,1219.05,414 name=InsertDiscScroll overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=410,530,1101,210
        4 StackPanel at=410,530,1101,210 by=fixed-scroll in=3 limit=410,530,1101,210 name=InsertDiscList role=none
          5 Button at=410,530,1101,66 by=fixed-scroll drawn focus heard="Amber Saga" in=3 limit=410,530,1101,210 name=InsertDisc0 op paints=#707070FF,#D4D4D4FF role=button
            6 TextBlock at=435.5,539,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Amber Saga" in=3 layout=101.56,31.2 limit=410,530,1101,66 owner=5 paints=#EEEEF0FF role=text size=26 text="Amber Saga"
          7 Button at=410,602,1101,66 by=fixed-scroll drawn focus heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 limit=410,530,1101,210 name=InsertDisc1 op paints=#707070FF,#D4D4D4FF role=button
            8 TextBlock at=435.5,611,1035,48 box=690,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 layout=689.78,31.2 limit=410,602,1101,66 owner=7 paints=#D4D4D4FF role=text size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)"
          9 Button at=410,674,1101,66 by=fixed-scroll drawn focus heard="Cobalt Harbor" in=3 limit=410,530,1101,210 name=InsertDisc2 op paints=#707070FF,#D4D4D4FF role=button
            10 TextBlock at=435.5,683,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Cobalt Harbor" in=3 layout=114.53,31.2 limit=410,674,1101,66 owner=9 paints=#D4D4D4FF role=text size=26 text="Cobalt Harbor"
      11 ButtonBar at=1362.5,758,148.5,66 by=panel limit=350.48,434,1219.05,414 paints=#D4D4D4FF role=toolbar
        12 Button at=1362.5,758,148.5,66 by=clip drawn focus heard=CANCEL limit=1362.5,758,148.5,66 name=InsertDiscCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          13 TextBlock at=1388,767,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1362.5,758,148.5,66 owner=12 paints=#D4D4D4FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 7
edge 5 left same
edge 5 right 12
edge 7 up 5
edge 7 down 9
edge 7 left same
edge 7 right 12
edge 9 up 7
edge 9 down 12
edge 9 left same
edge 9 right 12
edge 12 up 9
edge 12 down same
edge 12 left 9
edge 12 right same
press pageup focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press pagedown focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press first focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press last focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press back top=none
