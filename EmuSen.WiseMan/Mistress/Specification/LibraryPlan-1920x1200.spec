# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case LibraryPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 LibraryPane at=72,42,1776,1146 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1143,72,681,36 box=454,24 by=panel caps=13.999 face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=96,60,1029,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1029,58.5 box=686,39 by=panel caps=22.3984 face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,202.5,66 by=panel focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=231,148.5,154.5,36 box=103,24 by=clip caps=13.999 face=Barlow heard="SAME DUMP" layout=102.88,24 limit=207,136.5,202.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=421.5,136.5,199.5,66 by=panel focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=445.5,148.5,151.5,36 box=101,24 by=clip caps=13.999 face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,136.5,199.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=633,136.5,324,66 by=panel focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=657,148.5,276,36 box=184,24 by=clip caps=13.999 face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=633,136.5,324,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=969,136.5,253.5,66 by=panel focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=993,148.5,205.5,36 box=137,24 by=clip caps=13.999 face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=969,136.5,253.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=1234.5,136.5,148.5,66 by=panel focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=1258.5,148.5,100.5,36 box=67,24 by=clip caps=13.999 face=Barlow heard=HIDDEN layout=66.66,24 limit=1234.5,136.5,148.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    33 LunaTable at=96,220.5,1036.5,834 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,237,1036.5,817.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,817.5
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,237,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,252,964.5,49.5 in=34 paints=#707070FF role=none
                            37 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,252,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 role=none
                              38 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,252,118.5,48 in=34 owner=35 paints=#4C9AE8FF role=text
                              39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,252,369,48 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              40 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,252,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=236.7,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        41 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,318,1036.5,81 focus heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          42 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,333,964.5,51 in=34 paints=#707070FF role=none
                            43 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,333,964.5,49.5 heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 role=none
                              44 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,333,118.5,49.5 box=79,33 caps=18.916 face="Adwaita Sans" heard="✓ hide" in=34 layout=78.07,32.27 owner=41 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                              45 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,333,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.79,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                              46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,333,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=236.7,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        47 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,399,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          48 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,414,964.5,49.5 in=34 paints=#707070FF role=none
                            49 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,414,964.5,48 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                              50 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,414,118.5,48 in=34 owner=47 paints=#4C9AE8FF role=text
                              51 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,414,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.66,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                              52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,414,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=234.49,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        53 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,480,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          54 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,495,964.5,49.5 in=34 paints=#707070FF role=none
                            55 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,495,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                              56 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,495,118.5,48 in=34 owner=53 paints=#4C9AE8FF role=text
                              57 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,495,369,48 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,495,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=234.49,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        59 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,561,1036.5,81 focus heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          60 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,576,964.5,51 in=34 paints=#707070FF role=none
                            61 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,576,964.5,49.5 heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 role=none
                              62 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,576,118.5,49.5 box=79,33 caps=18.916 face="Adwaita Sans" heard="✓ hide" in=34 layout=78.07,32.27 owner=59 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                              63 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,576,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=233.14,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,576,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=233.01,31.2 owner=59 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        65 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,642,1036.5,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 op paints=#D4D4D4FF role=listitem
                          66 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,657,964.5,49.5 in=34 paints=#707070FF role=none
                            67 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,657,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 role=none
                              68 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,657,118.5,48 in=34 owner=65 paints=#4C9AE8FF role=text
                              69 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,657,369,48 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=240.99,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                              70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,657,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=243.8,31.2 owner=65 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        71 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,723,1036.5,81 focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 op paints=#D4D4D4FF role=listitem
                          72 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,738,964.5,49.5 in=34 paints=#707070FF role=none
                            73 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,738,964.5,48 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 role=none
                              74 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,738,118.5,48 in=34 owner=71 paints=#4C9AE8FF role=text
                              75 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,738,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=243.46,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                              76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=691.5,738,369,48 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=238.26,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                77 Border at=96,1054.5,1036.5,64.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  78 Grid at=102,1060.5,1024.5,52.5 by=panel limit=0,0,1920,1200 role=none
                    79 Grid at=102,1060.5,1024.5,40.5 by=panel limit=0,0,1920,1200 role=none
                      80 TextBlock at=114,1060.5,1012.5,40.5 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 to hide" layout=78.96,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 to hide"
              81 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                82 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  83 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel fades=b limit=0,0,1920,1200 overflow=24 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    84 StackPanel at=1204.5,268.5,571.5,826.5 by=scroll in=83 limit=1204.5,268.5,571.5,802.5 name=LibraryDetail role=none seen=1204.5,268.5,571.5,802.5
                      85 TextBlock at=1204.5,268.5,571.5,174 box=381,116 by=scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=83 layout=365.18,115.2 limit=1204.5,268.5,571.5,802.5 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      86 TextBlock at=1204.5,451.5,571.5,120 box=381,80 by=scroll caps=15.3989 face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=83 layout=353.52,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      87 TextBlock at=1204.5,580.5,571.5,198 box=381,132 by=scroll caps=15.3989 face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=83 layout=371.43,132 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      88 TextBlock at=1204.5,787.5,571.5,40.5 box=381,27 by=scroll caps=15.3989 face=Barlow heard="System: psx" in=83 layout=113.15,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      89 TextBlock at=1204.5,837,571.5,159 box=381,106 by=scroll caps=15.3989 face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=83 layout=379.59,105.6 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      90 TextBlock at=1204.5,1005,571.5,40.5 box=381,27 by=scroll caps=15.3989 face=Barlow heard="Header checksum: not checked yet." in=83 layout=332.13,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
                      91 TextBlock at=1204.5,1054.5,571.5,40.5 box=381,27 by=scroll caps=15.3989 face=Barlow heard="Shown in the library." in=83 layout=189.07,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text seen=1204.5,1054.5,571.5,16.5 size=22 text="Shown in the library."
            92 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              93 HintBar at=144,1135.5,1632,43.5 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          94 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            95 Border at=420,276,1080,679.5 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              96 DockPanel at=469.5,325.5,981,580.5 by=panel limit=0,0,1920,1200 role=none
                97 TextBlock at=469.5,325.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                98 StackPanel at=469.5,738,981,168 by=panel limit=0,0,1920,1200 role=none
                  99 Button at=469.5,738,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    100 TextBlock at=511.5,754.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,738,981,81 owner=99 paints=#EEEEF0FF role=text size=26 text=No
                  101 Button at=469.5,825,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    102 TextBlock at=511.5,841.5,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,825,981,81 owner=101 paints=#D4D4D4FF role=text size=26 text=Yes
                103 ScrollViewer at=469.5,402,981,318 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,402,981,318
                  104 TextBlock at=469.5,402,981,318 box=654,212 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=103 layout=653.33,211.2 limit=469.5,402,981,318 paints=#D4D4D4FF role=text size=22 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
focus 99
edge 12 up same
edge 12 down 35
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 35
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 35
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 35
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 35
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 35
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 99
edge 27 left 25
edge 27 right 12
edge 35 up 17
edge 35 down 41
edge 35 left same
edge 35 right 27
edge 41 up 35
edge 41 down 47
edge 41 left same
edge 41 right 27
edge 47 up 41
edge 47 down 53
edge 47 left same
edge 47 right 27
edge 53 up 47
edge 53 down 59
edge 53 left same
edge 53 right 12
edge 59 up 53
edge 59 down 65
edge 59 left same
edge 59 right 12
edge 65 up 59
edge 65 down 71
edge 65 left same
edge 65 right 12
edge 71 up 65
edge 71 down 101
edge 71 left same
edge 71 right 99
edge 99 up 65
edge 99 down 101
edge 99 left 71
edge 99 right 27
edge 101 up 99
edge 101 down same
edge 101 left 19
edge 101 right 27
press pageup focus="Button  (No)" top=LibraryWindow
press pagedown focus="Button  (No)" top=LibraryWindow
press first focus="Button  (No)" top=LibraryWindow
press last focus="Button  (No)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there)" top=LibraryWindow
