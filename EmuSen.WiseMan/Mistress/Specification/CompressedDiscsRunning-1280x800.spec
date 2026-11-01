# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CompressedDiscsRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Compressed Discs"
case CompressedDiscsRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 StackPanel at=274,580,733,76 by=panel limit=234,124,812.35,548 role=none
        3 HintText at=274,580,733,24 box=733,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Converting. The library is only read." layout=228.28,24 limit=234,124,812.35,548 name=CompressedDiscsStatus paints=#8D8D91FF role=text size=20 text="Converting. The library is only read."
        4 ButtonBar at=920,612,87,44 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=toolbar
          5 Button at=920,612,87,44 by=clip drawn focus heard=CLOSE limit=920,612,87,44 name=CompressedDiscsClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            6 TextBlock at=937,618,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=920,612,87,44 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=274,152,733,418 by=panel fades=b limit=234,124,812.35,548 overflow=448 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=274,152,733,418
        8 StackPanel at=274,152,729,866 by=scroll in=7 limit=274,152,733,418 role=none seen=274,152,729,418
          9 HintText name=CompressedDiscsAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          10 FieldRow at=274,240,729,153 by=scroll heard="Convert or check" in=7 limit=274,152,733,418 paints=#D4D4D4FF role=group
            11 TextBlock at=274,240,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=131.26,28.8 limit=274,240,729,153 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Convert or check"
            12 HintText at=274,273,729,72 box=729,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=724.46,72 limit=274,240,729,153 paints=#8D8D91FF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=274,349,729,44 by=clip heard="Convert or check" in=7 limit=274,240,729,153 role=none
              14 Button at=274,349,197,44 by=clip drawn heard="CONVERT 3 CHOSEN" in=7 limit=274,240,729,153 name=CompressedDiscsConvert off paints=#707070FF,#D4D4D4FF role=button text="Convert 3 Chosen"
                15 TextBlock at=291,355,163,32 box=163,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT 3 CHOSEN" in=7 layout=162.76,31.2 limit=274,349,197,44 off owner=14 paints=#707070FF role=text size=26 text="CONVERT 3 CHOSEN"
              16 Button at=479,349,176,44 by=clip drawn heard="CHECK 3 CHOSEN" in=7 limit=274,240,729,153 name=CompressedDiscsCheck off paints=#707070FF,#D4D4D4FF role=button text="Check 3 Chosen"
                17 TextBlock at=496,355,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK 3 CHOSEN" in=7 layout=141.54,31.2 limit=479,349,176,44 off owner=16 paints=#707070FF role=text size=26 text="CHECK 3 CHOSEN"
              18 Button at=663,349,77,44 by=clip drawn focus heard=STOP in=7 limit=274,240,729,153 name=CompressedDiscsStop op paints=#707070FF,#D4D4D4FF role=button text=Stop
                19 TextBlock at=680,355,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=42.17,31.2 limit=663,349,77,44 owner=18 paints=#D4D4D4FF role=text size=26 text=STOP
              20 Button at=748,349,101,44 by=clip drawn heard=RESUME in=7 limit=274,240,729,153 name=CompressedDiscsResume off paints=#707070FF,#D4D4D4FF role=button text=Resume
                21 TextBlock at=765,355,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=66.64,31.2 limit=748,349,101,44 off owner=20 paints=#707070FF role=text size=26 text=RESUME
          22 FieldRow at=274,409,729,102 by=scroll heard=Progress in=7 limit=274,152,733,418 paints=#D4D4D4FF role=group
            23 TextBlock at=274,409,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=68.93,28.8 limit=274,409,729,102 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Progress
            24 StackPanel at=274,442,729,69 by=clip heard=Progress in=7 limit=274,409,729,102 role=none
              25 ProgressBar at=274,442,729,4 by=clip drawn in=7 limit=274,409,729,102 name=CompressedDiscsProgress paints=#333337FF,#4C9AE8FF role=progressbar
              26 TextBlock at=274,452,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin." in=7 layout=418.27,28.8 limit=274,409,729,102 name=CompressedDiscsProgressText paints=#D4D4D4FF role=text size=24 text="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin."
              27 HintText name=CompressedDiscsLog paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          28 FieldRow at=274,527,729,491 by=scroll heard=Discs in=7 limit=274,152,733,418 paints=#4C9AE8FF,#8D8D91FF,#D4D4D4FF role=group seen=274,527,729,43
            29 TextBlock at=274,527,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Discs in=7 layout=40.78,28.8 limit=274,527,729,491 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Discs
            30 HintText at=274,560,729,24 box=729,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each disc's size and state; choose discs to convert or check only those." in=7 layout=459.94,24 limit=274,527,729,491 paints=#8D8D91FF part role=text seen=274,560,729,10 size=20 text="Each disc's size and state; choose discs to convert or check only those."
focus 5
edge 5 up 18
edge 5 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/1/0
edge 5 left 18
edge 5 right same
edge 18 up same
edge 18 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0
edge 18 left same
edge 18 right 5
press pageup focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press pagedown focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press first focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press last focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press back top=none
