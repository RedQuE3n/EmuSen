# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case LibraryPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 LibraryPane at=48,28,1184,738 by=panel heard="Library Manager" limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 Border at=48,28,1184,51 by=panel heard="LIBRARY MANAGER" limit=0,0,1280,774 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,774 role=none
                7 TextBlock at=853,48,363,24 box=363,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=362.22,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,777,39 by=panel limit=0,0,1280,774 role=none
                  9 TextBlock at=64,40,777,39 box=777,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,774 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
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
                    33 Border at=64,154,691,420 by=panel limit=0,0,1280,774 paints=#1B1B1EFF,#707070FF role=none
                      34 DockPanel at=81,171,657,386 by=panel limit=0,0,1280,774 role=none
                        35 TextBlock at=81,171,657,39 box=657,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        36 StackPanel at=81,465,657,92 by=panel limit=0,0,1280,774 role=none
                          37 Button at=81,465,657,44 by=panel drawn focus heard=No limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=No
                            38 TextBlock at=98,471,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=81,465,657,44 owner=37 paints=#EEEEF0FF role=text size=26 text=NO
                          39 Button at=81,513,657,44 by=panel drawn focus heard=Yes limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            40 TextBlock at=98,519,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=81,513,657,44 owner=39 paints=#D4D4D4FF role=text size=26 text=YES
                        41 ScrollViewer at=81,222,657,231 by=panel limit=0,0,1280,774 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=81,222,657,231
                          42 TextBlock at=81,222,657,231 box=657,231 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=41 layout=655.01,230.4 limit=81,222,657,231 paints=#D4D4D4FF role=text size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
                43 Border at=64,579,691,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
                  44 Grid at=68,583,683,81 by=panel limit=0,0,1280,774 role=none
                    45 Grid at=68,583,683,29 by=panel limit=0,0,1280,774 role=none
                      46 TextBlock at=76,583,675,29 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="2 to hide"
                    47 StackPanel at=649,620,102,44 by=panel limit=0,0,1280,774 role=none
                      48 Button at=649,620,102,44 by=panel drawn focus heard="Apply the marked changes" limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        49 TextBlock at=666,626,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=649,620,102,44 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              50 Border at=771,149,445,519 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
                51 Border at=771,149,445,519 by=panel limit=0,0,1280,774 role=none
                  52 ScrollViewer at=779,157,429,503 by=panel limit=0,0,1280,774 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=779,157,429,503
                    53 StackPanel at=779,157,429,503 by=fixed-scroll in=52 limit=779,157,429,503 name=LibraryDetail role=none
                      54 TextBlock at=779,157,429,58 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=52 layout=361.46,57.6 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      55 TextBlock at=779,221,429,58 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=52 layout=395.83,57.6 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      56 TextBlock at=779,285,429,87 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=52 layout=419.14,86.4 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      57 TextBlock at=779,378,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=52 layout=93.91,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      58 TextBlock at=779,413,429,87 box=429,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=52 layout=419.9,86.4 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      59 TextBlock at=779,506,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=52 layout=278.28,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      60 TextBlock at=779,541,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=52 layout=158.35,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Shown in the library."
            61 Border at=64,676,1152,44 by=panel limit=0,0,1280,774 name=AdaptiveActions role=none
              62 WrapPanel at=64,676,1152,44 by=panel limit=0,0,1280,774 role=none
                63 ActionButton at=64,676,77,44 by=panel drawn heard=STOP limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  64 TextBlock at=81,682,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=64,676,77,44 off owner=63 paints=#707070FF role=text size=26 text=STOP
            65 HintBar at=64,736,1152,22 by=panel heard="Hide…, Back, Section, List, Apply…, More" limit=0,0,1280,774 role=group
focus 37
edge 12 up same
edge 12 down 37
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 37
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 37
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 37
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 37
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 37
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 37
edge 27 left 25
edge 27 right 12
edge 37 up 17
edge 37 down 39
edge 37 left same
edge 37 right 12
edge 39 up 37
edge 39 down 48
edge 39 left same
edge 39 right 12
edge 48 up 39
edge 48 down same
edge 48 left 19
edge 48 right 27
press pageup focus="Button 'No' (No)" top=LibraryWindow
press pagedown focus="Button 'No' (No)" top=LibraryWindow
press first focus="Button 'No' (No)" top=LibraryWindow
press last focus="Button 'No' (No)" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" top=LibraryWindow
