# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case LibraryRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 LibraryPane at=72,42,1776,1107 by=panel heard="Library Manager" limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 Border at=72,42,1776,76.5 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1161 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                7 TextBlock at=1273.5,72,550.5,36 box=367,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=366.62,24 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=96,60,1159.5,58.5 by=panel limit=0,0,1920,1161 role=none
                  9 TextBlock at=96,60,1159.5,58.5 box=773,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1161 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
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
                    33 LunaTable at=96,223.5,1036.5,645 by=panel heard=Rows limit=0,0,1920,1161 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=114,232.5,57,43.5 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                      35 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=162,229.5,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                      36 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,472.5,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                        37 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,72,48 role=none
                          38 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=36 paints=#D4D4D4FF role=text size=26 text=Game
                      39 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=634.5,229.5,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                      40 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,232.5,471,43.5 box=314,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=56.71,28.8 paints=#D4D4D4FF role=text size=24 text=Finding
                      41 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,288,1036.5,580.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,288,1036.5,580.5
                42 Border at=96,868.5,1036.5,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
                  43 Grid at=102,874.5,1024.5,121.5 by=panel limit=0,0,1920,1161 role=none
                    44 Grid at=102,874.5,1024.5,43.5 by=panel limit=0,0,1920,1161 role=none
                      45 ProgressBar at=102,892.5,180,6 by=panel drawn heard=Progress limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                      46 TextBlock at=294,874.5,832.5,43.5 box=555,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=343.82,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    47 StackPanel at=973.5,930,153,66 by=panel limit=0,0,1920,1161 role=none
                      48 Button at=973.5,930,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        49 TextBlock at=999,939,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=973.5,930,153,66 off owner=48 paints=#707070FF role=text size=26 text="APPLY…"
              50 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                51 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 role=none
                  52 TextBlock at=1168.5,235.5,643.5,754.5 box=429,503 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=409.18,144 limit=0,0,1920,1161 name=LibraryOverview paints=#D4D4D4FF role=text size=24 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            53 Border at=96,1014,1728,66 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              54 WrapPanel at=96,1014,1728,66 by=panel limit=0,0,1920,1161 role=none
                55 ActionButton at=96,1014,115.5,66 by=panel drawn focus heard=STOP limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                  56 TextBlock at=121.5,1023,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1014,115.5,66 owner=55 paints=#D4D4D4FF role=text size=26 text=STOP
            57 HintBar at=96,1104,1728,33 by=panel heard="Back, Section, List, Apply…, More" limit=0,0,1920,1161 role=group
focus 12
edge 12 up same
edge 12 down 36
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 35
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 36
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 36
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 39
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 36
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 36
edge 27 left 25
edge 27 right 12
edge 35 up 17
edge 35 down 55
edge 35 left 17
edge 35 right 36
edge 36 up 19
edge 36 down 39
edge 36 left 35
edge 36 right 25
edge 39 up 23
edge 39 down 55
edge 39 left 35
edge 39 right 25
edge 55 up 35
edge 55 down same
edge 55 left same
edge 55 right 25
press pageup focus="TextBox Search files (Search)" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" top=LibraryWindow
