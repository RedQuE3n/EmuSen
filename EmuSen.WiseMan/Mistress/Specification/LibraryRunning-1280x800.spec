# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case LibraryRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,738 by=panel heard="Library Manager" limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 Border at=48,28,1184,51 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,774 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,774 role=none
                7 TextBlock at=849,48,367,24 box=367,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=366.62,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,773,39 by=panel limit=0,0,1280,774 role=none
                  9 TextBlock at=64,40,773,39 box=773,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,774 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=13 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
              15 Border at=64,91,972,46 by=panel limit=0,0,1280,774 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,774 role=none
                  17 ToggleButton at=64,92,65,44 by=panel drawn focus heard=All limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=All
                    18 TextBlock at=81,98,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=64,92,65,44 owner=17 paints=#EEEEF0FF role=text size=26 text=ALL
                  19 ToggleButton at=137,92,131,44 by=panel drawn focus heard="Same Dump" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="Same Dump"
                    20 TextBlock at=154,98,97,32 box=97,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=96.28,31.2 limit=137,92,131,44 owner=19 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
                  21 ToggleButton at=276,92,129,44 by=panel drawn focus heard="Same Game" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="Same Game"
                    22 TextBlock at=293,98,95,32 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=94.48,31.2 limit=276,92,129,44 owner=21 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
                  23 ToggleButton at=413,92,208,44 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="Broken or Suspect"
                    24 TextBlock at=430,98,174,32 box=174,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="BROKEN OR SUSPECT" layout=173.78,31.2 limit=413,92,208,44 owner=23 paints=#FFFFFFFF role=text size=26 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=629,92,162,44 by=panel drawn focus heard=Unrecognised limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Unrecognised
                    26 TextBlock at=646,98,128,32 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UNRECOGNISED layout=127.35,31.2 limit=629,92,162,44 owner=25 paints=#FFFFFFFF role=text size=26 text=UNRECOGNISED
                  27 ToggleButton at=799,92,96,44 by=panel drawn focus heard=Hidden limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Hidden
                    28 TextBlock at=816,98,62,32 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDDEN layout=61.41,31.2 limit=799,92,96,44 owner=27 paints=#FFFFFFFF role=text size=26 text=HIDDEN
            29 Grid at=64,149,1152,519 by=panel limit=0,0,1280,774 role=none
              30 Grid at=64,149,691,519 by=panel limit=0,0,1280,774 role=none
                31 Border at=64,149,691,430 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
                  32 Grid at=64,149,691,430 by=panel limit=0,0,1280,774 role=none
                    33 LunaTable at=64,149,691,430 by=panel heard=Rows limit=0,0,1280,774 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=76,155,38,29 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                      35 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=108,153,6,32 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                      36 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=114,153,315,32 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                        37 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=114,153,48,32 role=none
                          38 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=114,153,48,32 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=36 paints=#D4D4D4FF role=text size=26 text=Game
                      39 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=423,153,6,32 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                      40 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=429,155,314,29 box=314,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=56.71,28.8 paints=#D4D4D4FF role=text size=24 text=Finding
                      41 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,192,691,387 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,192,691,387
                42 Border at=64,579,691,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
                  43 Grid at=68,583,683,81 by=panel limit=0,0,1280,774 role=none
                    44 Grid at=68,583,683,29 by=panel limit=0,0,1280,774 role=none
                      45 ProgressBar at=68,595,120,4 by=panel drawn heard=Progress limit=0,0,1280,774 paints=#333337FF,#4C9AE8FF role=progressbar
                      46 TextBlock at=196,583,555,29 box=555,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=343.82,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    47 StackPanel at=649,620,102,44 by=panel limit=0,0,1280,774 role=none
                      48 Button at=649,620,102,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        49 TextBlock at=666,626,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=649,620,102,44 off owner=48 paints=#707070FF role=text size=26 text="APPLY…"
              50 Border at=771,149,445,519 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
                51 Border at=771,149,445,519 by=panel limit=0,0,1280,774 role=none
                  52 TextBlock at=779,157,429,503 box=429,503 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=409.18,144 limit=0,0,1280,774 name=LibraryOverview paints=#D4D4D4FF role=text size=24 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            53 Border at=64,676,1152,44 by=panel limit=0,0,1280,774 name=AdaptiveActions role=none
              54 WrapPanel at=64,676,1152,44 by=panel limit=0,0,1280,774 role=none
                55 ActionButton at=64,676,77,44 by=panel drawn focus heard=STOP limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                  56 TextBlock at=81,682,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=64,676,77,44 owner=55 paints=#D4D4D4FF role=text size=26 text=STOP
            57 HintBar at=64,736,1152,22 by=panel heard="Back, Section, List, Apply…, More" limit=0,0,1280,774 role=group
focus 12
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
edge 25 down 36
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 36
edge 27 left 25
edge 27 right 12
edge 35 up 17
edge 35 down 55
edge 35 left same
edge 35 right 36
edge 36 up 19
edge 36 down 39
edge 36 left 35
edge 36 right 25
edge 39 up 23
edge 39 down 55
edge 39 left 35
edge 39 right 25
edge 55 up 35
edge 55 down same
edge 55 left same
edge 55 right 25
press pageup focus="TextBox Search files (Search)" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" top=LibraryWindow
