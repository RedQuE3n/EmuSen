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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,548,1221,186 by=panel limit=350.48,548,1219.05,186 paints=#D4D4D4FF role=none seen=350.48,548,1219.05,186
    1 StackPanel at=410,590,1101,120 by=panel limit=350.48,548,1219.05,186 role=none
      2 HintText at=410,590,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="No PlayStation games are in the library." layout=254.62,24 limit=350.48,548,1219.05,186 name=InsertDiscAbout paints=#8D8D91FF role=text size=20 text="No PlayStation games are in the library."
      3 ButtonBar at=1362.5,644,148.5,66 by=panel limit=350.48,548,1219.05,186 paints=#D4D4D4FF role=toolbar
        4 Button at=1362.5,644,148.5,66 by=clip drawn focus heard=CANCEL limit=1362.5,644,148.5,66 name=InsertDiscCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          5 TextBlock at=1388,653,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1362.5,644,148.5,66 owner=4 paints=#EEEEF0FF role=text size=26 text=CANCEL
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
