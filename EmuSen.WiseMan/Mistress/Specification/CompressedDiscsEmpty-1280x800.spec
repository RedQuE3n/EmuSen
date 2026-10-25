# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CompressedDiscsEmpty-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Compressed Discs"
case CompressedDiscsEmpty
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 StackPanel at=274,580,733,76 by=panel limit=234,124,812.35,548 role=none
        3 HintText at=274,580,733,24 by=panel drawn limit=234,124,812.35,548 name=CompressedDiscsStatus paints=#86868AFF role=text
        4 ButtonBar at=917,612,90,44 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=toolbar
          5 Button at=917,612,90,44 by=clip drawn focus heard=CLOSE limit=917,612,90,44 name=CompressedDiscsClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=934,618,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=917,612,90,44 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=274,152,733,418 by=panel fades=b limit=234,124,812.35,548 overflow=59 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=274,152,733,418
        8 StackPanel at=274,152,729,477 by=scroll in=7 limit=274,152,733,418 role=none seen=274,152,729,418
          9 HintText name=CompressedDiscsAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=274,240,729,153 by=scroll heard="Convert or check" in=7 limit=274,152,733,418 paints=#9C9CA0FF role=group
            11 TextBlock at=274,240,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=138.12,28.8 limit=274,240,729,153 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Convert or check"
            12 HintText at=274,273,729,72 box=729,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=726.78,72 limit=274,240,729,153 paints=#86868AFF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=274,349,729,44 by=clip heard="Convert or check" in=7 limit=274,240,729,153 role=none
              14 Button at=274,349,222,44 by=clip drawn heard="CONVERT EVERY DISC" in=7 limit=274,240,729,153 name=CompressedDiscsConvert off paints=#55555AFF,#9C9CA0FF role=button text="Convert Every Disc"
                15 TextBlock at=291,355,188,32 box=188,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT EVERY DISC" in=7 layout=187.8,31.2 limit=274,349,222,44 off owner=14 paints=#55555AFF role=text size=26 text="CONVERT EVERY DISC"
              16 Button at=504,349,202,44 by=clip drawn heard="CHECK CONVERTED" in=7 limit=274,240,729,153 name=CompressedDiscsCheck off paints=#55555AFF,#9C9CA0FF role=button text="Check Converted"
                17 TextBlock at=521,355,168,32 box=168,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK CONVERTED" in=7 layout=167.99,31.2 limit=504,349,202,44 off owner=16 paints=#55555AFF role=text size=26 text="CHECK CONVERTED"
              18 Button at=714,349,78,44 by=clip drawn heard=STOP in=7 limit=274,240,729,153 name=CompressedDiscsStop off paints=#55555AFF,#9C9CA0FF role=button text=Stop
                19 TextBlock at=731,355,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=43.99,31.2 limit=714,349,78,44 off owner=18 paints=#55555AFF role=text size=26 text=STOP
              20 Button at=800,349,105,44 by=clip drawn heard=RESUME in=7 limit=274,240,729,153 name=CompressedDiscsResume off paints=#55555AFF,#9C9CA0FF role=button text=Resume
                21 TextBlock at=817,355,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=70.56,31.2 limit=800,349,105,44 off owner=20 paints=#55555AFF role=text size=26 text=RESUME
          22 FieldRow at=274,409,729,62 by=scroll heard=Progress in=7 limit=274,152,733,418 paints=#9C9CA0FF role=group
            23 TextBlock at=274,409,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=71.38,28.8 limit=274,409,729,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Progress
            24 StackPanel at=274,442,729,29 by=clip heard=Progress in=7 limit=274,409,729,62 role=none
              25 TextBlock at=274,442,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No batch has run since this window opened." in=7 layout=358.42,28.8 limit=274,409,729,62 name=CompressedDiscsProgressText paints=#9C9CA0FF role=text size=24 text="No batch has run since this window opened."
          26 FieldRow at=274,487,729,142 by=scroll heard=Discs in=7 limit=274,152,733,418 paints=#9C9CA0FF role=group seen=274,487,729,83
            27 TextBlock at=274,487,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Discs in=7 layout=43.75,28.8 limit=274,487,729,142 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Discs
            28 HintText at=274,520,729,24 box=729,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Each disc's size and state; choose discs to convert or check only those." in=7 layout=483.2,24 limit=274,487,729,142 paints=#86868AFF part role=text size=20 text="Each disc's size and state; choose discs to convert or check only those."
            29 ContentControl at=274,548,729,81 by=clip heard=Discs in=7 limit=274,487,729,142 paints=#9C9CA0FF role=none seen=274,548,729,22
              30 EmptyState at=278,548,720,81 by=clip heard="No discs to convert" in=7 limit=274,548,729,81 name=CompressedDiscsEmpty paints=#86868AFF,#9C9CA0FF role=text seen=278,548,720,22
                31 TextBlock at=559,548,158,29 box=158,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No discs to convert" in=7 layout=157.13,28.8 limit=278,548,720,81 name=PART_Message paints=#86868AFF part role=text seen=559,548,158,22 size=24 text="No discs to convert"
focus 5
edge 5 up same
edge 5 down same
edge 5 left same
edge 5 right same
press pageup focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press pagedown focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press first focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press last focus="Button 'Close'" title="Compressed Discs" top=CompressedDiscsWindow
press back top=none
