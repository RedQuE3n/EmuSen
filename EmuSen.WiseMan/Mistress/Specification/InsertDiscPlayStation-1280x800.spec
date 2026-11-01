# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state InsertDiscPlayStation-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,189,812.7,396
title "Insert Disc"
case InsertDiscPlayStation
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,289,813,276 by=panel limit=234,289,812.35,276 paints=#D4D4D4FF role=none seen=234,289,812.35,276
    1 StackPanel at=274,317,733,232 by=panel limit=234,289,812.35,276 role=none
      2 HintText at=274,317,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later." layout=636.26,24 limit=234,289,812.35,276 name=InsertDiscAbout paints=#8D8D91FF role=text size=20 text="The disc goes in as on a console: the lid opens, the disc goes in, and the lid closes a moment later."
      3 ScrollViewer at=274,353,733,140 by=panel limit=234,289,812.35,276 name=InsertDiscScroll overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=274,353,733,140
        4 StackPanel at=274,353,733,140 by=fixed-scroll in=3 limit=274,353,733,140 name=InsertDiscList role=none
          5 Button at=274,353,733,44 by=fixed-scroll drawn focus heard="Amber Saga" in=3 limit=274,353,733,140 name=InsertDisc0 op paints=#707070FF,#D4D4D4FF role=button
            6 TextBlock at=291,359,102,32 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Amber Saga" in=3 layout=101.56,31.2 limit=274,353,733,44 owner=5 paints=#EEEEF0FF role=text size=26 text="Amber Saga"
          7 Button at=274,401,733,44 by=fixed-scroll drawn focus heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 limit=274,353,733,140 name=InsertDisc1 op paints=#707070FF,#D4D4D4FF role=button
            8 TextBlock at=291,407,690,32 box=690,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)" in=3 layout=689.78,31.2 limit=274,401,733,44 owner=7 paints=#D4D4D4FF role=text size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Disc 2 of 3)"
          9 Button at=274,449,733,44 by=fixed-scroll drawn focus heard="Cobalt Harbor" in=3 limit=274,353,733,140 name=InsertDisc2 op paints=#707070FF,#D4D4D4FF role=button
            10 TextBlock at=291,455,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Cobalt Harbor" in=3 layout=114.53,31.2 limit=274,449,733,44 owner=9 paints=#D4D4D4FF role=text size=26 text="Cobalt Harbor"
      11 ButtonBar at=908,505,99,44 by=panel limit=234,289,812.35,276 paints=#D4D4D4FF role=toolbar
        12 Button at=908,505,99,44 by=clip drawn focus heard=CANCEL limit=908,505,99,44 name=InsertDiscCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          13 TextBlock at=925,511,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=908,505,99,44 owner=12 paints=#D4D4D4FF role=text size=26 text=CANCEL
focus 5
edge 5 up same
edge 5 down 7
edge 5 left same
edge 5 right same
edge 7 up 5
edge 7 down 9
edge 7 left same
edge 7 right same
edge 9 up 7
edge 9 down 12
edge 9 left same
edge 9 right same
edge 12 up 9
edge 12 down same
edge 12 left same
edge 12 right same
press pageup focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press pagedown focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press first focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press last focus="Button InsertDisc0" title="Insert Disc" top=InsertDiscWindow
press back top=none
