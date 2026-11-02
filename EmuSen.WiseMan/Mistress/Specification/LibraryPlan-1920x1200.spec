# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case LibraryPlan
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
                    33 Border at=96,282,1036.5,630 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF,#707070FF role=none
                      34 DockPanel at=121.5,307.5,985.5,579 by=panel limit=0,0,1920,1200 role=none
                        35 TextBlock at=121.5,307.5,985.5,58.5 box=657,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        36 StackPanel at=121.5,748.5,985.5,138 by=panel limit=0,0,1920,1200 role=none
                          37 Button at=121.5,748.5,985.5,66 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=No
                            38 TextBlock at=147,757.5,36,48 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=121.5,748.5,985.5,66 owner=37 paints=#EEEEF0FF role=text size=26 text=NO
                          39 Button at=121.5,820.5,985.5,66 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            40 TextBlock at=147,829.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=121.5,820.5,985.5,66 owner=39 paints=#D4D4D4FF role=text size=26 text=YES
                        41 ScrollViewer at=121.5,384,985.5,346.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=121.5,384,985.5,346.5
                          42 TextBlock at=121.5,384,985.5,346.5 box=657,231 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=41 layout=655.01,230.4 limit=121.5,384,985.5,346.5 paints=#D4D4D4FF role=text size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
                43 Border at=96,970.5,1036.5,67.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  44 Grid at=102,976.5,1024.5,55.5 by=panel limit=0,0,1920,1200 role=none
                    45 Grid at=102,976.5,1024.5,43.5 by=panel limit=0,0,1920,1200 role=none
                      46 TextBlock at=114,976.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="2 to hide"
              47 Border at=1156.5,223.5,667.5,814.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                48 Border at=1192.5,259.5,595.5,742.5 by=panel limit=0,0,1920,1200 role=none
                  49 ScrollViewer at=1204.5,271.5,571.5,718.5 by=panel fades=b limit=0,0,1920,1200 overflow=31.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1204.5,271.5,571.5,718.5
                    50 StackPanel at=1204.5,271.5,571.5,750 by=scroll in=49 limit=1204.5,271.5,571.5,718.5 name=LibraryDetail role=none seen=1204.5,271.5,571.5,718.5
                      51 TextBlock at=1204.5,271.5,571.5,87 box=381,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=49 layout=361.46,57.6 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      52 TextBlock at=1204.5,367.5,571.5,130.5 box=381,87 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=49 layout=366.34,86.4 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      53 TextBlock at=1204.5,507,571.5,174 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=49 layout=377.16,115.2 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      54 TextBlock at=1204.5,690,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=49 layout=93.91,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      55 TextBlock at=1204.5,742.5,571.5,174 box=381,116 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=49 layout=362.59,115.2 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                      56 TextBlock at=1204.5,925.5,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Header checksum: not checked yet." in=49 layout=278.28,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Header checksum: not checked yet."
                      57 TextBlock at=1204.5,978,571.5,43.5 box=381,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Shown in the library." in=49 layout=158.35,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text seen=1204.5,978,571.5,12 size=24 text="Shown in the library."
            58 Border at=96,1050,1728,66 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              59 WrapPanel at=96,1050,1728,66 by=panel limit=0,0,1920,1200 role=none
                60 ActionButton at=96,1050,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  61 TextBlock at=121.5,1059,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1050,115.5,66 off owner=60 paints=#707070FF role=text size=26 text=STOP
            62 HintBar at=96,1140,1728,36 by=panel heard="Hide…, Back, Finding, List, Apply…, More" limit=0,0,1920,1200 role=group
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
