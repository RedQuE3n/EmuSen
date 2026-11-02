# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case LibraryRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,764 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=756,48,460,24 box=460,24 by=panel caps=13.999 drawn face=Barlow heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=459.1,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,680,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,680,39 box=680,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
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
                31 Border at=64,147,691,512 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,147,691,512 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    33 LunaTable at=64,147,691,512 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,158,691,501 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,501
                35 Border at=64,659,691,87 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  36 Grid at=68,663,683,79 by=panel limit=0,0,1280,800 role=none
                    37 Grid at=68,663,683,27 by=panel limit=0,0,1280,800 role=none
                      38 ProgressBar at=68,675,120,4 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                      39 TextBlock at=196,663,555,27 box=555,27 by=panel caps=15.3989 drawn face=Barlow heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=406.3,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    40 StackPanel at=674,698,77,44 by=panel limit=0,0,1280,800 role=none
                      41 Button at=674,698,77,44 by=panel drawn focus heard=STOP limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                        42 TextBlock at=691,704,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=674,698,77,44 owner=41 paints=#D4D4D4FF role=text size=26 text=STOP
              43 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                44 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  45 TextBlock at=803,179,381,535 box=381,535 by=panel caps=15.3989 drawn face=Barlow heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=380.09,184.8 limit=0,0,1280,800 name=LibraryOverview paints=#D4D4D4FF role=text size=22 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            46 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              47 HintBar at=96,757,1088,29 by=panel heard="Back, Finding, List, Apply…, More" limit=0,0,1280,800 role=group
focus 17
edge 12 up same
edge 12 down 41
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 41
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 41
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 41
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 41
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 41
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 41
edge 27 left 25
edge 27 right 12
edge 41 up 25
edge 41 down same
edge 41 left 17
edge 41 right 27
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back top=none
