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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        3 HintText at=410,871,1101,36 box=734,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Converting. The library is only read." layout=239.44,24 limit=350.48,187,1219.05,822 name=CompressedDiscsStatus paints=#86868AFF role=text size=20 text="Converting. The library is only read."
        4 ButtonBar at=1376,919,135,66 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=toolbar
          5 Button at=1376,919,135,66 by=clip drawn focus heard=CLOSE limit=1376,919,135,66 name=CompressedDiscsClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=1401.5,928,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,919,135,66 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=410,229,1101,627 by=panel fades=b limit=350.48,187,1219.05,822 overflow=672 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=410,229,1101,627
        8 StackPanel at=410,229,1095,1299 by=scroll in=7 limit=410,229,1101,627 role=none seen=410,229,1095,627
          9 HintText name=CompressedDiscsAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=410,361,1095,229.5 by=scroll heard="Convert or check" in=7 limit=410,229,1101,627 paints=#9C9CA0FF role=group
            11 TextBlock at=410,361,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=138.12,28.8 limit=410,361,1095,229.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Convert or check"
            12 HintText at=410,410.5,1095,108 box=730,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=726.78,72 limit=410,361,1095,229.5 paints=#86868AFF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=410,524.5,1095,66 by=clip heard="Convert or check" in=7 limit=410,361,1095,229.5 role=none
              14 Button at=410,524.5,310.5,66 by=clip drawn heard="CONVERT 3 CHOSEN" in=7 limit=410,361,1095,229.5 name=CompressedDiscsConvert off paints=#55555AFF,#9C9CA0FF role=button text="Convert 3 Chosen"
                15 TextBlock at=435.5,533.5,259.5,48 box=173,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT 3 CHOSEN" in=7 layout=172.69,31.2 limit=410,524.5,310.5,66 off owner=14 paints=#55555AFF role=text size=26 text="CONVERT 3 CHOSEN"
              16 Button at=732.5,524.5,276,66 by=clip drawn heard="CHECK 3 CHOSEN" in=7 limit=410,361,1095,229.5 name=CompressedDiscsCheck off paints=#55555AFF,#9C9CA0FF role=button text="Check 3 Chosen"
                17 TextBlock at=758,533.5,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK 3 CHOSEN" in=7 layout=149.53,31.2 limit=732.5,524.5,276,66 off owner=16 paints=#55555AFF role=text size=26 text="CHECK 3 CHOSEN"
              18 Button at=1020.5,524.5,117,66 by=clip drawn focus heard=STOP in=7 limit=410,361,1095,229.5 name=CompressedDiscsStop op paints=#55555AFF,#9C9CA0FF role=button text=Stop
                19 TextBlock at=1046,533.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=43.99,31.2 limit=1020.5,524.5,117,66 owner=18 paints=#9C9CA0FF role=text size=26 text=STOP
              20 Button at=1149.5,524.5,157.5,66 by=clip drawn heard=RESUME in=7 limit=410,361,1095,229.5 name=CompressedDiscsResume off paints=#55555AFF,#9C9CA0FF role=button text=Resume
                21 TextBlock at=1175,533.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=70.56,31.2 limit=1149.5,524.5,157.5,66 off owner=20 paints=#55555AFF role=text size=26 text=RESUME
          22 FieldRow at=410,614.5,1095,153 by=scroll heard=Progress in=7 limit=410,229,1101,627 paints=#9C9CA0FF role=group
            23 TextBlock at=410,614.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=71.38,28.8 limit=410,614.5,1095,153 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Progress
            24 StackPanel at=410,664,1095,103.5 by=clip heard=Progress in=7 limit=410,614.5,1095,153 role=none
              25 ProgressBar at=410,664,1095,6 by=clip drawn in=7 limit=410,614.5,1095,153 name=CompressedDiscsProgress paints=#333337FF,#4C9AE8FF role=progressbar
              26 TextBlock at=410,679,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin." in=7 layout=435.22,28.8 limit=410,614.5,1095,153 name=CompressedDiscsProgressText paints=#9C9CA0FF role=text size=24 text="1 of 3 discs done. Now: psx/Cobalt Harbor (Japan).bin."
              27 HintText name=CompressedDiscsLog paints=#86868AFF role=text volatile="its words name the run's own folders"
          28 FieldRow at=410,791.5,1095,736.5 by=scroll heard=Discs in=7 limit=410,229,1101,627 paints=#4C9AE8FF,#86868AFF,#9C9CA0FF role=group seen=410,791.5,1095,64.5
            29 TextBlock at=410,791.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Discs in=7 layout=43.75,28.8 limit=410,791.5,1095,736.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Discs
            30 HintText at=410,841,1095,36 box=730,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each disc's size and state; choose discs to convert or check only those." in=7 layout=483.2,24 limit=410,791.5,1095,736.5 paints=#86868AFF part role=text seen=410,841,1095,15 size=20 text="Each disc's size and state; choose discs to convert or check only those."
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
