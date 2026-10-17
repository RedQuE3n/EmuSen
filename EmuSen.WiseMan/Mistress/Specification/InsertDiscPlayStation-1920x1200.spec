# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state InsertDiscPlayStation-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,261.5,1219.05,638
title "Insert Disc"
case InsertDiscPlayStation
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,412,1221,459 by=panel limit=350.48,412,1219.05,458 paints=#9C9CA0FF role=none seen=350.48,412,1219.05,458
    1 StackPanel at=410,454,1101,393 by=panel limit=350.48,412,1219.05,458 role=none
      2 HintText at=410,454,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later." layout=666.06,24 limit=350.48,412,1219.05,458 name=InsertDiscAbout paints=#86868AFF role=text size=20 text="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later."
      3 ScrollViewer at=410,508,1101,253.5 by=panel limit=350.48,412,1219.05,458 name=InsertDiscScroll overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=410,508,1101,253.5
        4 StackPanel at=410,508,1101,253.5 by=fixed-scroll in=3 limit=410,508,1101,253.5 name=InsertDiscList role=none
          5 Button at=410,508,1101,66 by=fixed-scroll drawn focus heard="Amber Saga" in=3 limit=410,508,1101,253.5 name=InsertDisc0 op paints=#55555AFF,#9C9CA0FF role=button
            6 TextBlock at=435.5,517,157.5,48 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Amber Saga" in=3 layout=104.91,31.2 limit=410,508,1101,66 owner=5 paints=#EEEEF0FF role=text size=26 text="Amber Saga"
          7 Button at=410,580,1101,109.5 by=fixed-scroll drawn focus heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 limit=410,508,1101,253.5 name=InsertDisc1 op paints=#55555AFF,#9C9CA0FF role=button
            8 TextBlock at=435.5,587.5,1024.5,94.5 box=683,63 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 layout=677.27,62.4 limit=410,580,1101,109.5 owner=7 paints=#9C9CA0FF role=text size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)"
          9 Button at=410,695.5,1101,66 by=fixed-scroll drawn focus heard="Cobalt Harbor" in=3 limit=410,508,1101,253.5 name=InsertDisc2 op paints=#55555AFF,#9C9CA0FF role=button
            10 TextBlock at=435.5,704.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Cobalt Harbor" in=3 layout=120.64,31.2 limit=410,695.5,1101,66 owner=9 paints=#9C9CA0FF role=text size=26 text="Cobalt Harbor"
      11 ButtonBar at=1356.5,779.5,154.5,66 by=panel limit=350.48,412,1219.05,458 paints=#9C9CA0FF role=toolbar
        12 Button at=1356.5,779.5,154.5,66 by=clip drawn focus heard=CANCEL limit=1356.5,779.5,154.5,66 name=InsertDiscCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          13 TextBlock at=1382,788.5,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1356.5,779.5,154.5,66 owner=12 paints=#9C9CA0FF role=text size=26 text=CANCEL
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
