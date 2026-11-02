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
      2 LibraryPane at=48,28,1184,764 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=762,48,454,24 box=454,24 by=panel caps=13.999 face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,686,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,686,39 box=686,39 by=panel caps=22.3984 face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,135,44 by=panel focus heard="Same Dump" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=154,99,103,24 box=103,24 by=clip caps=13.999 face=Barlow heard="SAME DUMP" layout=102.88,24 limit=138,91,135,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=281,91,133,44 by=panel focus heard="Same Game" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=297,99,101,24 box=101,24 by=clip caps=13.999 face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,91,133,44 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=422,91,216,44 by=panel focus heard="Broken or Suspect" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=438,99,184,24 box=184,24 by=clip caps=13.999 face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=422,91,216,44 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=646,91,169,44 by=panel focus heard=Unrecognised limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=662,99,137,24 box=137,24 by=clip caps=13.999 face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=646,91,169,44 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=823,91,99,44 by=panel focus heard=Hidden limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=839,99,67,24 box=67,24 by=clip caps=13.999 face=Barlow heard=HIDDEN layout=66.66,24 limit=823,91,99,44 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              30 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                31 Border at=64,147,691,556 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,147,691,556 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    33 LunaTable at=64,147,691,556 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,158,691,545 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,545
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,158,691,54 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,168,643,33 in=34 paints=#707070FF role=none
                            37 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,168,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 role=none
                              38 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,168,79,32 in=34 owner=35 paints=#4C9AE8FF role=text
                              39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,168,246,32 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              40 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,168,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=236.7,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        41 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,212,691,54 focus heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          42 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,222,643,34 in=34 paints=#707070FF role=none
                            43 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,222,643,33 heard="Mark: hide, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 role=none
                              44 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,222,79,33 box=79,33 caps=18.916 face="Adwaita Sans" heard="✓ hide" in=34 layout=78.07,32.27 owner=41 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                              45 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,222,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.79,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                              46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,222,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=236.7,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        47 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,266,691,54 focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          48 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,276,643,33 in=34 paints=#707070FF role=none
                            49 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,276,643,32 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                              50 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,276,79,32 in=34 owner=47 paints=#4C9AE8FF role=text
                              51 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,276,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=244.66,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                              52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,276,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=234.49,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        53 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,320,691,54 focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          54 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,330,643,33 in=34 paints=#707070FF role=none
                            55 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,330,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 role=none
                              56 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,330,79,32 in=34 owner=53 paints=#4C9AE8FF role=text
                              57 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,330,246,32 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=197.96,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,330,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=234.49,31.2 owner=53 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        59 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,374,691,54 focus heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          60 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,384,643,34 in=34 paints=#707070FF role=none
                            61 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,384,643,33 heard="Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 role=none
                              62 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,384,79,33 box=79,33 caps=18.916 face="Adwaita Sans" heard="✓ hide" in=34 layout=78.07,32.27 owner=59 paints=#4C9AE8FF role=text size=26 text="✓ hide"
                              63 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,384,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=233.14,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,384,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=233.01,31.2 owner=59 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        65 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,428,691,54 focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 op paints=#D4D4D4FF role=listitem
                          66 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,438,643,33 in=34 paints=#707070FF role=none
                            67 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,438,643,32 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 role=none
                              68 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,438,79,32 in=34 owner=65 paints=#4C9AE8FF role=text
                              69 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,438,246,32 box=246,32 caps=18.1987 face=Barlow heard=Game in=34 layout=240.99,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                              70 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,438,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=243.8,31.2 owner=65 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        71 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,482,691,54 focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 op paints=#D4D4D4FF role=listitem
                          72 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,492,643,33 in=34 paints=#707070FF role=none
                            73 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,492,643,32 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 role=none
                              74 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,492,79,32 in=34 owner=71 paints=#4C9AE8FF role=text
                              75 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=191,492,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Game in=34 layout=243.46,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                              76 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=461,492,246,32 box=246,32 caps=18.1987 ellipsis face=Barlow heard=Finding in=34 layout=238.26,31.2 owner=71 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                77 Border at=64,703,691,43 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  78 Grid at=68,707,683,35 by=panel limit=0,0,1280,800 role=none
                    79 Grid at=68,707,683,27 by=panel limit=0,0,1280,800 role=none
                      80 TextBlock at=76,707,675,27 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 to hide" layout=78.96,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 to hide"
              81 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                82 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  83 ScrollViewer at=803,179,381,535 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=803,179,381,535
                    84 StackPanel at=803,179,381,535 by=fixed-scroll in=83 limit=803,179,381,535 name=LibraryDetail role=none
                      85 TextBlock at=803,179,381,80 box=381,80 by=fixed-scroll caps=16.0864 face="Barlow Medium" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=83 layout=333.56,79.2 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      86 TextBlock at=803,265,381,80 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=83 layout=353.52,79.2 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      87 TextBlock at=803,351,381,132 box=381,132 by=fixed-scroll caps=15.3989 face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=83 layout=371.43,132 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      88 TextBlock at=803,489,381,27 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="System: psx" in=83 layout=113.15,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      89 TextBlock at=803,522,381,106 box=381,106 by=fixed-scroll caps=15.3989 face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=83 layout=379.59,105.6 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      90 TextBlock at=803,634,381,27 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="Header checksum: not checked yet." in=83 layout=332.13,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
                      91 TextBlock at=803,667,381,27 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="Shown in the library." in=83 layout=189.07,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Shown in the library."
            92 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              93 HintBar at=96,757,1088,29 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          94 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveCover role=none
            95 Border at=280,184,720,453 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              96 DockPanel at=313,217,654,387 by=panel limit=0,0,1280,800 role=none
                97 TextBlock at=313,217,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                98 StackPanel at=313,492,654,112 by=panel limit=0,0,1280,800 role=none
                  99 Button at=313,492,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    100 TextBlock at=341,503,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,492,654,54 owner=99 paints=#EEEEF0FF role=text size=26 text=No
                  101 Button at=313,550,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    102 TextBlock at=341,561,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,550,654,54 owner=101 paints=#D4D4D4FF role=text size=26 text=Yes
                103 ScrollViewer at=313,268,654,212 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,268,654,212
                  104 TextBlock at=313,268,654,212 box=654,212 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=103 layout=653.33,211.2 limit=313,268,654,212 paints=#D4D4D4FF role=text size=22 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
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
edge 99 right 12
edge 101 up 99
edge 101 down same
edge 101 left 19
edge 101 right 12
press pageup focus="Button  (No)" top=LibraryWindow
press pagedown focus="Button  (No)" top=LibraryWindow
press first focus="Button  (No)" top=LibraryWindow
press last focus="Button  (No)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: hide, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there)" top=LibraryWindow
