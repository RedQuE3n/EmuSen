# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title "Library Manager"
case LibraryPlan
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
              19 Border at=282,206,717,227 by=panel limit=234,124,812.35,548 paints=#1B1B1EFF,#707070FF role=none
                20 DockPanel at=299,223,683,193 by=panel limit=234,124,812.35,548 role=none
                  21 TextBlock at=299,223,683,29 box=683,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  22 StackPanel at=790,372,192,44 by=panel limit=234,124,812.35,548 role=none
                    23 Button at=790,372,99,44 by=panel drawn focus heard=Cancel limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      24 TextBlock at=807,378,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=790,372,99,44 owner=23 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    25 Button at=897,372,85,44 by=panel drawn focus heard=Apply limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      26 TextBlock at=914,378,51,32 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=897,372,85,44 owner=25 paints=#D4D4D4FF role=text size=26 text=APPLY
                  27 ScrollViewer at=299,264,683,96 by=panel fades=b limit=234,124,812.35,548 overflow=135 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=299,264,683,96
                    28 TextBlock at=299,264,683,231 box=683,231 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=27 layout=677.52,230.4 limit=299,264,683,96 paints=#D4D4D4FF role=text seen=299,264,683,96 size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
        29 Border at=274,441,733,96 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          30 WrapPanel at=274,441,733,96 by=panel limit=234,124,812.35,548 role=none
            31 ActionButton at=274,441,73,44 by=panel drawn focus heard=HIDE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Hide
              32 TextBlock at=291,447,39,32 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=274,441,73,44 owner=31 paints=#D4D4D4FF role=text size=26 text=HIDE
            33 ActionButton at=359,441,230,44 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Show in File Manager"
              34 TextBlock at=376,447,196,32 box=196,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=195.86,31.2 limit=359,441,230,44 owner=33 paints=#D4D4D4FF role=text size=26 text="SHOW IN FILE MANAGER"
            35 ActionButton at=601,441,132,44 by=panel drawn focus heard="SAVE LIST…" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Save List…"
              36 TextBlock at=618,447,98,32 box=98,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=97.11,31.2 limit=601,441,132,44 owner=35 paints=#D4D4D4FF role=text size=26 text="SAVE LIST…"
            37 ActionButton at=745,441,144,44 by=panel drawn focus heard="CHECK AGAIN" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Check Again"
              38 TextBlock at=762,447,110,32 box=110,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=109.38,31.2 limit=745,441,144,44 owner=37 paints=#D4D4D4FF role=text size=26 text="CHECK AGAIN"
            39 ActionButton at=901,441,77,44 by=panel drawn heard=STOP limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              40 TextBlock at=918,447,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=901,441,77,44 off owner=39 paints=#707070FF role=text size=26 text=STOP
            41 ActionButton at=274,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              42 TextBlock at=291,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=274,493,80,44 owner=41 paints=#D4D4D4FF role=text size=26 text=MARK
        43 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          44 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            45 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              46 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="2 to hide"
            47 StackPanel at=917,586,102,44 by=panel limit=234,124,812.35,548 role=none
              48 Button at=917,586,102,44 by=panel drawn focus heard="Apply the marked changes" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                49 TextBlock at=934,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=917,586,102,44 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
        50 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Check Again, Mark" limit=234,124,812.35,548 role=group
focus 23
edge 5 up same
edge 5 down 23
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 31
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 33
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 33
edge 14 left 12
edge 14 right 5
edge 23 up 5
edge 23 down 37
edge 23 left 35
edge 23 right 25
edge 25 up 5
edge 25 down 48
edge 25 left 23
edge 25 right same
edge 31 up 10
edge 31 down 41
edge 31 left same
edge 31 right 33
edge 33 up 12
edge 33 down 41
edge 33 left 31
edge 33 right 35
edge 35 up 23
edge 35 down 41
edge 35 left 33
edge 35 right 37
edge 37 up 23
edge 37 down 41
edge 37 left 35
edge 37 right 25
edge 41 up 31
edge 41 down 48
edge 41 left same
edge 41 right 33
edge 48 up 25
edge 48 down same
edge 48 left 37
edge 48 right same
press pageup focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press pagedown focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press first focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press last focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
