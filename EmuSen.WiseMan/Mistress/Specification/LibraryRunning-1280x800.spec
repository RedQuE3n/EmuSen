# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,145,812.7,484
title "Library Manager"
case LibraryRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,245,813,364 by=panel limit=234,245,812.35,364 paints=#D4D4D4FF role=none seen=234,245,812.35,364
    1 LibraryPane at=258,257,765,352 by=panel heard="Library Manager" limit=234,245,812.35,364 role=pane
      2 Grid at=258,257,765,352 by=panel limit=234,245,812.35,364 role=none
        3 DockPanel at=274,269,733,46 by=panel limit=234,245,812.35,364 role=none
          4 Border at=827,269,180,46 by=panel limit=234,245,812.35,364 name=AdaptiveSearch role=none
            5 TextBox at=827,269,180,46 by=panel drawn focus heard=Search limit=234,245,812.35,364 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,276,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,276,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=6 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
          8 Border at=274,269,553,46 by=panel limit=234,245,812.35,364 name=AdaptiveNavigation role=none
            9 WholeRow at=274,269,553,46 by=panel limit=234,245,812.35,364 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,270,65,44 by=panel drawn focus heard=All limit=234,245,812.35,364 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,276,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,270,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,270,131,44 by=panel drawn focus heard="Same Dump" limit=234,245,812.35,364 op paints=#FFFFFFFF role=button text="Same Dump"
                13 TextBlock at=364,276,97,32 box=97,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=96.28,31.2 limit=347,270,131,44 owner=12 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
              14 ToggleButton at=486,270,129,44 by=panel drawn focus heard="Same Game" limit=234,245,812.35,364 op paints=#FFFFFFFF role=button text="Same Game"
                15 TextBlock at=503,276,95,32 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=94.48,31.2 limit=486,270,129,44 owner=14 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
        16 Grid at=274,327,733,43 by=panel limit=234,245,812.35,364 role=none
          17 Border at=274,327,733,43 by=panel limit=234,245,812.35,364 name=AdaptiveContent role=none
            18 Grid at=274,327,733,43 by=panel limit=234,245,812.35,364 role=none
              19 LunaTable at=274,327,733,43 by=panel heard=Rows limit=234,245,812.35,364 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286,333,38,29 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                21 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=318,331,6,32 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                22 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,331,336,32 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  23 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,331,48,32 role=none
                    24 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,331,48,32 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=22 paints=#D4D4D4FF role=text size=26 text=Game
                25 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=654,331,6,32 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                26 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,333,335,29 box=335,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=56.71,28.8 paints=#D4D4D4FF role=text size=24 text=Finding
        27 Border at=274,378,733,96 by=panel limit=234,245,812.35,364 name=AdaptiveActions role=none
          28 WrapPanel at=274,378,733,96 by=panel limit=234,245,812.35,364 role=none
            29 ActionButton at=274,378,73,44 by=panel drawn focus heard=HIDE limit=234,245,812.35,364 op paints=#707070FF,#D4D4D4FF role=button text=Hide
              30 TextBlock at=291,384,39,32 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=274,378,73,44 owner=29 paints=#D4D4D4FF role=text size=26 text=HIDE
            31 ActionButton at=359,378,230,44 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=234,245,812.35,364 op paints=#707070FF,#D4D4D4FF role=button text="Show in File Manager"
              32 TextBlock at=376,384,196,32 box=196,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=195.86,31.2 limit=359,378,230,44 owner=31 paints=#D4D4D4FF role=text size=26 text="SHOW IN FILE MANAGER"
            33 ActionButton at=601,378,132,44 by=panel drawn focus heard="SAVE LIST…" limit=234,245,812.35,364 op paints=#707070FF,#D4D4D4FF role=button text="Save List…"
              34 TextBlock at=618,384,98,32 box=98,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=97.11,31.2 limit=601,378,132,44 owner=33 paints=#D4D4D4FF role=text size=26 text="SAVE LIST…"
            35 ActionButton at=745,378,144,44 by=panel drawn heard="CHECK AGAIN" limit=234,245,812.35,364 off paints=#707070FF,#D4D4D4FF role=button text="Check Again"
              36 TextBlock at=762,384,110,32 box=110,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=109.38,31.2 limit=745,378,144,44 off owner=35 paints=#707070FF role=text size=26 text="CHECK AGAIN"
            37 ActionButton at=901,378,77,44 by=panel drawn focus heard=STOP limit=234,245,812.35,364 op paints=#707070FF,#D4D4D4FF role=button text=Stop
              38 TextBlock at=918,384,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=901,378,77,44 owner=37 paints=#D4D4D4FF role=text size=26 text=STOP
            39 ActionButton at=274,430,80,44 by=panel drawn focus heard=MARK limit=234,245,812.35,364 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              40 TextBlock at=291,436,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=274,430,80,44 owner=39 paints=#D4D4D4FF role=text size=26 text=MARK
        41 Border at=258,482,765,89 by=panel limit=234,245,812.35,364 name=AdaptiveStatus role=none
          42 Grid at=262,486,757,81 by=panel limit=234,245,812.35,364 role=none
            43 Grid at=262,486,757,29 by=panel limit=234,245,812.35,364 role=none
              44 ProgressBar at=262,498,120,4 by=panel drawn heard=Progress limit=234,245,812.35,364 paints=#333337FF,#4C9AE8FF role=progressbar
              45 TextBlock at=390,486,629,29 box=629,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=343.82,28.8 limit=234,245,812.35,364 paints=#D4D4D4FF role=text size=24 text="Checking each file: 2 of 4, Amber Saga (USA)"
            46 StackPanel at=917,523,102,44 by=panel limit=234,245,812.35,364 role=none
              47 Button at=917,523,102,44 by=panel drawn heard="Apply the marked changes" limit=234,245,812.35,364 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                48 TextBlock at=934,529,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=917,523,102,44 off owner=47 paints=#707070FF role=text size=26 text="APPLY…"
        49 HintBar at=274,579,733,22 by=panel heard="Open, Back, Section, Apply…, Check Again, Mark" limit=234,245,812.35,364 role=group
focus 5
edge 5 up same
edge 5 down 37
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 21
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 22
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 22
edge 14 left 12
edge 14 right 5
edge 21 up 10
edge 21 down 29
edge 21 left same
edge 21 right 22
edge 22 up 12
edge 22 down 29
edge 22 left 21
edge 22 right 5
edge 29 up 21
edge 29 down 39
edge 29 left same
edge 29 right 31
edge 31 up 22
edge 31 down 39
edge 31 left 29
edge 31 right 33
edge 33 up 22
edge 33 down 39
edge 33 left 31
edge 33 right 37
edge 37 up 5
edge 37 down 39
edge 37 left 33
edge 37 right same
edge 39 up 29
edge 39 down same
edge 39 left same
edge 39 right 31
press pageup focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press first focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press last focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
