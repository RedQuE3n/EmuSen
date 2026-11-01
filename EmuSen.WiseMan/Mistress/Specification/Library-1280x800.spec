# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Library-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Library Manager"
case Library
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 LibraryPane at=258,136,765,536 by=panel heard="Library Manager" limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=6 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,65,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,149,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,149,131,44 by=panel drawn focus heard="Same Dump" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Same Dump"
                13 TextBlock at=364,155,97,32 box=97,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=96.28,31.2 limit=347,149,131,44 owner=12 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
              14 ToggleButton at=486,149,129,44 by=panel drawn focus heard="Same Game" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Same Game"
                15 TextBlock at=503,155,95,32 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=94.48,31.2 limit=486,149,129,44 owner=14 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
        16 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
          17 Border at=274,206,733,227 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            18 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
              19 LunaTable at=274,206,733,227 by=panel heard=Rows limit=234,124,812.35,548 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=286,212,38,29 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                21 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=318,210,6,32 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                22 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,210,336,32 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  23 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,210,48,32 role=none
                    24 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=324,210,48,32 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=22 paints=#D4D4D4FF role=text size=26 text=Game
                25 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=654,210,6,32 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                26 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,212,335,29 box=335,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=56.71,28.8 paints=#D4D4D4FF role=text size=24 text=Finding
                27 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,249,733,184 fades=tb name=PART_ScrollViewer overflow=110 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=274,249,733,184
                  28 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,223,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=27 op paints=#333337FF,#D4D4D4FF role=listitem seen=274,249,733,16
                    29 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,228,713,31 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file" in=27 role=none seen=284,249,713,10
                      30 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,228,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=27 layout=14.4,30.84 owner=28 paints=#D4D4D4FF role=text seen=284,249,38,10 size=24 text="☐"
                      31 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=322,229,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=27 layout=139.25,28.8 owner=28 paints=#D4D4D4FF role=edit seen=322,249,338,9 size=24 text="Amber Saga (USA)"
                      32 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,229,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=27 layout=203.02,28.8 owner=28 paints=#D4D4D4FF role=edit seen=660,249,337,9 size=24 text="Same dump as 1 other file"
                  33 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,265,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=27 op paints=#333337FF,#D4D4D4FF role=listitem
                    34 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,270,713,31 heard="Mark: not marked, Game: Amber Saga (USA) [copier], Finding: Same dump as 1 other file" in=27 role=none
                      35 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,270,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=27 layout=14.4,30.84 owner=33 paints=#D4D4D4FF role=text size=24 text="☐"
                      36 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=322,271,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=27 layout=205.63,28.8 owner=33 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA) [copier]"
                      37 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,271,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=27 layout=203.02,28.8 owner=33 paints=#D4D4D4FF role=edit size=24 text="Same dump as 1 other file"
                  38 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,307,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=27 op paints=#333337FF,#D4D4D4FF role=listitem
                    39 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,312,713,31 heard="Mark: not marked, Game: Amber Saga (Europe) (Rev 1), Finding: Same game as 1 other file (Amber Saga)" in=27 role=none
                      40 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,312,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=27 layout=14.4,30.84 owner=38 paints=#D4D4D4FF role=text size=24 text="☐"
                      41 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=322,313,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=27 layout=218.33,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (Europe) (Rev 1)"
                      42 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,313,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=27 layout=310.03,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text="Same game as 1 other file (Amber Saga)"
                  43 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,349,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=27 op paints=#333337FF,#D4D4D4FF role=listitem
                    44 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,354,713,31 heard="Mark: not marked, Game: Amber Saga (USA), Finding: Same game as 1 other file (Amber Saga)" in=27 role=none
                      45 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,354,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=27 layout=14.4,30.84 owner=43 paints=#D4D4D4FF role=text size=24 text="☐"
                      46 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=322,355,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=27 layout=139.25,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                      47 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,355,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding in=27 layout=310.03,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text="Same game as 1 other file (Amber Saga)"
                  48 ListBoxItem allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=274,391,733,42 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=27 op paints=#333337FF,#D4D4D4FF role=listitem
                    49 Grid allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,396,713,31 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Finding: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=27 role=none
                      50 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=284,396,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=27 layout=14.4,30.84 owner=48 paints=#EEEEF0FF role=text size=24 text="☐"
                      51 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=322,397,338,29 box=338,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=27 layout=337.22,28.8 owner=48 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      52 TableCell allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=660,397,337,29 box=337,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Finding in=27 layout=330.36,28.8 owner=48 paints=#EEEEF0FF role=edit size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
        53 Border at=274,441,733,96 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          54 WrapPanel at=274,441,733,96 by=panel limit=234,124,812.35,548 role=none
            55 ActionButton at=274,441,73,44 by=panel drawn focus heard=HIDE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Hide
              56 TextBlock at=291,447,39,32 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=274,441,73,44 owner=55 paints=#D4D4D4FF role=text size=26 text=HIDE
            57 ActionButton at=359,441,230,44 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Show in File Manager"
              58 TextBlock at=376,447,196,32 box=196,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=195.86,31.2 limit=359,441,230,44 owner=57 paints=#D4D4D4FF role=text size=26 text="SHOW IN FILE MANAGER"
            59 ActionButton at=601,441,132,44 by=panel drawn focus heard="SAVE LIST…" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Save List…"
              60 TextBlock at=618,447,98,32 box=98,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=97.11,31.2 limit=601,441,132,44 owner=59 paints=#D4D4D4FF role=text size=26 text="SAVE LIST…"
            61 ActionButton at=745,441,144,44 by=panel drawn focus heard="CHECK AGAIN" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Check Again"
              62 TextBlock at=762,447,110,32 box=110,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=109.38,31.2 limit=745,441,144,44 owner=61 paints=#D4D4D4FF role=text size=26 text="CHECK AGAIN"
            63 ActionButton at=901,441,77,44 by=panel drawn heard=STOP limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              64 TextBlock at=918,447,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=901,441,77,44 off owner=63 paints=#707070FF role=text size=26 text=STOP
            65 ActionButton at=274,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              66 TextBlock at=291,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=274,493,80,44 owner=65 paints=#D4D4D4FF role=text size=26 text=MARK
        67 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          68 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            69 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              70 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            71 StackPanel at=917,586,102,44 by=panel limit=234,124,812.35,548 role=none
              72 Button at=917,586,102,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                73 TextBlock at=934,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=917,586,102,44 off owner=72 paints=#707070FF role=text size=26 text="APPLY…"
        74 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Check Again, Mark" limit=234,124,812.35,548 role=group
focus 21
edge 5 up same
edge 5 down 28
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
edge 21 down 28
edge 21 left 28
edge 21 right 22
edge 22 up 12
edge 22 down 28
edge 22 left 21
edge 22 right 28
edge 28 up 21
edge 28 down 33
edge 28 left same
edge 28 right same
edge 33 up 28
edge 33 down 38
edge 33 left same
edge 33 right same
edge 38 up 33
edge 38 down 43
edge 38 left same
edge 38 right same
edge 43 up 38
edge 43 down 48
edge 43 left same
edge 43 right same
edge 48 up 43
edge 48 down "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty"
edge 48 left same
edge 48 right same
edge 55 up "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)"
edge 55 down 65
edge 55 left same
edge 55 right 57
edge 57 up "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)"
edge 57 down 65
edge 57 left 55
edge 57 right 59
edge 59 up "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)"
edge 59 down 65
edge 59 left 57
edge 59 right 61
edge 61 up "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)"
edge 61 down 65
edge 61 left 59
edge 61 right same
edge 65 up 55
edge 65 down same
edge 65 left same
edge 65 right 57
edge "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" up 48
edge "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" down "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)"
edge "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" left same
edge "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty" right same
edge "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" up "row:ListBox/5/Mark: not marked, Game: Cobalt Harbor (Empty), Finding: Size: Cobalt Harbor (Empty).sfc is empty"
edge "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" down 55
edge "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" left same
edge "row:ListBox/6/Mark: not marked, Game: Granite Choir (Translated En), Finding: Not recognised by ScreenScraper (a hint, not a fault)" right same
press pageup focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
press pagedown focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
press first focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
press last focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
press back top=none
