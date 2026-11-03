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
      2 LibraryPane at=48,28,1184,732 by=panel heard="Library Manager" limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=756,52,460,24 box=460,24 by=panel caps=13.999 drawn face=Barlow heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=459.1,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=64,44,680,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,680,39 box=680,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=146,100,103,44 by=panel drawn focus heard="Same Dump" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=146,108,103,24 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=146,100,103,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=281,100,101,44 by=panel drawn focus heard="Same Game" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=281,108,101,24 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,100,101,44 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=414,100,184,44 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=414,108,184,24 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=414,100,184,44 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=630,100,137,44 by=panel drawn focus heard=Unrecognised limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=630,108,137,24 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=630,100,137,44 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=799,100,67,44 by=panel drawn focus heard=Hidden limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=799,108,67,24 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=799,100,67,44 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              30 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                31 Border at=64,160,691,424 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,160,691,424 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    33 LunaTable at=64,160,691,424 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=64,160,691,424 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,424
                35 Border at=64,584,691,118 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  36 Grid at=64,602,691,100 by=panel limit=0,0,1280,800 role=none
                    37 Grid at=64,602,691,39 by=panel limit=0,0,1280,800 role=none
                      38 ProgressBar at=64,602,691,8 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                      39 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=406.3,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    40 StackPanel at=496,658,259,44 by=panel limit=0,0,1280,800 role=none
                      41 Button at=496,658,77,44 by=panel drawn focus heard=STOP limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                        42 TextBlock at=513,664,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=496,658,77,44 owner=41 paints=#D4D4D4FF role=text size=26 text=STOP
                      43 Button at=585,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                        44 StackPanel at=602,664,136,32 by=clip limit=585,658,170,44 off role=none
                          45 PadGlyph at=602,669,22,22 by=clip heard=Menu limit=585,658,170,44 off role=image
                          46 TextBlock at=634,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=585,658,170,44 off owner=43 paints=#707070FF role=text size=26 text="Apply…"
              47 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                48 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  49 TextBlock at=795,184,397,494 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=380.09,184.8 limit=0,0,1280,800 name=LibraryOverview paints=#D4D4D4FF role=text size=22 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            50 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              51 HintBar at=64,716,1152,30 by=panel heard="Back, Finding, More, List, Apply…" limit=0,0,1280,800 role=group
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
edge 41 up 23
edge 41 down same
edge 41 left 21
edge 41 right 12
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back top=none
