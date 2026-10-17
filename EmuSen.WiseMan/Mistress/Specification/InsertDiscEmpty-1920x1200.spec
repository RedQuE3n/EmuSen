# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state InsertDiscEmpty-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,397.5,1219.05,366
title "Insert Disc"
case InsertDiscEmpty
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,548,1221,186 by=panel limit=350.48,548,1219.05,186 paints=#9C9CA0FF role=none seen=350.48,548,1219.05,186
    1 StackPanel at=410,590,1101,120 by=panel limit=350.48,548,1219.05,186 role=none
      2 HintText at=410,590,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="No PlayStation games are in the library." layout=267.52,24 limit=350.48,548,1219.05,186 name=InsertDiscAbout paints=#86868AFF role=text size=20 text="No PlayStation games are in the library."
      3 ButtonBar at=1356.5,644,154.5,66 by=panel limit=350.48,548,1219.05,186 paints=#9C9CA0FF role=toolbar
        4 Button at=1356.5,644,154.5,66 by=clip drawn focus heard=CANCEL limit=1356.5,644,154.5,66 name=InsertDiscCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          5 TextBlock at=1382,653,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1356.5,644,154.5,66 owner=4 paints=#EEEEF0FF role=text size=26 text=CANCEL
focus 4
edge 4 up same
edge 4 down same
edge 4 left same
edge 4 right same
press pageup focus="Button 'Cancel'" title="Insert Disc" top=InsertDiscWindow
press pagedown focus="Button 'Cancel'" title="Insert Disc" top=InsertDiscWindow
press first focus="Button 'Cancel'" title="Insert Disc" top=InsertDiscWindow
press last focus="Button 'Cancel'" title="Insert Disc" top=InsertDiscWindow
press back top=none
