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
                7 TextBlock at=853,48,363,24 box=363,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 1 same dumps · 2 broken or suspect · 0 hidden" layout=362.22,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="29 files · 1 same dumps · 2 broken or suspect · 0 hidden"
                8 StackPanel at=64,40,777,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,777,39 box=777,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=13 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
              15 Border at=64,91,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,92,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,100,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,92,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,92,135,44 by=panel drawn focus heard="Same Dump" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=154,100,103,24 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=138,92,135,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=281,92,133,44 by=panel drawn focus heard="Same Game" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=297,100,101,24 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=281,92,133,44 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=422,92,216,44 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=438,100,184,24 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=422,92,216,44 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=646,92,169,44 by=panel drawn focus heard=Unrecognised limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=662,100,137,24 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=646,92,169,44 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=823,92,99,44 by=panel drawn focus heard=Hidden limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=839,100,67,24 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=823,92,99,44 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=64,149,1152,543 by=panel limit=0,0,1280,800 role=none
              30 Grid at=64,149,691,543 by=panel limit=0,0,1280,800 role=none
                31 Border at=64,149,691,498 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  32 Grid at=64,149,691,498 by=panel limit=0,0,1280,800 role=none
                    33 Border at=64,188,691,420 by=panel limit=0,0,1280,800 paints=#1B1B1EFF,#707070FF role=none
                      34 DockPanel at=81,205,657,386 by=panel limit=0,0,1280,800 role=none
                        35 TextBlock at=81,205,657,39 box=657,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        36 StackPanel at=81,499,657,92 by=panel limit=0,0,1280,800 role=none
                          37 Button at=81,499,657,44 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=No
                            38 TextBlock at=98,505,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=81,499,657,44 owner=37 paints=#EEEEF0FF role=text size=26 text=NO
                          39 Button at=81,547,657,44 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            40 TextBlock at=98,553,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=81,547,657,44 owner=39 paints=#D4D4D4FF role=text size=26 text=YES
                        41 ScrollViewer at=81,256,657,231 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=81,256,657,231
                          42 TextBlock at=81,256,657,231 box=657,231 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=41 layout=655.01,230.4 limit=81,256,657,231 paints=#D4D4D4FF role=text size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
                43 Border at=64,647,691,45 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  44 Grid at=68,651,683,37 by=panel limit=0,0,1280,800 role=none
                    45 Grid at=68,651,683,29 by=panel limit=0,0,1280,800 role=none
                      46 TextBlock at=76,651,675,29 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="2 to hide"
              47 Border at=771,149,445,543 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                48 Border at=795,173,397,495 by=panel limit=0,0,1280,800 role=none
                  49 ScrollViewer at=803,181,381,479 by=panel fades=b limit=0,0,1280,800 overflow=21 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=803,181,381,479
                    50 StackPanel at=803,181,381,500 by=scroll in=49 limit=803,181,381,479 name=LibraryDetail role=none seen=803,181,381,479
                      51 TextBlock at=803,181,381,58 box=381,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=49 layout=361.46,57.6 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      52 TextBlock at=803,245,381,87 box=381,87 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=49 layout=366.34,86.4 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      53 TextBlock at=803,338,381,116 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=49 layout=377.16,115.2 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      54 TextBlock at=803,460,381,29 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=49 layout=93.91,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      55 TextBlock at=803,495,381,116 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=49 layout=362.59,115.2 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      56 TextBlock at=803,617,381,29 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=49 layout=278.28,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      57 TextBlock at=803,652,381,29 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=49 layout=158.35,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text seen=803,652,381,8 size=24 text="Shown in the library."
            58 Border at=64,700,1152,44 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              59 WrapPanel at=64,700,1152,44 by=panel limit=0,0,1280,800 role=none
                60 ActionButton at=64,700,77,44 by=panel drawn heard=STOP limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  61 TextBlock at=81,706,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=64,700,77,44 off owner=60 paints=#707070FF role=text size=26 text=STOP
            62 HintBar at=64,760,1152,24 by=panel heard="Hide…, Back, Finding, List, Apply…, More" limit=0,0,1280,800 role=group
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
edge 39 down same
edge 39 left same
edge 39 right 12
press pageup focus="Button 'No' (No)" top=LibraryWindow
press pagedown focus="Button 'No' (No)" top=LibraryWindow
press first focus="Button 'No' (No)" top=LibraryWindow
press last focus="Button 'No' (No)" top=LibraryWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Amber Saga (USA), Finding: Same dump as 1 other file)" top=LibraryWindow
