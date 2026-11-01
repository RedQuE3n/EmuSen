# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,68,1219.05,971
title Packages
case Packages
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,218,1221,792 by=panel limit=350.48,218,1219.05,791 paints=#9C9CA0FF role=none seen=350.48,218,1219.05,791
    1 PackagesPane at=386,236,1149,774 by=panel heard=Packages limit=350.48,218,1219.05,791 role=pane seen=386,236,1149,773
      2 Grid at=386,236,1149,774 by=panel limit=350.48,218,1219.05,791 role=none seen=386,236,1149,773
        3 DockPanel at=410,254,1101,69 by=panel limit=350.48,218,1219.05,791 role=none
          4 Border at=1241,254,270,69 by=panel limit=350.48,218,1219.05,791 name=AdaptiveSearch role=none
            5 TextBox at=1241,254,270,69 by=panel drawn focus heard=Search limit=350.48,218,1219.05,791 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,264.5,237,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,264.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,254,831,69 by=panel limit=350.48,218,1219.05,791 name=AdaptiveNavigation role=none
            9 ScrollViewer at=410,254,831,69 by=panel limit=350.48,218,1219.05,791 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=410,254,831,69
              10 StackPanel at=410,254,831,69 by=fixed-scroll in=9 limit=410,254,831,69 role=none
                11 ToggleButton at=410,255.5,99,66 by=fixed-scroll drawn focus heard=All in=9 limit=410,254,831,69 op paints=#FFFFFFFF role=button text=All
                  12 TextBlock at=435.5,264.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL in=9 layout=31.25,31.2 limit=410,255.5,99,66 owner=11 paints=#EEEEF0FF role=text size=26 text=ALL
                13 ToggleButton at=521,255.5,187.5,66 by=fixed-scroll drawn focus heard=Packaged in=9 limit=410,254,831,69 op paints=#FFFFFFFF role=button text=Packaged
                  14 TextBlock at=546.5,264.5,136.5,48 box=91,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PACKAGED in=9 layout=90.43,31.2 limit=521,255.5,187.5,66 owner=13 paints=#FFFFFFFF role=text size=26 text=PACKAGED
                15 ToggleButton at=720.5,255.5,162,66 by=fixed-scroll drawn focus heard="Not yet" in=9 limit=410,254,831,69 op paints=#FFFFFFFF role=button text="Not yet"
                  16 TextBlock at=746,264.5,111,48 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="NOT YET" in=9 layout=73.5,31.2 limit=720.5,255.5,162,66 owner=15 paints=#FFFFFFFF role=text size=26 text="NOT YET"
                17 ToggleButton at=894.5,255.5,190.5,66 by=fixed-scroll drawn focus heard=Problems in=9 limit=410,254,831,69 op paints=#FFFFFFFF role=button text=Problems
                  18 TextBlock at=920,264.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PROBLEMS in=9 layout=92.35,31.2 limit=894.5,255.5,190.5,66 owner=17 paints=#FFFFFFFF role=text size=26 text=PROBLEMS
        19 Grid at=410,341,1101,444 by=panel limit=350.48,218,1219.05,791 role=none
          20 Border at=410,341,1101,444 by=panel limit=350.48,218,1219.05,791 name=AdaptiveContent role=none
            21 Grid at=410,341,1101,444 by=panel limit=350.48,218,1219.05,791 role=none
              22 LunaTable at=410,341,1101,444 by=panel heard=Rows limit=350.48,218,1219.05,791 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                23 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=428,350,61.5,43.5 box=41,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                24 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=480.5,347,9,48 focus heard="Resize Mark" paints=#9C9CA0FF role=thumb
                25 Button allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=489.5,347,601.5,48 drawn focus heard="Game, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  26 StackPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=489.5,347,75,48 role=none
                    27 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=489.5,347,75,48 box=50,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=49.89,31.2 owner=25 paints=#9C9CA0FF role=text size=26 text=Game
                28 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1082,347,9,48 focus heard="Resize Game" paints=#9C9CA0FF role=thumb
                29 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1091,350,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=42.48,28.8 paints=#9C9CA0FF role=text size=24 text=State
                30 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,405.5,1101,379.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=410,405.5,1101,379.5
                  31 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,405.5,1101,63 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    32 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,413,1071,46.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=30 role=none
                      33 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,413,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=31 paints=#9C9CA0FF role=text size=24 text="☐"
                      34 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,414.5,601.5,43.5 box=401,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=30 layout=399.62,28.8 owner=31 paints=#9C9CA0FF role=edit size=24 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                      35 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,414.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=21.79,28.8 owner=31 paints=#9C9CA0FF role=edit size=24 text=OK
                  36 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,468.5,1101,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    37 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,476,1071,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=30 role=none
                      38 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,476,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=36 paints=#9C9CA0FF role=text size=24 text="☐"
                      39 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,477.5,601.5,43.5 box=401,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=203.04,28.8 owner=36 paints=#9C9CA0FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      40 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,477.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=21.79,28.8 owner=36 paints=#9C9CA0FF role=edit size=24 text=OK
                  41 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,531.5,1101,63 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    42 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,539,1071,46.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=30 role=none
                      43 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,539,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=41 paints=#9C9CA0FF role=text size=24 text="☐"
                      44 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,540.5,601.5,43.5 box=401,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=199.15,28.8 owner=41 paints=#9C9CA0FF role=edit size=24 text="Ruby Lantern (Synthetic)"
                      45 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,540.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=70.8,28.8 owner=41 paints=#9C9CA0FF role=edit size=24 text=Changed
                  46 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,594.5,1101,63 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    47 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,602,1071,46.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=30 role=none
                      48 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,602,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=46 paints=#EEEEF0FF role=text size=24 text="☐"
                      49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,603.5,601.5,43.5 box=401,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=30 layout=394.8,28.8 owner=46 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,603.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=112.3,28.8 owner=46 paints=#EEEEF0FF role=edit size=24 text="Not packaged"
                  51 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,657.5,1101,63 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    52 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,665,1071,46.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=30 role=none
                      53 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,665,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=51 paints=#9C9CA0FF role=text size=24 text="☐"
                      54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,666.5,601.5,43.5 box=401,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=147.12,28.8 owner=51 paints=#9C9CA0FF role=edit size=24 text="Ember Choir (USA)"
                      55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,666.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=112.3,28.8 owner=51 paints=#9C9CA0FF role=edit size=24 text="Not packaged"
                  56 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,720.5,1101,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=30 op paints=#333337FF,#9C9CA0FF role=listitem
                    57 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,728,1071,46.5 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=30 role=none
                      58 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,728,61.5,46.5 box=41,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=30 layout=14.4,30.84 owner=56 paints=#9C9CA0FF role=text size=24 text="☐"
                      59 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=486.5,729.5,601.5,43.5 box=401,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=143.54,28.8 owner=56 paints=#9C9CA0FF role=edit size=24 text="Amber Saga (USA)"
                      60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1088,729.5,361.5,43.5 box=241,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=112.3,28.8 owner=56 paints=#9C9CA0FF role=edit size=24 text="Not packaged"
        61 Border at=410,797,1101,66 by=panel limit=350.48,218,1219.05,791 name=AdaptiveActions role=none
          62 ScrollViewer at=410,797,1101,66 by=panel limit=350.48,218,1219.05,791 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=410,797,1101,66
            63 StackPanel at=410,797,1101,66 by=fixed-scroll in=62 limit=410,797,1101,66 role=none
              64 ActionButton at=410,797,123,66 by=fixed-scroll drawn focus heard=MAKE in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Make
                65 TextBlock at=435.5,806,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE in=62 layout=47.11,31.2 limit=410,797,123,66 owner=64 paints=#9C9CA0FF role=text size=26 text=MAKE
              66 ActionButton at=551,797,139.5,66 by=fixed-scroll drawn focus heard=CHECK in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Check
                67 TextBlock at=576.5,806,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK in=62 layout=58.19,31.2 limit=551,797,139.5,66 owner=66 paints=#9C9CA0FF role=text size=26 text=CHECK
              68 ActionButton at=708.5,797,93,66 by=fixed-scroll drawn focus heard=ZIP in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
                69 TextBlock at=734,806,42,48 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP in=62 layout=27.35,31.2 limit=708.5,797,93,66 owner=68 paints=#9C9CA0FF role=text size=26 text=ZIP
              70 ActionButton at=819.5,797,276,66 by=fixed-scroll drawn focus heard="CONVERT TO CHD" in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
                71 TextBlock at=845,806,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" in=62 layout=149.58,31.2 limit=819.5,797,276,66 owner=70 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
              72 ActionButton at=1113.5,797,169.5,66 by=fixed-scroll drawn focus heard=REFRESH in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
                73 TextBlock at=1139,806,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH in=62 layout=78.94,31.2 limit=1113.5,797,169.5,66 owner=72 paints=#9C9CA0FF role=text size=26 text=REFRESH
              74 ActionButton at=1301,797,123,66 by=fixed-scroll drawn focus heard=MARK in=62 limit=410,797,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
                75 TextBlock at=1326.5,806,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK in=62 layout=47.48,31.2 limit=1301,797,123,66 owner=74 paints=#9C9CA0FF role=text size=26 text=MARK
        76 Border at=386,875,1149,78 by=panel limit=350.48,218,1219.05,791 name=AdaptiveStatus role=none
          77 DockPanel at=392,881,1137,66 by=panel limit=350.48,218,1219.05,791 role=none
            78 StackPanel at=761,881,768,66 by=panel limit=350.48,218,1219.05,791 role=none
              79 Button at=761,881,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,218,1219.05,791 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                80 TextBlock at=786.5,890,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=761,881,157.5,66 off owner=79 paints=#55555AFF role=text size=26 text="APPLY…"
              81 MeterRow at=936.5,881,592.5,66 by=panel drawn heard=Space limit=350.48,218,1219.05,791 paints=#9C9CA0FF role=progressbar
                82 TextBlock at=936.5,893,210,43.5 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Space layout=49.58,28.8 limit=936.5,881,592.5,66 paints=#B4B4B8FF part role=text size=24 text=Space
                83 ProgressBar at=1146.5,903.5,300,21 drawn name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                84 TextBlock at=1458.5,893,70.5,43.5 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=60% layout=39.24,28.8 limit=936.5,881,592.5,66 paints=#B4B4B8FF part role=text size=24 text=60%
            85 StackPanel at=392,881,369,66 by=panel limit=350.48,218,1219.05,791 role=none
              86 TextBlock at=392,893,196.5,43.5 box=131,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=350.48,218,1219.05,791 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
        87 HintBar at=410,965,1101,33 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=350.48,218,1219.05,791 role=group
focus 24
edge 5 up same
edge 5 down 31
edge 5 left 17
edge 5 right same
edge 11 up same
edge 11 down 24
edge 11 left same
edge 11 right 13
edge 13 up same
edge 13 down 25
edge 13 left 11
edge 13 right 15
edge 15 up same
edge 15 down 25
edge 15 left 13
edge 15 right 17
edge 17 up same
edge 17 down 25
edge 17 left 15
edge 17 right 5
edge 24 up 11
edge 24 down 31
edge 24 left 11
edge 24 right 25
edge 25 up 13
edge 25 down 31
edge 25 left 24
edge 25 right 5
edge 31 up 24
edge 31 down 36
edge 31 left same
edge 31 right 5
edge 36 up 31
edge 36 down 41
edge 36 left same
edge 36 right 5
edge 41 up 36
edge 41 down 46
edge 41 left same
edge 41 right 5
edge 46 up 41
edge 46 down 51
edge 46 left same
edge 46 right 74
edge 51 up 46
edge 51 down 56
edge 51 left same
edge 51 right 74
edge 56 up 51
edge 56 down 64
edge 56 left same
edge 56 right 74
edge 64 up 56
edge 64 down same
edge 64 left same
edge 64 right 66
edge 66 up 56
edge 66 down same
edge 66 left 64
edge 66 right 68
edge 68 up 56
edge 68 down same
edge 68 left 66
edge 68 right 70
edge 70 up 56
edge 70 down same
edge 70 left 68
edge 70 right 72
edge 72 up 56
edge 72 down same
edge 72 left 70
edge 72 right 74
edge 74 up 56
edge 74 down same
edge 74 left 72
edge 74 right same
press pageup focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press pagedown focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press first focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press last focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press back top=none
