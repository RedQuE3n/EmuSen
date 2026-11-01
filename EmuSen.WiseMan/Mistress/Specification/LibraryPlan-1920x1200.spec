# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case LibraryPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 LibraryPane at=72,42,1776,1107 by=panel heard="Library Manager" limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 Border at=72,42,1776,76.5 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1161 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                7 TextBlock at=1279.5,72,544.5,36 box=363,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=362.22,24 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=96,60,1165.5,58.5 by=panel limit=0,0,1920,1161 role=none
                  9 TextBlock at=96,60,1165.5,58.5 box=777,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1161 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=13 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 role=none
                  17 ToggleButton at=96,138,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    18 TextBlock at=121.5,147,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,138,97.5,66 owner=17 paints=#EEEEF0FF role=text size=26 text=ALL
                  19 ToggleButton at=205.5,138,196.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="Same Dump"
                    20 TextBlock at=231,147,145.5,48 box=97,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=96.28,31.2 limit=205.5,138,196.5,66 owner=19 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
                  21 ToggleButton at=414,138,193.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="Same Game"
                    22 TextBlock at=439.5,147,142.5,48 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=94.48,31.2 limit=414,138,193.5,66 owner=21 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
                  23 ToggleButton at=619.5,138,312,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="Broken or Suspect"
                    24 TextBlock at=645,147,261,48 box=174,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="BROKEN OR SUSPECT" layout=173.78,31.2 limit=619.5,138,312,66 owner=23 paints=#FFFFFFFF role=text size=26 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=943.5,138,243,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Unrecognised
                    26 TextBlock at=969,147,192,48 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UNRECOGNISED layout=127.35,31.2 limit=943.5,138,243,66 owner=25 paints=#FFFFFFFF role=text size=26 text=UNRECOGNISED
                  27 ToggleButton at=1198.5,138,144,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Hidden
                    28 TextBlock at=1224,147,93,48 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDDEN layout=61.41,31.2 limit=1198.5,138,144,66 owner=27 paints=#FFFFFFFF role=text size=26 text=HIDDEN
            29 Grid at=96,223.5,1728,778.5 by=panel limit=0,0,1920,1161 role=none
              30 Grid at=96,223.5,1036.5,778.5 by=panel limit=0,0,1920,1161 role=none
                31 Border at=96,223.5,1036.5,645 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                  32 Grid at=96,223.5,1036.5,645 by=panel limit=0,0,1920,1161 role=none
                    33 Border at=96,231,1036.5,630 by=panel limit=0,0,1920,1161 paints=#1B1B1EFF,#707070FF role=none
                      34 DockPanel at=121.5,256.5,985.5,579 by=panel limit=0,0,1920,1161 role=none
                        35 TextBlock at=121.5,256.5,985.5,58.5 box=657,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        36 StackPanel at=121.5,697.5,985.5,138 by=panel limit=0,0,1920,1161 role=none
                          37 Button at=121.5,697.5,985.5,66 by=panel drawn focus heard=No limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=No
                            38 TextBlock at=147,706.5,36,48 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=121.5,697.5,985.5,66 owner=37 paints=#EEEEF0FF role=text size=26 text=NO
                          39 Button at=121.5,769.5,985.5,66 by=panel drawn focus heard=Yes limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            40 TextBlock at=147,778.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=121.5,769.5,985.5,66 owner=39 paints=#D4D4D4FF role=text size=26 text=YES
                        41 ScrollViewer at=121.5,333,985.5,346.5 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=121.5,333,985.5,346.5
                          42 TextBlock at=121.5,333,985.5,346.5 box=657,231 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=41 layout=655.01,230.4 limit=121.5,333,985.5,346.5 paints=#D4D4D4FF role=text size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
                43 Border at=96,868.5,1036.5,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
                  44 Grid at=102,874.5,1024.5,121.5 by=panel limit=0,0,1920,1161 role=none
                    45 Grid at=102,874.5,1024.5,43.5 by=panel limit=0,0,1920,1161 role=none
                      46 TextBlock at=114,874.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="2 to hide"
                    47 StackPanel at=973.5,930,153,66 by=panel limit=0,0,1920,1161 role=none
                      48 Button at=973.5,930,153,66 by=panel drawn focus heard="Apply the marked changes" limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        49 TextBlock at=999,939,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=973.5,930,153,66 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              50 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                51 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 role=none
                  52 ScrollViewer at=1168.5,235.5,643.5,754.5 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1168.5,235.5,643.5,754.5
                    53 StackPanel at=1168.5,235.5,643.5,754.5 by=fixed-scroll in=52 limit=1168.5,235.5,643.5,754.5 name=LibraryDetail role=none
                      54 TextBlock at=1168.5,235.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=52 layout=361.46,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      55 TextBlock at=1168.5,331.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=52 layout=395.83,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      56 TextBlock at=1168.5,427.5,643.5,130.5 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=52 layout=419.14,86.4 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      57 TextBlock at=1168.5,567,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=52 layout=93.91,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      58 TextBlock at=1168.5,619.5,643.5,130.5 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=52 layout=419.9,86.4 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      59 TextBlock at=1168.5,759,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=52 layout=278.28,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      60 TextBlock at=1168.5,811.5,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=52 layout=158.35,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Shown in the library."
            61 Border at=96,1014,1728,66 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              62 WrapPanel at=96,1014,1728,66 by=panel limit=0,0,1920,1161 role=none
                63 ActionButton at=96,1014,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  64 TextBlock at=121.5,1023,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1014,115.5,66 off owner=63 paints=#707070FF role=text size=26 text=STOP
            65 HintBar at=96,1104,1728,33 by=panel heard="Hide…, Back, Section, List, Apply…, More" limit=0,0,1920,1161 role=group
focus 37
edge 12 up same
edge 12 down 37
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 37
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 37
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 37
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 37
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 37
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 37
edge 27 left 25
edge 27 right 12
edge 37 up 17
edge 37 down 39
edge 37 left same
edge 37 right 12
edge 39 up 37
edge 39 down 48
edge 39 left same
edge 39 right 12
edge 48 up 39
edge 48 down same
edge 48 left 19
edge 48 right 27
press pageup focus="Button 'No' (No)" top=LibraryWindow
press pagedown focus="Button 'No' (No)" top=LibraryWindow
press first focus="Button 'No' (No)" top=LibraryWindow
press last focus="Button 'No' (No)" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" top=LibraryWindow
