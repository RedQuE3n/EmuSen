# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CompressedDiscsRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Compressed Discs"
case CompressedDiscsRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        3 HintText at=410,871,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Converting. The library is only read." layout=228.28,24 limit=350.48,187,1219.05,822 name=CompressedDiscsStatus paints=#8D8D91FF role=text size=20 text="Converting. The library is only read."
        4 ButtonBar at=1380.5,919,130.5,66 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=toolbar
          5 Button at=1380.5,919,130.5,66 by=clip drawn focus heard=CLOSE limit=1380.5,919,130.5,66 name=CompressedDiscsClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            6 TextBlock at=1406,928,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1380.5,919,130.5,66 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=410,229,1101,627 by=panel fades=b limit=350.48,187,1219.05,822 overflow=672 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=410,229,1101,627
        8 StackPanel at=410,229,1095,1299 by=scroll in=7 limit=410,229,1101,627 role=none seen=410,229,1095,627
          9 HintText name=CompressedDiscsAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          10 FieldRow at=410,361,1095,229.5 by=scroll heard="Convert or check" in=7 limit=410,229,1101,627 paints=#D4D4D4FF role=group
            11 TextBlock at=410,361,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=131.26,28.8 limit=410,361,1095,229.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Convert or check"
            12 HintText at=410,410.5,1095,108 box=730,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=724.46,72 limit=410,361,1095,229.5 paints=#8D8D91FF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=410,524.5,1095,66 by=clip heard="Convert or check" in=7 limit=410,361,1095,229.5 role=none
              14 Button at=410,524.5,295.5,66 by=clip drawn heard="CONVERT 3 CHOSEN" in=7 limit=410,361,1095,229.5 name=CompressedDiscsConvert off paints=#707070FF,#D4D4D4FF role=button text="Convert 3 Chosen"
                15 TextBlock at=435.5,533.5,244.5,48 box=163,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT 3 CHOSEN" in=7 layout=162.76,31.2 limit=410,524.5,295.5,66 off owner=14 paints=#707070FF role=text size=26 text="CONVERT 3 CHOSEN"
              16 Button at=717.5,524.5,264,66 by=clip drawn heard="CHECK 3 CHOSEN" in=7 limit=410,361,1095,229.5 name=CompressedDiscsCheck off paints=#707070FF,#D4D4D4FF role=button text="Check 3 Chosen"
                17 TextBlock at=743,533.5,213,48 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK 3 CHOSEN" in=7 layout=141.54,31.2 limit=717.5,524.5,264,66 off owner=16 paints=#707070FF role=text size=26 text="CHECK 3 CHOSEN"
              18 Button at=993.5,524.5,115.5,66 by=clip drawn focus heard=STOP in=7 limit=410,361,1095,229.5 name=CompressedDiscsStop op paints=#707070FF,#D4D4D4FF role=button text=Stop
                19 TextBlock at=1019,533.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=42.17,31.2 limit=993.5,524.5,115.5,66 owner=18 paints=#D4D4D4FF role=text size=26 text=STOP
              20 Button at=1121,524.5,151.5,66 by=clip drawn heard=RESUME in=7 limit=410,361,1095,229.5 name=CompressedDiscsResume off paints=#707070FF,#D4D4D4FF role=button text=Resume
                21 TextBlock at=1146.5,533.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=66.64,31.2 limit=1121,524.5,151.5,66 off owner=20 paints=#707070FF role=text size=26 text=RESUME
          22 FieldRow at=410,614.5,1095,153 by=scroll heard=Progress in=7 limit=410,229,1101,627 paints=#D4D4D4FF role=group
            23 TextBlock at=410,614.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=68.93,28.8 limit=410,614.5,1095,153 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Progress
            24 StackPanel at=410,664,1095,103.5 by=clip heard=Progress in=7 limit=410,614.5,1095,153 role=none
              25 ProgressBar at=410,664,1095,6 by=clip drawn in=7 limit=410,614.5,1095,153 name=CompressedDiscsProgress paints=#333337FF,#4C9AE8FF role=progressbar
              26 TextBlock at=410,679,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin." in=7 layout=418.27,28.8 limit=410,614.5,1095,153 name=CompressedDiscsProgressText paints=#D4D4D4FF role=text size=24 text="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin."
              27 HintText name=CompressedDiscsLog paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          28 FieldRow at=410,791.5,1095,736.5 by=scroll heard=Discs in=7 limit=410,229,1101,627 paints=#4C9AE8FF,#8D8D91FF,#D4D4D4FF role=group seen=410,791.5,1095,64.5
            29 TextBlock at=410,791.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Discs in=7 layout=40.78,28.8 limit=410,791.5,1095,736.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Discs
            30 HintText at=410,841,1095,36 box=730,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each disc's size and state; choose discs to convert or check only those." in=7 layout=459.94,24 limit=410,791.5,1095,736.5 paints=#8D8D91FF part role=text seen=410,841,1095,15 size=20 text="Each disc's size and state; choose discs to convert or check only those."
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
