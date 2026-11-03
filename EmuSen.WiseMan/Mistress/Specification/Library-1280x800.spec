# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Library-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Library
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,732 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 AdaptiveHeaderRow at=48,28,1184,60 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 StackPanel at=745,51,471,26 by=panel heard="4 files · ⚠ 2 suspect dumps · 1 duplicate · checked today" limit=0,0,1280,800 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=745,52,50,24 box=50,24 by=panel caps=13.999 drawn face=Barlow heard="4 files" layout=49.36,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="4 files"
                  9 TextBlock at=795,52,13,24 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=808,51,168,26 box=168,26 by=panel caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 2 suspect dumps" layout=167.02,25.7 limit=0,0,1280,800 paints=#DAA520FF role=text size=20 text="⚠ 2 suspect dumps"
                  11 TextBlock at=976,52,13,24 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=989,52,90,24 box=90,24 by=panel caps=13.999 drawn face=Barlow heard="1 duplicate" layout=89.06,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="1 duplicate"
                  13 TextBlock at=1079,52,13,24 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  14 TextBlock at=1092,52,124,24 box=124,24 by=panel caps=13.999 drawn face=Barlow heard="checked today" layout=123.14,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="checked today"
                15 StackPanel at=64,44,669,39 by=panel limit=0,0,1280,800 role=none
                  16 TextBlock at=64,44,669,39 box=669,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            17 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              18 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                19 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  20 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    21 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=20 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              22 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                23 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  24 SectionTab at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    25 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=24 paints=#EEEEF0FF role=text size=20 text=ALL
                  26 SectionTab at=146,100,103,44 by=panel drawn focus heard="Same Dump" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="Same Dump"
                    27 TextBlock at=146,108,103,24 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=146,100,103,44 owner=26 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  28 SectionTab at=281,100,101,44 by=panel drawn focus heard="Same Game" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="Same Game"
                    29 TextBlock at=281,108,101,24 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,100,101,44 owner=28 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  30 SectionTab at=414,100,184,44 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    31 TextBlock at=414,108,184,24 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=414,100,184,44 owner=30 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  32 SectionTab at=630,100,137,44 by=panel drawn focus heard=Unrecognised limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=Unrecognised
                    33 TextBlock at=630,108,137,24 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=630,100,137,44 owner=32 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  34 SectionTab at=799,100,67,44 by=panel drawn focus heard=Hidden limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=Hidden
                    35 TextBlock at=799,108,67,24 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=799,100,67,44 owner=34 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            36 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              37 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                38 Border at=64,160,691,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  39 Grid at=64,160,691,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    40 LunaTable at=64,160,691,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      41 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,432
                        42 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          43 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,53 in=41 role=none
                            44 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,53 in=41 paints=#0E0E10FF role=none
                              45 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,160,667,53 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=41 role=none
                                46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,170,310,32 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=197.96,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                47 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,170,309,32 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=41 layout=277.68,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        48 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          49 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,53 in=41 role=none
                            50 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,53 in=41 paints=#0E0E10FF role=none
                              51 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,214,667,53 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=41 role=none
                                52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,224,310,32 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=290.42,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                                53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,224,309,32 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=41 layout=277.68,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        54 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,53 in=41 role=none
                            56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,53 in=41 paints=#0E0E10FF role=none
                              57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,268,667,53 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                                58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,278,310,32 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=303.6,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                                59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,278,309,32 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=308.41,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          61 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,53 in=41 role=none
                            62 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,53 in=41 paints=#0E0E10FF role=none
                              63 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,322,667,53 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=41 role=none
                                64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,332,310,32 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=197.96,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                65 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,332,309,32 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=308.41,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        66 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,54 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 op paints=#333337FF,#EEEEF0FF role=listitem
                          67 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,53 in=41 role=none
                            68 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,53 in=41 paints=#26262BFF,#4C9AE8FF role=none
                              69 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,376,667,53 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=41 role=none
                                70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,386,310,32 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=41 layout=302.09,31.2 owner=66 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                71 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,386,309,32 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=306.46,31.2 owner=66 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        72 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          73 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,53 in=41 role=none
                            74 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,53 in=41 paints=#0E0E10FF role=none
                              75 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,430,667,53 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=41 role=none
                                76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,440,310,32 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=41 layout=240.99,31.2 owner=72 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                                77 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,440,309,32 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=303.06,31.2 owner=72 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        78 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,54 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 op paints=#333337FF,#D4D4D4FF role=listitem
                          79 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,53 in=41 role=none
                            80 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,53 in=41 paints=#0E0E10FF role=none
                              81 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,484,667,53 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=41 role=none
                                82 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,494,310,32 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=41 layout=297.52,31.2 owner=78 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                                83 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,494,309,32 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=41 layout=300.9,31.2 owner=78 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                84 Border at=64,592,691,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  85 Grid at=64,610,691,92 by=panel limit=0,0,1280,800 role=none
                    86 Grid at=64,610,691,31 by=panel limit=0,0,1280,800 role=none
                      87 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    88 StackPanel at=585,658,170,44 by=panel limit=0,0,1280,800 role=none
                      89 Button at=585,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                        90 StackPanel at=602,664,136,32 by=clip limit=585,658,170,44 off role=none
                          91 PadGlyph at=602,669,22,22 by=clip heard=Menu limit=585,658,170,44 off role=image
                          92 TextBlock at=634,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=585,658,170,44 off owner=89 paints=#707070FF role=text size=26 text="Apply…"
              93 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                94 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  95 ScrollViewer at=795,184,397,494 by=panel fades=b limit=0,0,1280,800 overflow=31 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=795,184,397,494
                    96 StackPanel at=795,184,397,525 by=scroll in=95 limit=795,184,397,494 name=LibraryDetail paints=#D4D4D4FF role=none seen=795,184,397,494
                      97 TextBlock at=795,184,397,116 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=95 layout=365.18,115.2 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      98 TextBlock at=795,306,397,80 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=95 layout=353.52,79.2 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      99 TextBlock at=795,392,397,106 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=95 layout=395.36,105.6 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      100 TextBlock at=795,504,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=95 layout=113.15,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      101 TextBlock at=795,537,397,106 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=95 layout=379.59,105.6 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      102 TextBlock at=795,649,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Header checksum: not checked yet." in=95 layout=332.13,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
            103 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              104 HintBar at=64,716,1152,30 by=panel heard="Hide…, Back, Finding, More, List, Apply…" limit=0,0,1280,800 role=group
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
