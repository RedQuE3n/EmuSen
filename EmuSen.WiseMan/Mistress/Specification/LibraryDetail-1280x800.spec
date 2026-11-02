# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case LibraryDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,764 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=762,48,454,24 box=454,24 by=panel caps=13.999 drawn face=Barlow heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=453.66,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,686,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,686,39 box=686,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,135,44 by=panel drawn focus heard="Same Dump" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=154,99,103,24 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=138,91,135,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=281,91,133,44 by=panel drawn focus heard="Same Game" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=297,99,101,24 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,91,133,44 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=422,91,216,44 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=438,99,184,24 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=422,91,216,44 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=646,91,169,44 by=panel drawn focus heard=Unrecognised limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=662,99,137,24 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=646,91,169,44 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=823,91,99,44 by=panel drawn focus heard=Hidden limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=839,99,67,24 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=823,91,99,44 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              30 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                31 Border at=64,147,691,556 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,147,691,556 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    33 LunaTable at=64,147,691,556 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,158,691,545 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,545
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,158,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,168,643,33 in=34 paints=#707070FF role=none
                            37 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,168,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 paints=#4C9AE8FF role=none
                              38 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,168,286,32 box=286,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,168,285,32 box=285,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        40 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,212,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          41 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,222,643,33 in=34 paints=#707070FF role=none
                            42 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,222,643,32 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 paints=#4C9AE8FF role=none
                              43 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,222,286,32 box=286,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=279.19,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                              44 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,222,285,32 box=285,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        45 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,266,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          46 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,276,643,33 in=34 paints=#707070FF role=none
                            47 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,276,643,32 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#4C9AE8FF role=none
                              48 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,276,286,32 box=286,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=279.24,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                              49 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,276,285,32 box=285,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=283.04,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        50 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,320,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          51 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,330,643,33 in=34 paints=#707070FF role=none
                            52 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,330,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#4C9AE8FF role=none
                              53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,330,286,32 box=286,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=50 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              54 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,330,285,32 box=285,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=283.04,31.2 owner=50 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        55 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,374,691,54 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,384,643,33 in=34 paints=#707070FF role=none
                            57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,384,643,32 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 paints=#4C9AE8FF role=none
                              58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,384,286,32 box=286,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=266.08,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,384,285,32 box=285,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=277.32,31.2 owner=55 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,428,691,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 op paints=#D4D4D4FF role=listitem
                          61 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,438,643,33 in=34 paints=#707070FF role=none
                            62 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,438,643,32 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 paints=#4C9AE8FF role=none
                              63 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,438,286,32 box=286,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=240.99,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                              64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,438,285,32 box=285,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=280.28,31.2 owner=60 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        65 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,482,691,54 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 op paints=#D4D4D4FF role=listitem
                          66 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,492,643,33 in=34 paints=#707070FF role=none
                            67 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=88,492,643,32 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 paints=#4C9AE8FF role=none
                              68 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=112,492,286,32 box=286,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=278.12,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                              69 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=422,492,285,32 box=285,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=273.83,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                70 Border at=64,703,691,43 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  71 Grid at=68,707,683,35 by=panel limit=0,0,1280,800 role=none
                    72 Grid at=68,707,683,27 by=panel limit=0,0,1280,800 role=none
                      73 TextBlock at=76,707,675,27 box=675,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
              74 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                75 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  76 ScrollViewer at=803,179,381,535 by=panel fades=b limit=0,0,1280,800 overflow=16 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=803,179,381,535
                    77 StackPanel at=803,179,381,551 by=scroll in=76 limit=803,179,381,535 name=LibraryDetail role=none seen=803,179,381,535
                      78 TextBlock at=803,179,381,116 box=381,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=76 layout=365.18,115.2 limit=803,179,381,535 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      79 TextBlock at=803,301,381,80 box=381,80 by=scroll caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=76 layout=353.52,79.2 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      80 TextBlock at=803,387,381,132 box=381,132 by=scroll caps=15.3989 drawn face=Barlow heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=76 layout=371.43,132 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      81 TextBlock at=803,525,381,27 box=381,27 by=scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=76 layout=113.15,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      82 TextBlock at=803,558,381,106 box=381,106 by=scroll caps=15.3989 drawn face=Barlow heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=76 layout=379.59,105.6 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      83 TextBlock at=803,670,381,27 box=381,27 by=scroll caps=15.3989 drawn face=Barlow heard="Header checksum: not checked yet." in=76 layout=332.13,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Header checksum: not checked yet."
                      84 TextBlock at=803,703,381,27 box=381,27 by=scroll caps=15.3989 drawn face=Barlow heard="Shown in the library." in=76 layout=189.07,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text seen=803,703,381,11 size=22 text="Shown in the library."
            85 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              86 HintBar at=96,757,1088,29 by=panel heard="Hide…, Back, Finding, List, Apply…, More" limit=0,0,1280,800 role=group
focus 55
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
edge 35 down 40
edge 35 left same
edge 35 right 27
edge 40 up 35
edge 40 down 45
edge 40 left same
edge 40 right 27
edge 45 up 40
edge 45 down 50
edge 45 left same
edge 45 right 27
edge 50 up 45
edge 50 down 55
edge 50 left same
edge 50 right 12
edge 55 up 50
edge 55 down 60
edge 55 left same
edge 55 right 12
edge 60 up 55
edge 60 down 65
edge 60 left same
edge 60 right 12
edge 65 up 60
edge 65 down same
edge 65 left same
edge 65 right 12
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file)" top=LibraryWindow
