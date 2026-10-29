# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CompressedDiscsEmpty-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Compressed Discs"
case CompressedDiscsEmpty
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 StackPanel at=410,871,1101,114 by=panel limit=350.48,187,1219.05,822 role=none
        3 HintText at=410,871,1101,36 by=panel drawn limit=350.48,187,1219.05,822 name=CompressedDiscsStatus paints=#86868AFF role=text
        4 ButtonBar at=1376,919,135,66 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=toolbar
          5 Button at=1376,919,135,66 by=clip drawn focus heard=CLOSE limit=1376,919,135,66 name=CompressedDiscsClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=1401.5,928,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1376,919,135,66 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=410,229,1101,627 by=panel fades=b limit=350.48,187,1219.05,822 overflow=273 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=410,229,1101,627
        8 StackPanel at=410,229,1095,900 by=scroll in=7 limit=410,229,1101,627 paints=#86868AFF,#9C9CA0FF role=none seen=410,229,1095,627
          9 HintText name=CompressedDiscsAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=410,361,1095,229.5 by=scroll heard="Convert or check" in=7 limit=410,229,1101,627 paints=#9C9CA0FF role=group
            11 TextBlock at=410,361,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=138.12,28.8 limit=410,361,1095,229.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Convert or check"
            12 HintText at=410,410.5,1095,108 box=730,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=726.78,72 limit=410,361,1095,229.5 paints=#86868AFF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=410,524.5,1095,66 by=clip heard="Convert or check" in=7 limit=410,361,1095,229.5 role=none
              14 Button at=410,524.5,333,66 by=clip drawn heard="CONVERT EVERY DISC" in=7 limit=410,361,1095,229.5 name=CompressedDiscsConvert off paints=#55555AFF,#9C9CA0FF role=button text="Convert Every Disc"
                15 TextBlock at=435.5,533.5,282,48 box=188,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT EVERY DISC" in=7 layout=187.8,31.2 limit=410,524.5,333,66 off owner=14 paints=#55555AFF role=text size=26 text="CONVERT EVERY DISC"
              16 Button at=755,524.5,303,66 by=clip drawn heard="CHECK CONVERTED" in=7 limit=410,361,1095,229.5 name=CompressedDiscsCheck off paints=#55555AFF,#9C9CA0FF role=button text="Check Converted"
                17 TextBlock at=780.5,533.5,252,48 box=168,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK CONVERTED" in=7 layout=167.99,31.2 limit=755,524.5,303,66 off owner=16 paints=#55555AFF role=text size=26 text="CHECK CONVERTED"
              18 Button at=1070,524.5,117,66 by=clip drawn heard=STOP in=7 limit=410,361,1095,229.5 name=CompressedDiscsStop off paints=#55555AFF,#9C9CA0FF role=button text=Stop
                19 TextBlock at=1095.5,533.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=43.99,31.2 limit=1070,524.5,117,66 off owner=18 paints=#55555AFF role=text size=26 text=STOP
              20 Button at=1199,524.5,157.5,66 by=clip drawn heard=RESUME in=7 limit=410,361,1095,229.5 name=CompressedDiscsResume off paints=#55555AFF,#9C9CA0FF role=button text=Resume
                21 TextBlock at=1224.5,533.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=70.56,31.2 limit=1199,524.5,157.5,66 off owner=20 paints=#55555AFF role=text size=26 text=RESUME
          22 FieldRow at=410,614.5,1095,93 by=scroll heard=Progress in=7 limit=410,229,1101,627 paints=#9C9CA0FF role=group
            23 TextBlock at=410,614.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=71.38,28.8 limit=410,614.5,1095,93 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Progress
            24 StackPanel at=410,664,1095,43.5 by=clip heard=Progress in=7 limit=410,614.5,1095,93 role=none
              25 TextBlock at=410,664,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No batch has run since this window opened." in=7 layout=358.42,28.8 limit=410,614.5,1095,93 name=CompressedDiscsProgressText paints=#9C9CA0FF role=text size=24 text="No batch has run since this window opened."
          26 FieldRow at=410,731.5,1095,160.5 by=scroll heard="Output folder" in=7 limit=410,229,1101,627 paints=#9C9CA0FF role=group seen=410,731.5,1095,124.5
            27 TextBlock at=410,731.5,1095,43.5 box=730,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Output folder" in=7 layout=107.06,28.8 limit=410,731.5,1095,160.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Output folder"
            28 HintText at=410,781,1095,36 box=730,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Kept for every batch. It may not be inside the library, nor hold it." in=7 layout=433.48,24 limit=410,731.5,1095,160.5 paints=#86868AFF part role=text size=20 text="Kept for every batch. It may not be inside the library, nor hold it."
            29 PathPickerRow at=410,823,1095,69 by=clip heard="Output folder" in=7 limit=410,731.5,1095,160.5 name=CompressedDiscsOutput paints=#9C9CA0FF role=group seen=410,823,1095,33
              30 TextBox at=410,823,900,69 drawn focus heard="Choose the Folder for CHDs" in=7 name=PART_Path op paints=#0E0E10FF,#55555AFF,#D2D2D6FF part role=edit seen=410,823,900,33
                31 ScrollViewer allow="a text box scrolls its own text" at=426.5,833.5,867,48 in=7 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer seen=426.5,833.5,867,22.5
                  32 TextBlock name=PART_Placeholder paints=#86868AFF part role=text volatile="its words name the run's own folders"
              33 Button at=1322,824.5,183,66 drawn focus heard=BROWSE... in=7 name=PART_Browse op paints=#55555AFF,#9C9CA0FF part role=button seen=1322,824.5,183,31.5 text=Browse...
                34 TextBlock at=1347.5,833.5,132,48 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=BROWSE... in=7 layout=87.54,31.2 limit=1322,824.5,183,66 owner=33 paints=#9C9CA0FF role=text seen=1347.5,833.5,132,22.5 size=26 text=BROWSE...
focus 5
edge 5 up 33
edge 5 down same
edge 5 left 30
edge 5 right same
edge 30 up same
edge 30 down 5
edge 30 left same
edge 30 right 33
edge 33 up same
edge 33 down 5
edge 33 left 30
edge 33 right same
press pageup focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press pagedown focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press first focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press last focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press back top=none
