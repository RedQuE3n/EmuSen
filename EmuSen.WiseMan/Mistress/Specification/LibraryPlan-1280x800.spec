# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case LibraryPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,732 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=762,52,454,24 box=454,24 by=panel caps=13.999 face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=64,44,686,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,686,39 box=686,39 by=panel caps=22.3984 face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  17 SectionTab at=80,100,34,44 by=panel focus heard=All limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=146,100,103,44 by=panel focus heard="Same Dump" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Same Dump"
                    20 TextBlock at=146,108,103,24 box=103,24 by=clip caps=13.999 face=Barlow heard="SAME DUMP" layout=102.88,24 limit=146,100,103,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 SectionTab at=281,100,101,44 by=panel focus heard="Same Game" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Same Game"
                    22 TextBlock at=281,108,101,24 box=101,24 by=clip caps=13.999 face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,100,101,44 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 SectionTab at=414,100,184,44 by=panel focus heard="Broken or Suspect" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    24 TextBlock at=414,108,184,24 box=184,24 by=clip caps=13.999 face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=414,100,184,44 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 SectionTab at=630,100,137,44 by=panel focus heard=Unrecognised limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=Unrecognised
                    26 TextBlock at=630,108,137,24 box=137,24 by=clip caps=13.999 face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=630,100,137,44 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 SectionTab at=799,100,67,44 by=panel focus heard=Hidden limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=Hidden
                    28 TextBlock at=799,108,67,24 box=67,24 by=clip caps=13.999 face=Barlow heard=HIDDEN layout=66.66,24 limit=799,100,67,44 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              30 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                31 Border at=64,160,691,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,160,691,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    33 LunaTable at=64,160,691,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,432
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,54 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          36 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,53 in=34 role=none
                            37 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,53 in=34 paints=#0E0E10FF role=none
                              38 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,160,667,53 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 role=none
                                39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,170,258,32 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                40 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,170,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=249.81,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        41 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,54 focus heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          42 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,53 in=34 role=none
                            43 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,214,691,53 in=34 paints=#0E0E10FF role=none
                              44 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,214,667,53 heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 role=none
                                45 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,224,79,34 box=79,34 caps=19.1128 face="Adwaita Mono" heard="✓ hide" in=34 layout=78.96,33.41 owner=41 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                                46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,224,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.79,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                                47 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,224,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=249.81,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        48 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,54 focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          49 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,53 in=34 role=none
                            50 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,268,691,53 in=34 paints=#0E0E10FF role=none
                              51 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,268,667,53 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,278,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=256.65,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                                53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,278,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=257.04,31.2 owner=48 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        54 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,54 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,53 in=34 role=none
                            56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,322,691,53 in=34 paints=#0E0E10FF role=none
                              57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,322,667,53 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                                58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,332,258,32 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,332,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=257.04,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,54 focus heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 paints=#333337FF,#EEEEF0FF role=listitem
                          61 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,53 in=34 role=none
                            62 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,376,691,53 in=34 paints=#26262BFF,#4C9AE8FF role=none
                              63 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,376,667,53 heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 role=none
                                64 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,386,79,34 box=79,34 caps=19.1128 face="Adwaita Mono" heard="✓ hide" in=34 layout=78.96,33.41 owner=60 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                                65 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,386,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=247.1,31.2 owner=60 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                66 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,386,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=246.27,31.2 owner=60 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        67 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,54 focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          68 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,53 in=34 role=none
                            69 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,430,691,53 in=34 paints=#0E0E10FF role=none
                              70 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,430,667,53 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 role=none
                                71 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,440,258,32 box=258,32 caps=18.1987 face=Barlow heard=Game in=34 layout=240.99,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                                72 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,440,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=243.8,31.2 owner=67 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        73 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,54 focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 paints=#333337FF,#D4D4D4FF role=listitem
                          74 DockPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,53 in=34 role=none
                            75 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,484,691,53 in=34 paints=#0E0E10FF role=none
                              76 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,484,667,53 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 role=none
                                77 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,494,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=256.59,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                                78 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=473,494,258,32 box=258,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=251.29,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                79 Border at=64,592,691,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  80 Grid at=64,610,691,92 by=panel limit=0,0,1280,800 role=none
                    81 Grid at=64,610,691,31 by=panel limit=0,0,1280,800 role=none
                      82 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 face=Barlow heard="2 to hide" layout=78.96,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 to hide"
                    83 StackPanel at=585,658,170,44 by=panel limit=0,0,1280,800 role=none
                      84 Button at=585,658,170,44 by=panel focus heard="Apply the marked changes" limit=0,0,1280,800 paints=#707070FF,#D4D4D4FF role=button
                        85 StackPanel at=602,664,136,32 by=clip limit=585,658,170,44 role=none
                          86 PadGlyph at=602,669,22,22 by=clip heard=Menu limit=585,658,170,44 role=image
                          87 TextBlock at=634,664,83,32 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=585,658,170,44 owner=84 paints=#D4D4D4FF role=text size=26 text="Apply…"
              88 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                89 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  90 ScrollViewer at=795,184,397,494 by=panel fades=b limit=0,0,1280,800 overflow=31 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=795,184,397,494
                    91 StackPanel at=795,184,397,525 by=scroll in=90 limit=795,184,397,494 name=LibraryDetail paints=#D4D4D4FF role=none seen=795,184,397,494
                      92 TextBlock at=795,184,397,116 box=397,116 by=scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=90 layout=365.18,115.2 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      93 TextBlock at=795,306,397,80 box=397,80 by=scroll caps=15.3989 face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=90 layout=353.52,79.2 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      94 TextBlock at=795,392,397,106 box=397,106 by=scroll caps=15.3989 face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=90 layout=395.36,105.6 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      95 TextBlock at=795,504,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="System: psx" in=90 layout=113.15,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      96 TextBlock at=795,537,397,106 box=397,106 by=scroll caps=15.3989 face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=90 layout=379.59,105.6 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      97 TextBlock at=795,649,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="Header checksum: not checked yet." in=90 layout=332.13,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
            98 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              99 HintBar at=64,716,1152,30 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          100 Panel at=0,0,1280,702 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            101 Border at=280,132,720,438 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              102 DockPanel at=313,165,654,372 by=panel limit=0,0,1280,800 role=none
                103 TextBlock at=313,165,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                104 StackPanel at=313,425,654,112 by=panel limit=0,0,1280,800 role=none
                  105 Button at=313,425,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    106 TextBlock at=341,436,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,425,654,54 owner=105 paints=#EEEEF0FF role=text size=26 text=No
                  107 Button at=313,483,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    108 TextBlock at=341,494,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,483,654,54 owner=107 paints=#D4D4D4FF role=text size=26 text=Yes
                109 ScrollViewer at=313,216,654,197 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,216,654,197
                  110 StackPanel at=313,216,654,197 by=fixed-scroll in=109 limit=313,216,654,197 role=none
                    111 TextBlock at=313,216,654,106 box=654,106 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=109 layout=653.33,105.6 limit=313,216,654,197 paints=#D4D4D4FF role=text size=22 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                    112 TextBlock at=313,330,654,83 box=654,83 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Settings ▸ Library ▸ Hidden games." in=109 layout=620.47,82.94 limit=313,216,654,197 paints=#D4D4D4FF role=text size=22 text="Only games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Settings ▸ Library ▸ Hidden games."
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
