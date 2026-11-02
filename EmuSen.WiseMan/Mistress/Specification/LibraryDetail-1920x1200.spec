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
      2 LibraryPane at=72,42,1776,1146 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1279.5,72,544.5,36 box=363,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=362.22,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=96,60,1165.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1165.5,58.5 box=777,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=13 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,138,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,138,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,138,202.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=231,150,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=207,138,202.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=421.5,138,199.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=445.5,150,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,138,199.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=633,138,324,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=657,150,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=633,138,324,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=969,138,253.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=993,150,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=969,138,253.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=1234.5,138,148.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=1258.5,150,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1234.5,138,148.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,223.5,1728,814.5 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,223.5,1036.5,814.5 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 role=none
                    33 LunaTable at=96,223.5,1036.5,747 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,730.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,730.5
                        35 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,255,964.5,49.5 in=34 paints=#707070FF role=none
                            37 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,255,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=34 paints=#4C9AE8FF role=none
                              38 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,255,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=35 paints=#EEEEF0FF role=edit size=26 text="Amber Saga (USA)"
                              39 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,255,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=35 paints=#EEEEF0FF role=edit size=26 text="Same dump as 1 other file"
                        40 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 op paints=#D4D4D4FF role=listitem
                          41 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,336,964.5,49.5 in=34 paints=#707070FF role=none
                            42 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,336,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=34 paints=#4C9AE8FF role=none
                              43 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,336,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=290.42,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA) [copier]"
                              44 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,336,463.5,48 box=309,32 caps=18.1987 drawn face=Barlow heard=Finding in=34 layout=277.68,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Same dump as 1 other file"
                        45 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          46 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,417,964.5,49.5 in=34 paints=#707070FF role=none
                            47 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,417,964.5,48 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#4C9AE8FF role=none
                              48 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,417,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=303.6,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (Europe) (Rev 1)"
                              49 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,417,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=308.41,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        50 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,483,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 op paints=#D4D4D4FF role=listitem
                          51 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,498,964.5,49.5 in=34 paints=#707070FF role=none
                            52 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,498,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=34 paints=#4C9AE8FF role=none
                              53 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,498,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=197.96,31.2 owner=50 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              54 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,498,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=308.41,31.2 owner=50 paints=#D4D4D4FF role=edit size=26 text="Same game as 1 other file (Amber Saga)"
                        55 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,564,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          56 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,579,964.5,49.5 in=34 paints=#707070FF role=none
                            57 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,579,964.5,48 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=34 paints=#4C9AE8FF role=none
                              58 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,579,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=302.09,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              59 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,579,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=306.46,31.2 owner=55 paints=#DAA520FF role=edit size=26 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                        60 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,645,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 op paints=#D4D4D4FF role=listitem
                          61 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,660,964.5,49.5 in=34 paints=#707070FF role=none
                            62 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,660,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" in=34 paints=#4C9AE8FF role=none
                              63 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,660,465,48 box=310,32 caps=18.1987 drawn face=Barlow heard=Game in=34 layout=240.99,31.2 owner=60 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Empty)"
                              64 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,660,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=303.06,31.2 owner=60 paints=#DAA520FF role=edit size=26 text="Size: Cobalt Harbor (Empty).sfc is empty"
                        65 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,726,1036.5,81 drawn focus heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 op paints=#D4D4D4FF role=listitem
                          66 Border allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,741,964.5,49.5 in=34 paints=#707070FF role=none
                            67 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=132,741,964.5,48 heard="Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" in=34 paints=#4C9AE8FF role=none
                              68 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=168,741,465,48 box=310,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=34 layout=297.52,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Granite Choir (Translated En)"
                              69 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=633,741,463.5,48 box=309,32 caps=18.1987 drawn ellipsis face=Barlow heard=Finding in=34 layout=300.9,31.2 owner=65 paints=#D4D4D4FF role=edit size=26 text="Not recognised by ScreenScraper (a hint, not a fault)"
                70 Border at=96,970.5,1036.5,67.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  71 Grid at=102,976.5,1024.5,55.5 by=panel limit=0,0,1920,1200 role=none
                    72 Grid at=102,976.5,1024.5,43.5 by=panel limit=0,0,1920,1200 role=none
                      73 TextBlock at=114,976.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
              74 Border at=1156.5,223.5,667.5,814.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                75 Border at=1192.5,259.5,595.5,742.5 by=panel limit=0,0,1920,1200 role=none
                  76 ScrollViewer at=1204.5,271.5,571.5,718.5 by=panel fades=b limit=0,0,1920,1200 overflow=31.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1204.5,271.5,571.5,718.5
                    77 StackPanel at=1204.5,271.5,571.5,750 by=scroll in=76 limit=1204.5,271.5,571.5,718.5 name=LibraryDetail role=none seen=1204.5,271.5,571.5,718.5
                      78 TextBlock at=1204.5,271.5,571.5,87 box=381,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=76 layout=361.46,57.6 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      79 TextBlock at=1204.5,367.5,571.5,130.5 box=381,87 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=76 layout=366.34,86.4 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      80 TextBlock at=1204.5,507,571.5,174 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=76 layout=377.16,115.2 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      81 TextBlock at=1204.5,690,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=76 layout=93.91,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      82 TextBlock at=1204.5,742.5,571.5,174 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=76 layout=362.59,115.2 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      83 TextBlock at=1204.5,925.5,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=76 layout=278.28,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      84 TextBlock at=1204.5,978,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=76 layout=158.35,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text seen=1204.5,978,571.5,12 size=24 text="Shown in the library."
            85 Border at=96,1050,1728,66 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              86 WrapPanel at=96,1050,1728,66 by=panel limit=0,0,1920,1200 role=none
                87 ActionButton at=96,1050,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  88 TextBlock at=121.5,1059,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1050,115.5,66 off owner=87 paints=#707070FF role=text size=26 text=STOP
            89 HintBar at=96,1140,1728,36 by=panel heard="Hide…, Back, Finding, List, Apply…, More" limit=0,0,1920,1200 role=group
focus 35
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
