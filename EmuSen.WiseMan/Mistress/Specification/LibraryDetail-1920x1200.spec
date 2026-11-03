# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case LibraryDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 LibraryPane at=72,42,1776,1098 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1143,78,681,36 box=454,24 by=panel caps=13.999 drawn face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=96,66,1029,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1029,58.5 box=686,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=219,150,154.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=219,162,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=219,150,154.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=421.5,150,151.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=421.5,162,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,150,151.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=621,150,276,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=621,162,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=621,150,276,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=945,150,205.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=945,162,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=945,150,205.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=1198.5,150,100.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=1198.5,162,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1198.5,150,100.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    33 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          36 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=34 role=none
                            37 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              38 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 role=none
                                39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,255,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                40 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,255,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        41 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          42 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=34 role=none
                            43 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              44 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 role=none
                                45 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,336,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=290.42,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                                46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,336,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        47 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=34 role=none
                            49 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              50 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                51 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,417,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=303.6,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                                52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,417,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=308.41,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        53 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          54 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=34 role=none
                            55 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              56 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,483,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                57 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,498,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,498,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=308.41,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        59 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 op paints=#333337FF,#EEEEF0FF role=listitem
                          60 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=34 role=none
                            61 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=34 paints=#26262BFF,#4C9AE8FF role=none
                              62 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,564,1000.5,79.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 role=none
                                63 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,579,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=302.09,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,579,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=306.46,31.2 owner=59 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        65 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          66 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=34 role=none
                            67 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              68 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,645,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 role=none
                                69 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,660,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=240.99,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                                70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,660,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=303.06,31.2 owner=65 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        71 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,81 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 op paints=#333337FF,#D4D4D4FF role=listitem
                          72 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=34 role=none
                            73 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=34 paints=#0E0E10FF role=none
                              74 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,726,1000.5,79.5 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 role=none
                                75 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,741,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=297.52,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                                76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,741,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=300.9,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                77 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  78 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    79 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      80 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    81 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      82 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        83 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          84 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          85 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=82 paints=#707070FF role=text size=26 text="Apply…"
              86 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                87 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  88 ScrollViewer at=1192.5,276,595.5,741 by=panel fades=b limit=0,0,1920,1200 overflow=46.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1192.5,276,595.5,741
                    89 StackPanel at=1192.5,276,595.5,787.5 by=scroll in=88 limit=1192.5,276,595.5,741 name=LibraryDetail paints=#D4D4D4FF role=none seen=1192.5,276,595.5,741
                      90 TextBlock at=1192.5,276,595.5,174 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=88 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      91 TextBlock at=1192.5,459,595.5,120 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=88 layout=353.52,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      92 TextBlock at=1192.5,588,595.5,159 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=88 layout=395.36,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      93 TextBlock at=1192.5,756,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=88 layout=113.15,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      94 TextBlock at=1192.5,805.5,595.5,159 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=88 layout=379.59,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      95 TextBlock at=1192.5,973.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Header checksum: not checked yet." in=88 layout=332.13,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
            96 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              97 HintBar at=96,1074,1728,45 by=panel heard="Hide…, Back, Finding, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 59
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
edge 27 down 35
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
edge 47 right 12
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
edge 71 down same
edge 71 left same
edge 71 right 12
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file)" top=LibraryWindow
