# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Library-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case Library
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
                    33 LunaTable at=96,223.5,1036.5,645 by=panel heard=Rows limit=0,0,1920,1161 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=114,232.5,57,43.5 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                      35 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=162,229.5,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                      36 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,472.5,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                        37 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,72,48 role=none
                          38 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=171,229.5,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=36 paints=#D4D4D4FF role=text size=26 text=Game
                      39 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=634.5,229.5,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                      40 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,232.5,471,43.5 box=314,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=56.71,28.8 paints=#D4D4D4FF role=text size=24 text=Finding
                      41 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,288,1036.5,580.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,288,1036.5,580.5
                        42 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,288,1036.5,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          43 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,295.5,1006.5,46.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 role=none
                            44 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,295.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=42 paints=#D4D4D4FF role=text size=24 text="☐"
                            45 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,297,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=139.25,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                            46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,297,474,43.5 box=316,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=41 layout=203.02,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text="Same dump as 1 other file"
                        47 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,351,1036.5,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,358.5,1006.5,46.5 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 role=none
                            49 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,358.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=47 paints=#D4D4D4FF role=text size=24 text="☐"
                            50 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,360,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=205.63,28.8 owner=47 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA) [copier]"
                            51 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,360,474,43.5 box=316,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=41 layout=203.02,28.8 owner=47 paints=#D4D4D4FF role=edit size=24 text="Same dump as 1 other file"
                        52 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,414,1036.5,63 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          53 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,421.5,1006.5,46.5 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                            54 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,421.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=52 paints=#D4D4D4FF role=text size=24 text="☐"
                            55 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,423,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=218.33,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (Europe) (Rev 1)"
                            56 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,423,474,43.5 box=316,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=41 layout=310.03,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Same game as 1 other file (Amber Saga)"
                        57 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,477,1036.5,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          58 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,484.5,1006.5,46.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                            59 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,484.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=57 paints=#D4D4D4FF role=text size=24 text="☐"
                            60 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,486,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=139.25,28.8 owner=57 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                            61 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,486,474,43.5 box=316,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=41 layout=310.03,28.8 owner=57 paints=#D4D4D4FF role=edit size=24 text="Same game as 1 other file (Amber Saga)"
                        62 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,540,1036.5,63 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          63 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,547.5,1006.5,46.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 role=none
                            64 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,547.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=62 paints=#EEEEF0FF role=text size=24 text="☐"
                            65 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,549,475.5,43.5 box=317,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=41 layout=308.26,28.8 owner=62 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                            66 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,549,474,43.5 box=316,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Finding in=41 layout=315.94,28.8 owner=62 paints=#EEEEF0FF role=edit size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        67 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,603,1036.5,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          68 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,610.5,1006.5,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 role=none
                            69 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,610.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=67 paints=#D4D4D4FF role=text size=24 text="☐"
                            70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,612,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=169.97,28.8 owner=67 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Empty)"
                            71 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,612,474,43.5 box=316,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=41 layout=309.77,28.8 owner=67 paints=#D4D4D4FF role=edit size=24 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        72 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,666,1036.5,63 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          73 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,673.5,1006.5,46.5 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 role=none
                            74 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=111,673.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=41 layout=14.4,30.84 owner=72 paints=#D4D4D4FF role=text size=24 text="☐"
                            75 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,675,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=41 layout=221.18,28.8 owner=72 paints=#D4D4D4FF role=edit size=24 text="Granite Choir (Translated En)"
                            76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=643.5,675,474,43.5 box=316,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Finding in=41 layout=312.41,28.8 owner=72 paints=#D4D4D4FF role=edit size=24 text="Not recognised by ScreenScraper (a hint, not a fault)"
                77 Border at=96,868.5,1036.5,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
                  78 Grid at=102,874.5,1024.5,121.5 by=panel limit=0,0,1920,1161 role=none
                    79 Grid at=102,874.5,1024.5,43.5 by=panel limit=0,0,1920,1161 role=none
                      80 TextBlock at=114,874.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                    81 StackPanel at=973.5,930,153,66 by=panel limit=0,0,1920,1161 role=none
                      82 Button at=973.5,930,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        83 TextBlock at=999,939,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=973.5,930,153,66 off owner=82 paints=#707070FF role=text size=26 text="APPLY…"
              84 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                85 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 role=none
                  86 ScrollViewer at=1168.5,235.5,643.5,754.5 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1168.5,235.5,643.5,754.5
                    87 StackPanel at=1168.5,235.5,643.5,754.5 by=fixed-scroll in=86 limit=1168.5,235.5,643.5,754.5 name=LibraryDetail role=none
                      88 TextBlock at=1168.5,235.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=86 layout=361.46,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      89 TextBlock at=1168.5,331.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=86 layout=395.83,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      90 TextBlock at=1168.5,427.5,643.5,130.5 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=86 layout=419.14,86.4 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      91 TextBlock at=1168.5,567,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=86 layout=93.91,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      92 TextBlock at=1168.5,619.5,643.5,130.5 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=86 layout=419.9,86.4 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      93 TextBlock at=1168.5,759,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=86 layout=278.28,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      94 TextBlock at=1168.5,811.5,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=86 layout=158.35,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Shown in the library."
            95 Border at=96,1014,1728,66 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              96 WrapPanel at=96,1014,1728,66 by=panel limit=0,0,1920,1161 role=none
                97 ActionButton at=96,1014,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  98 TextBlock at=121.5,1023,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1014,115.5,66 off owner=97 paints=#707070FF role=text size=26 text=STOP
            99 HintBar at=96,1104,1728,33 by=panel heard="Hide…, Back, Section, List, Apply…, More" limit=0,0,1920,1161 role=group
focus 35
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
edge 25 down 42
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 36
edge 27 left 25
edge 27 right 12
edge 35 up 17
edge 35 down 42
edge 35 left 17
edge 35 right 36
edge 36 up 19
edge 36 down 42
edge 36 left 35
edge 36 right 25
edge 39 up 23
edge 39 down 42
edge 39 left 35
edge 39 right 25
edge 42 up 35
edge 42 down 47
edge 42 left same
edge 42 right 27
edge 47 up 42
edge 47 down 52
edge 47 left same
edge 47 right 27
edge 52 up 47
edge 52 down 57
edge 52 left same
edge 52 right 12
edge 57 up 52
edge 57 down 62
edge 57 left same
edge 57 right 12
edge 62 up 57
edge 62 down 67
edge 62 left same
edge 62 right 12
edge 67 up 62
edge 67 down 72
edge 67 left same
edge 67 right 12
edge 72 up 67
edge 72 down same
edge 72 left same
edge 72 right 12
press pageup focus="GridSplitter  (Resize Mark)" top=LibraryWindow
press pagedown focus="GridSplitter  (Resize Mark)" top=LibraryWindow
press first focus="GridSplitter  (Resize Mark)" top=LibraryWindow
press last focus="GridSplitter  (Resize Mark)" top=LibraryWindow
press back top=none
