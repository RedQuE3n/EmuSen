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
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=1117.5,76.5,706.5,39 by=panel heard="4 files · ⚠ 2 suspect dumps · 1 duplicate · checked today" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=1117.5,78,75,36 box=50,24 by=panel caps=13.999 drawn face=Barlow heard="4 files" layout=49.36,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="4 files"
                  9 TextBlock at=1192.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=1212,76.5,252,39 box=168,26 by=panel caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 2 suspect dumps" layout=167.02,25.7 limit=0,0,1920,1200 paints=#DAA520FF role=text size=20 text="⚠ 2 suspect dumps"
                  11 TextBlock at=1464,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1483.5,78,135,36 box=90,24 by=panel caps=13.999 drawn face=Barlow heard="1 duplicate" layout=89.06,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="1 duplicate"
                  13 TextBlock at=1618.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  14 TextBlock at=1638,78,186,36 box=124,24 by=panel caps=13.999 drawn face=Barlow heard="checked today" layout=123.14,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="checked today"
                15 StackPanel at=96,66,1003.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  16 TextBlock at=96,66,1003.5,58.5 box=669,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            17 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              18 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                19 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  20 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    21 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=20 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              22 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                23 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  24 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    25 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=24 paints=#EEEEF0FF role=text size=20 text=ALL
                  26 SectionTab at=219,150,154.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Dump"
                    27 TextBlock at=219,162,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=219,150,154.5,66 owner=26 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  28 SectionTab at=421.5,150,151.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Game"
                    29 TextBlock at=421.5,162,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,150,151.5,66 owner=28 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  30 SectionTab at=621,150,276,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    31 TextBlock at=621,162,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=621,150,276,66 owner=30 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  32 SectionTab at=945,150,205.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Unrecognised
                    33 TextBlock at=945,162,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=945,150,205.5,66 owner=32 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  34 SectionTab at=1198.5,150,100.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Hidden
                    35 TextBlock at=1198.5,162,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1198.5,150,100.5,66 owner=34 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            36 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              37 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                38 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  39 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    40 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      41 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        42 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          43 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=41 role=none
                            44 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              45 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 role=none
                                46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,255,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=197.96,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                47 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,255,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=41 layout=277.68,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        48 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          49 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=41 role=none
                            50 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              51 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 role=none
                                52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,336,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=290.42,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                                53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,336,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=41 layout=277.68,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        54 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=41 role=none
                            56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                                58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,417,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=303.6,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                                59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,417,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=308.41,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          61 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=41 role=none
                            62 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              63 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,483,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                                64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,498,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=197.96,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                65 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,498,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=308.41,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        66 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 op paints=#333337FF,#EEEEF0FF role=listitem
                          67 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=41 role=none
                            68 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,79.5 in=41 paints=#26262BFF,#4C9AE8FF role=none
                              69 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,564,1000.5,79.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 role=none
                                70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,579,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=41 layout=302.09,31.2 owner=66 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                71 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,579,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=306.46,31.2 owner=66 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        72 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          73 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=41 role=none
                            74 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              75 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,645,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 role=none
                                76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,660,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=240.99,31.2 owner=72 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                                77 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,660,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=303.06,31.2 owner=72 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        78 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,81 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          79 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=41 role=none
                            80 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,79.5 in=41 paints=#0E0E10FF role=none
                              81 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,726,1000.5,79.5 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 role=none
                                82 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,741,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=41 layout=297.52,31.2 owner=78 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                                83 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,741,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=300.9,31.2 owner=78 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                84 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  85 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    86 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      87 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    88 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      89 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        90 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          91 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          92 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=89 paints=#707070FF role=text size=26 text="Apply…"
              93 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                94 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  95 ScrollViewer at=1192.5,276,595.5,741 by=panel fades=b limit=0,0,1920,1200 overflow=46.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1192.5,276,595.5,741
                    96 StackPanel at=1192.5,276,595.5,787.5 by=scroll in=95 limit=1192.5,276,595.5,741 name=LibraryDetail paints=#D4D4D4FF role=none seen=1192.5,276,595.5,741
                      97 TextBlock at=1192.5,276,595.5,174 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=95 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      98 TextBlock at=1192.5,459,595.5,120 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=95 layout=353.52,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      99 TextBlock at=1192.5,588,595.5,159 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=95 layout=395.36,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      100 TextBlock at=1192.5,756,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=95 layout=113.15,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      101 TextBlock at=1192.5,805.5,595.5,159 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=95 layout=379.59,105.6 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      102 TextBlock at=1192.5,973.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Header checksum: not checked yet." in=95 layout=332.13,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
            103 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              104 HintBar at=96,1074,1728,45 by=panel heard="Hide…, Back, Finding, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 66
edge 19 up same
edge 19 down 42
edge 19 left 34
edge 19 right same
edge 24 up same
edge 24 down 42
edge 24 left same
edge 24 right 26
edge 26 up same
edge 26 down 42
edge 26 left 24
edge 26 right 28
edge 28 up same
edge 28 down 42
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 42
edge 30 left 28
edge 30 right 32
edge 32 up same
edge 32 down 42
edge 32 left 30
edge 32 right 34
edge 34 up same
edge 34 down 42
edge 34 left 32
edge 34 right 19
edge 42 up 24
edge 42 down 48
edge 42 left same
edge 42 right 34
edge 48 up 42
edge 48 down 54
edge 48 left same
edge 48 right 34
edge 54 up 48
edge 54 down 60
edge 54 left same
edge 54 right 19
edge 60 up 54
edge 60 down 66
edge 60 left same
edge 60 right 19
edge 66 up 60
edge 66 down 72
edge 66 left same
edge 66 right 19
edge 72 up 66
edge 72 down 78
edge 72 left same
edge 72 right 19
edge 78 up 72
edge 78 down same
edge 78 left same
edge 78 right 19
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file)" top=LibraryWindow
