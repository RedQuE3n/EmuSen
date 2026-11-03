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
      2 LibraryPane at=72,42,1776,1098 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1143,78,681,36 box=454,24 by=panel caps=13.999 face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=96,66,1029,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1029,58.5 box=686,39 by=panel caps=22.3984 face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel focus heard=Search limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  17 SectionTab at=120,150,51,66 by=panel focus heard=All limit=0,0,1920,1200 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=219,150,154.5,66 by=panel focus heard="Same Dump" limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text="Same Dump"
                    20 TextBlock at=219,162,154.5,36 box=103,24 by=clip caps=13.999 face=Barlow heard="SAME DUMP" layout=102.88,24 limit=219,150,154.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 SectionTab at=421.5,150,151.5,66 by=panel focus heard="Same Game" limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text="Same Game"
                    22 TextBlock at=421.5,162,151.5,36 box=101,24 by=clip caps=13.999 face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,150,151.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 SectionTab at=621,150,276,66 by=panel focus heard="Broken or Suspect" limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    24 TextBlock at=621,162,276,36 box=184,24 by=clip caps=13.999 face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=621,150,276,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 SectionTab at=945,150,205.5,66 by=panel focus heard=Unrecognised limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text=Unrecognised
                    26 TextBlock at=945,162,205.5,36 box=137,24 by=clip caps=13.999 face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=945,150,205.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 SectionTab at=1198.5,150,100.5,66 by=panel focus heard=Hidden limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text=Hidden
                    28 TextBlock at=1198.5,162,100.5,36 box=67,24 by=clip caps=13.999 face=Barlow heard=HIDDEN layout=66.66,24 limit=1198.5,150,100.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    33 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          36 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=34 role=none
                            37 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              38 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 role=none
                                39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,255,387,48 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                40 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,255,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=249.81,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        41 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,81 focus heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          42 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=34 role=none
                            43 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              44 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 role=none
                                45 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,336,118.5,51 box=79,34 caps=19.1128 face="Adwaita Mono" heard="✓ hide" in=34 layout=78.96,33.41 owner=41 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                                46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,336,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.79,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                                47 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,336,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=249.81,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        48 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          49 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=34 role=none
                            50 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              51 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,417,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=256.65,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                                53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,417,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=257.04,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        54 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,81 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=34 role=none
                            56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,483,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,498,387,48 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,498,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=257.04,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,81 focus heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 paints=#333337FF,#EEEEF0FF role=listitem
                          61 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=34 role=none
                            62 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=34 paints=#26262BFF,#4C9AE8FF role=none
                              63 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,564,1000.5,79.5 heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 role=none
                                64 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,579,118.5,51 box=79,34 caps=19.1128 face="Adwaita Mono" heard="✓ hide" in=34 layout=78.96,33.41 owner=60 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                                65 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,579,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=247.1,31.2 owner=60 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                66 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,579,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=246.27,31.2 owner=60 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        67 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          68 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=34 role=none
                            69 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              70 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,645,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 role=none
                                71 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,660,387,48 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=240.99,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                                72 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,660,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=243.8,31.2 owner=67 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        73 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,81 focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          74 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=34 role=none
                            75 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              76 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,726,1000.5,79.5 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 role=none
                                77 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286.5,741,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=256.59,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                                78 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=709.5,741,387,48 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=251.29,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                79 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  80 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    81 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      82 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 face=Barlow heard="2 to hide" layout=78.96,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 to hide"
                    83 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      84 Button at=877.5,987,255,66 by=panel focus heard="Apply the marked changes" limit=0,0,1920,1200 paints=#707070FF,#D4D4D4FF role=button
                        85 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 role=none
                          86 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 role=image
                          87 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 owner=84 paints=#D4D4D4FF role=text size=26 text="Apply…"
              88 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                89 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  90 ScrollViewer at=1192.5,276,595.5,741 by=panel fades=b limit=0,0,1920,1200 overflow=46.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1192.5,276,595.5,741
                    91 StackPanel at=1192.5,276,595.5,787.5 by=scroll in=90 limit=1192.5,276,595.5,741 name=LibraryDetail paints=#D4D4D4FF role=none seen=1192.5,276,595.5,741
                      92 TextBlock at=1192.5,276,595.5,174 box=397,116 by=scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=90 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      93 TextBlock at=1192.5,459,595.5,120 box=397,80 by=scroll caps=15.3989 face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=90 layout=353.52,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      94 TextBlock at=1192.5,588,595.5,159 box=397,106 by=scroll caps=15.3989 face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=90 layout=395.36,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      95 TextBlock at=1192.5,756,595.5,40.5 box=397,27 by=scroll caps=15.3989 face=Barlow heard="System: psx" in=90 layout=113.15,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      96 TextBlock at=1192.5,805.5,595.5,159 box=397,106 by=scroll caps=15.3989 face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=90 layout=379.59,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      97 TextBlock at=1192.5,973.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 face=Barlow heard="Header checksum: not checked yet." in=90 layout=332.13,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
            98 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              99 HintBar at=96,1074,1728,45 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          100 Panel at=0,0,1920,1053 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            101 Border at=420,198,1080,657 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              102 DockPanel at=469.5,247.5,981,558 by=panel limit=0,0,1920,1200 role=none
                103 TextBlock at=469.5,247.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                104 StackPanel at=469.5,637.5,981,168 by=panel limit=0,0,1920,1200 role=none
                  105 Button at=469.5,637.5,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    106 TextBlock at=511.5,654,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,637.5,981,81 owner=105 paints=#EEEEF0FF role=text size=26 text=No
                  107 Button at=469.5,724.5,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    108 TextBlock at=511.5,741,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,724.5,981,81 owner=107 paints=#D4D4D4FF role=text size=26 text=Yes
                109 ScrollViewer at=469.5,324,981,295.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,324,981,295.5
                  110 StackPanel at=469.5,324,981,295.5 by=fixed-scroll in=109 limit=469.5,324,981,295.5 role=none
                    111 TextBlock at=469.5,324,981,159 box=654,106 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=109 layout=653.33,105.6 limit=469.5,324,981,295.5 paints=#D4D4D4FF role=text size=22 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                    112 TextBlock at=469.5,495,981,124.5 box=654,83 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Settings ▸ Library ▸ Hidden games." in=109 layout=620.47,82.94 limit=469.5,324,981,295.5 paints=#D4D4D4FF role=text size=22 text="Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Settings ▸ Library ▸ Hidden games."
focus 105
edge 105 up same
edge 105 down 107
edge 105 left same
edge 105 right same
edge 107 up 105
edge 107 down same
edge 107 left same
edge 107 right same
press pageup focus="Button  (No)" top=LibraryWindow
press pagedown focus="Button  (No)" top=LibraryWindow
press first focus="Button  (No)" top=LibraryWindow
press last focus="Button  (No)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there)" top=LibraryWindow
