# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CompressedDiscsBeside-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Compressed Discs"
case CompressedDiscsBeside
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        3 HintText at=410,871,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="5 discs in the library." layout=134.66,24 limit=350.48,187,1219.05,822 name=CompressedDiscsStatus paints=#8D8D91FF role=text size=20 text="5 discs in the library."
        4 ButtonBar at=1380.5,919,130.5,66 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=toolbar
          5 Button at=1380.5,919,130.5,66 by=clip drawn focus heard=CLOSE limit=1380.5,919,130.5,66 name=CompressedDiscsClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            6 TextBlock at=1406,928,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1380.5,919,130.5,66 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=410,229,1101,627 by=panel fades=b limit=350.48,187,1219.05,822 overflow=1270.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=410,229,1101,627
        8 StackPanel at=410,229,1095,1897.5 by=scroll in=7 limit=410,229,1101,627 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=410,229,1095,627
          9 HintText name=CompressedDiscsAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          10 FieldRow at=410,361,1095,229.5 by=scroll heard="Convert or check" in=7 limit=410,229,1101,627 paints=#D4D4D4FF role=group
            11 TextBlock at=410,361,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=131.26,28.8 limit=410,361,1095,229.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Convert or check"
            12 HintText at=410,410.5,1095,108 box=730,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=724.46,72 limit=410,361,1095,229.5 paints=#8D8D91FF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=410,524.5,1095,66 by=clip heard="Convert or check" in=7 limit=410,361,1095,229.5 role=none
              14 Button at=410,524.5,315,66 by=clip drawn focus heard="CONVERT EVERY DISC" in=7 limit=410,361,1095,229.5 name=CompressedDiscsConvert op paints=#707070FF,#D4D4D4FF role=button text="Convert Every Disc"
                15 TextBlock at=435.5,533.5,264,48 box=176,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT EVERY DISC" in=7 layout=175.53,31.2 limit=410,524.5,315,66 owner=14 paints=#D4D4D4FF role=text size=26 text="CONVERT EVERY DISC"
              16 Button at=737,524.5,288,66 by=clip drawn focus heard="CHECK CONVERTED" in=7 limit=410,361,1095,229.5 name=CompressedDiscsCheck op paints=#707070FF,#D4D4D4FF role=button text="Check Converted"
                17 TextBlock at=762.5,533.5,237,48 box=158,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK CONVERTED" in=7 layout=157.17,31.2 limit=737,524.5,288,66 owner=16 paints=#D4D4D4FF role=text size=26 text="CHECK CONVERTED"
              18 Button at=1037,524.5,115.5,66 by=clip drawn heard=STOP in=7 limit=410,361,1095,229.5 name=CompressedDiscsStop off paints=#707070FF,#D4D4D4FF role=button text=Stop
                19 TextBlock at=1062.5,533.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=42.17,31.2 limit=1037,524.5,115.5,66 off owner=18 paints=#707070FF role=text size=26 text=STOP
              20 Button at=1164.5,524.5,151.5,66 by=clip drawn heard=RESUME in=7 limit=410,361,1095,229.5 name=CompressedDiscsResume off paints=#707070FF,#D4D4D4FF role=button text=Resume
                21 TextBlock at=1190,533.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=66.64,31.2 limit=1164.5,524.5,151.5,66 off owner=20 paints=#707070FF role=text size=26 text=RESUME
          22 FieldRow at=410,614.5,1095,93 by=scroll heard=Progress in=7 limit=410,229,1101,627 paints=#D4D4D4FF role=group
            23 TextBlock at=410,614.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=68.93,28.8 limit=410,614.5,1095,93 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Progress
            24 StackPanel at=410,664,1095,43.5 by=clip heard=Progress in=7 limit=410,614.5,1095,93 role=none
              25 TextBlock at=410,664,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No batch has run since this window opened." in=7 layout=343.3,28.8 limit=410,614.5,1095,93 name=CompressedDiscsProgressText paints=#D4D4D4FF role=text size=24 text="No batch has run since this window opened."
          26 FieldRow at=410,731.5,1095,310.5 by=scroll heard="CHDs go" in=7 limit=410,229,1101,627 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=group seen=410,731.5,1095,124.5
            27 TextBlock at=410,731.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CHDs go" in=7 layout=63.43,28.8 limit=410,731.5,1095,310.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="CHDs go"
            28 HintText at=410,781,1095,72 box=730,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Into the output folder, at each disc's path in the library; or, for this batch only, beside each original in the library, where the library then shows the game once." in=7 layout=719.88,48 limit=410,731.5,1095,310.5 paints=#8D8D91FF part role=text size=20 text="Into the output folder, at each disc's path in the library; or, for this batch only, beside each original in the library, where the library then shows the game once."
focus 5
edge 5 up 16
edge 5 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0
edge 5 left 16
edge 5 right same
edge 14 up same
edge 14 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 14 left same
edge 14 right 16
edge 16 up same
edge 16 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 16 left 14
edge 16 right 5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 up 14
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/6/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/1/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/7/0/2/0/0/0/0/0 right 5
press pageup focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press pagedown focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press first focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press last focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press back top=none
