# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state InsertDiscEmpty-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,265,812.7,244
title "Insert Disc"
case InsertDiscEmpty
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,365,813,124 by=panel limit=234,365,812.35,124 paints=#D4D4D4FF role=none seen=234,365,812.35,124
    1 StackPanel at=274,393,733,80 by=panel limit=234,365,812.35,124 role=none
      2 HintText at=274,393,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="No PlayStation games are in the library." layout=254.62,24 limit=234,365,812.35,124 name=InsertDiscAbout paints=#8D8D91FF role=text size=20 text="No PlayStation games are in the library."
      3 ButtonBar at=908,429,99,44 by=panel limit=234,365,812.35,124 paints=#D4D4D4FF role=toolbar
        4 Button at=908,429,99,44 by=clip drawn focus heard=CANCEL limit=908,429,99,44 name=InsertDiscCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          5 TextBlock at=925,435,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=908,429,99,44 owner=4 paints=#EEEEF0FF role=text size=26 text=CANCEL
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
