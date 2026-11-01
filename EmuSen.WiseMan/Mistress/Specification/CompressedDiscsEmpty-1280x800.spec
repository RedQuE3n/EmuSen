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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 DockPanel at=274,152,733,504 by=panel limit=234,124,812.35,548 role=none
      2 StackPanel at=274,580,733,76 by=panel limit=234,124,812.35,548 role=none
        3 HintText at=274,580,733,24 by=panel drawn limit=234,124,812.35,548 name=CompressedDiscsStatus paints=#8D8D91FF role=text
        4 ButtonBar at=920,612,87,44 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=toolbar
          5 Button at=920,612,87,44 by=clip drawn focus heard=CLOSE limit=920,612,87,44 name=CompressedDiscsClose op paints=#707070FF,#D4D4D4FF role=button text=Close
            6 TextBlock at=937,618,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=920,612,87,44 owner=5 paints=#EEEEF0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=274,152,733,418 by=panel fades=b limit=234,124,812.35,548 overflow=182 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=274,152,733,418
        8 StackPanel at=274,152,729,600 by=scroll in=7 limit=274,152,733,418 paints=#8D8D91FF,#D4D4D4FF role=none seen=274,152,729,418
          9 HintText name=CompressedDiscsAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          10 FieldRow at=274,240,729,153 by=scroll heard="Convert or check" in=7 limit=274,152,733,418 paints=#D4D4D4FF role=group
            11 TextBlock at=274,240,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Convert or check" in=7 layout=131.26,28.8 limit=274,240,729,153 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Convert or check"
            12 HintText at=274,273,729,72 box=729,72 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left." in=7 layout=724.46,72 limit=274,240,729,153 paints=#8D8D91FF part role=text size=20 text="The discs chosen below, or with none chosen every disc shown. A CHD is kept only when three checks pass: chdman's own, every track read back against the original, and the table of contents. Closing this window stops a batch; converting again does only what is left."
            13 WrapPanel at=274,349,729,44 by=clip heard="Convert or check" in=7 limit=274,240,729,153 role=none
              14 Button at=274,349,210,44 by=clip drawn heard="CONVERT EVERY DISC" in=7 limit=274,240,729,153 name=CompressedDiscsConvert off paints=#707070FF,#D4D4D4FF role=button text="Convert Every Disc"
                15 TextBlock at=291,355,176,32 box=176,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT EVERY DISC" in=7 layout=175.53,31.2 limit=274,349,210,44 off owner=14 paints=#707070FF role=text size=26 text="CONVERT EVERY DISC"
              16 Button at=492,349,192,44 by=clip drawn heard="CHECK CONVERTED" in=7 limit=274,240,729,153 name=CompressedDiscsCheck off paints=#707070FF,#D4D4D4FF role=button text="Check Converted"
                17 TextBlock at=509,355,158,32 box=158,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK CONVERTED" in=7 layout=157.17,31.2 limit=492,349,192,44 off owner=16 paints=#707070FF role=text size=26 text="CHECK CONVERTED"
              18 Button at=692,349,77,44 by=clip drawn heard=STOP in=7 limit=274,240,729,153 name=CompressedDiscsStop off paints=#707070FF,#D4D4D4FF role=button text=Stop
                19 TextBlock at=709,355,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP in=7 layout=42.17,31.2 limit=692,349,77,44 off owner=18 paints=#707070FF role=text size=26 text=STOP
              20 Button at=777,349,101,44 by=clip drawn heard=RESUME in=7 limit=274,240,729,153 name=CompressedDiscsResume off paints=#707070FF,#D4D4D4FF role=button text=Resume
                21 TextBlock at=794,355,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=RESUME in=7 layout=66.64,31.2 limit=777,349,101,44 off owner=20 paints=#707070FF role=text size=26 text=RESUME
          22 FieldRow at=274,409,729,62 by=scroll heard=Progress in=7 limit=274,152,733,418 paints=#D4D4D4FF role=group
            23 TextBlock at=274,409,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Progress in=7 layout=68.93,28.8 limit=274,409,729,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Progress
            24 StackPanel at=274,442,729,29 by=clip heard=Progress in=7 limit=274,409,729,62 role=none
              25 TextBlock at=274,442,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="No batch has run since this window opened." in=7 layout=343.3,28.8 limit=274,409,729,62 name=CompressedDiscsProgressText paints=#D4D4D4FF role=text size=24 text="No batch has run since this window opened."
          26 FieldRow at=274,487,729,107 by=scroll heard="Output folder" in=7 limit=274,152,733,418 paints=#D4D4D4FF role=group seen=274,487,729,83
            27 TextBlock at=274,487,729,29 box=729,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Output folder" in=7 layout=101.83,28.8 limit=274,487,729,107 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Output folder"
            28 HintText at=274,520,729,24 box=729,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Kept for every batch. It may not be inside the library, nor hold it." in=7 layout=412.34,24 limit=274,487,729,107 paints=#8D8D91FF part role=text size=20 text="Kept for every batch. It may not be inside the library, nor hold it."
            29 PathPickerRow at=274,548,729,46 by=clip heard="Output folder" in=7 limit=274,487,729,107 name=CompressedDiscsOutput paints=#D4D4D4FF role=group seen=274,548,729,22
              30 TextBox at=274,548,602,46 drawn focus heard="Choose the Folder for CHDs" in=7 name=PART_Path op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit seen=274,548,602,22
                31 ScrollViewer allow="a text box scrolls its own text" at=285,555,580,32 in=7 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer seen=285,555,580,15
                  32 TextBlock name=PART_Placeholder paints=#8D8D91FF part role=text volatile="its words name the run's own folders"
              33 Button at=884,549,119,44 drawn focus heard=BROWSE... in=7 name=PART_Browse op paints=#707070FF,#D4D4D4FF part role=button seen=884,549,119,21 text=Browse...
                34 TextBlock at=901,555,85,32 box=85,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=BROWSE... in=7 layout=84.71,31.2 limit=884,549,119,44 owner=33 paints=#D4D4D4FF role=text seen=901,555,85,15 size=26 text=BROWSE...
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
