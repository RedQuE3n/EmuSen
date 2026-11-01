# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Packages
case PackagesDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 PackagesPane at=386,205,1149,804 by=panel heard=Packages limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 ScrollViewer at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=410,223,831,69
              10 StackPanel at=410,223,831,69 by=fixed-scroll in=9 limit=410,223,831,69 role=none
                11 ToggleButton at=410,224.5,99,66 by=fixed-scroll drawn focus heard=All in=9 limit=410,223,831,69 op paints=#FFFFFFFF role=button text=All
                  12 TextBlock at=435.5,233.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL in=9 layout=31.25,31.2 limit=410,224.5,99,66 owner=11 paints=#EEEEF0FF role=text size=26 text=ALL
                13 ToggleButton at=521,224.5,187.5,66 by=fixed-scroll drawn focus heard=Packaged in=9 limit=410,223,831,69 op paints=#FFFFFFFF role=button text=Packaged
                  14 TextBlock at=546.5,233.5,136.5,48 box=91,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PACKAGED in=9 layout=90.43,31.2 limit=521,224.5,187.5,66 owner=13 paints=#FFFFFFFF role=text size=26 text=PACKAGED
                15 ToggleButton at=720.5,224.5,162,66 by=fixed-scroll drawn focus heard="Not yet" in=9 limit=410,223,831,69 op paints=#FFFFFFFF role=button text="Not yet"
                  16 TextBlock at=746,233.5,111,48 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="NOT YET" in=9 layout=73.5,31.2 limit=720.5,224.5,162,66 owner=15 paints=#FFFFFFFF role=text size=26 text="NOT YET"
                17 ToggleButton at=894.5,224.5,190.5,66 by=fixed-scroll drawn focus heard=Problems in=9 limit=410,223,831,69 op paints=#FFFFFFFF role=button text=Problems
                  18 TextBlock at=920,233.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PROBLEMS in=9 layout=92.35,31.2 limit=894.5,224.5,190.5,66 owner=17 paints=#FFFFFFFF role=text size=26 text=PROBLEMS
        19 Grid at=410,310,1101,474 by=panel limit=350.48,187,1219.05,822 role=none
          20 Border at=410,310,1101,474 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            21 Border at=410,310,1101,474 by=panel limit=350.48,187,1219.05,822 role=none
              22 ScrollViewer at=422,322,1077,450 by=panel fades=b limit=350.48,187,1219.05,822 overflow=712.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,450
                23 PackageDetail at=422,322,1077,1162.5 by=scroll in=22 limit=422,322,1077,450 name=PackagesDetail paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=422,322,1077,450
                  24 FieldRow at=422,322,1077,136.5 by=scroll heard=Package in=22 limit=422,322,1077,450 paints=#9C9CA0FF role=group
                    25 TextBlock at=422,322,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=22 layout=69.48,28.8 limit=422,322,1077,136.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
                    26 TextBlock at=422,371.5,1077,87 box=718,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=22 layout=675.02,57.6 limit=422,322,1077,136.5 paints=#9C9CA0FF role=text size=24 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                  27 FieldRow at=422,473.5,1077,136.5 by=scroll heard=Game in=22 limit=422,322,1077,450 paints=#9C9CA0FF role=group
                    28 TextBlock at=422,473.5,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Game in=22 layout=46.06,28.8 limit=422,473.5,1077,136.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Game
                    29 TextBlock at=422,523,1077,87 box=718,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=22 layout=674.42,57.6 limit=422,473.5,1077,136.5 paints=#9C9CA0FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                  30 FieldRow at=422,625,1077,93 by=scroll heard=State in=22 limit=422,322,1077,450 paints=#9C9CA0FF role=group
                    31 TextBlock at=422,625,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=State in=22 layout=42.48,28.8 limit=422,625,1077,93 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=State
                    32 TextBlock at=422,674.5,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not packaged" in=22 layout=112.3,28.8 limit=422,625,1077,93 paints=#9C9CA0FF role=text size=24 text="Not packaged"
                  33 FieldRow at=422,733,1077,93 by=scroll heard=Size in=22 limit=422,322,1077,450 paints=#9C9CA0FF role=group seen=422,733,1077,39
                    34 TextBlock at=422,733,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Size in=22 layout=33.41,28.8 limit=422,733,1077,93 name=PART_Label paints=#9C9CA0FF part role=text seen=422,733,1077,39 size=24 text=Size
        35 Border at=410,796,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          36 ScrollViewer at=410,796,1101,66 by=panel limit=350.48,187,1219.05,822 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=410,796,1101,66
            37 StackPanel at=410,796,1101,66 by=fixed-scroll in=36 limit=410,796,1101,66 role=none
              38 ActionButton at=410,796,123,66 by=fixed-scroll drawn focus heard=MAKE in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Make
                39 TextBlock at=435.5,805,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE in=36 layout=47.11,31.2 limit=410,796,123,66 owner=38 paints=#9C9CA0FF role=text size=26 text=MAKE
              40 ActionButton at=551,796,139.5,66 by=fixed-scroll drawn focus heard=CHECK in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Check
                41 TextBlock at=576.5,805,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK in=36 layout=58.19,31.2 limit=551,796,139.5,66 owner=40 paints=#9C9CA0FF role=text size=26 text=CHECK
              42 ActionButton at=708.5,796,93,66 by=fixed-scroll drawn focus heard=ZIP in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
                43 TextBlock at=734,805,42,48 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP in=36 layout=27.35,31.2 limit=708.5,796,93,66 owner=42 paints=#9C9CA0FF role=text size=26 text=ZIP
              44 ActionButton at=819.5,796,276,66 by=fixed-scroll drawn focus heard="CONVERT TO CHD" in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
                45 TextBlock at=845,805,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" in=36 layout=149.58,31.2 limit=819.5,796,276,66 owner=44 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
              46 ActionButton at=1113.5,796,169.5,66 by=fixed-scroll drawn focus heard=REFRESH in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
                47 TextBlock at=1139,805,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH in=36 layout=78.94,31.2 limit=1113.5,796,169.5,66 owner=46 paints=#9C9CA0FF role=text size=26 text=REFRESH
              48 ActionButton at=1301,796,123,66 by=fixed-scroll drawn focus heard=MARK in=36 limit=410,796,1101,66 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
                49 TextBlock at=1326.5,805,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK in=36 layout=47.48,31.2 limit=1301,796,123,66 owner=48 paints=#9C9CA0FF role=text size=26 text=MARK
        50 Border at=386,874,1149,78 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          51 DockPanel at=392,880,1137,66 by=panel limit=350.48,187,1219.05,822 role=none
            52 StackPanel at=761,880,768,66 by=panel limit=350.48,187,1219.05,822 role=none
              53 Button at=761,880,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                54 TextBlock at=786.5,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=761,880,157.5,66 off owner=53 paints=#55555AFF role=text size=26 text="APPLY…"
              55 MeterRow at=936.5,880,592.5,66 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=progressbar
                56 TextBlock at=936.5,892,210,43.5 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Space layout=49.58,28.8 limit=936.5,880,592.5,66 paints=#B4B4B8FF part role=text size=24 text=Space
                57 ProgressBar at=1146.5,902.5,300,21 drawn name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                58 TextBlock at=1458.5,892,70.5,43.5 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=60% layout=39.24,28.8 limit=936.5,880,592.5,66 paints=#B4B4B8FF part role=text size=24 text=60%
            59 StackPanel at=392,880,369,66 by=panel limit=350.48,187,1219.05,822 role=none
              60 TextBlock at=392,892,196.5,43.5 box=131,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
        61 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 5
edge 5 up same
edge 5 down 48
edge 5 left 17
edge 5 right same
edge 11 up same
edge 11 down 38
edge 11 left same
edge 11 right 13
edge 13 up same
edge 13 down 40
edge 13 left 11
edge 13 right 15
edge 15 up same
edge 15 down 42
edge 15 left 13
edge 15 right 17
edge 17 up same
edge 17 down 44
edge 17 left 15
edge 17 right 5
edge 38 up 11
edge 38 down item:ButtonBar/0//
edge 38 left same
edge 38 right 40
edge 40 up 13
edge 40 down item:ButtonBar/0//
edge 40 left 38
edge 40 right 42
edge 42 up 15
edge 42 down item:ButtonBar/0//
edge 42 left 40
edge 42 right 44
edge 44 up 15
edge 44 down item:ButtonBar/0//
edge 44 left 42
edge 44 right 46
edge 46 up 17
edge 46 down item:ButtonBar/0//
edge 46 left 44
edge 46 right 48
edge 48 up 5
edge 48 down item:ButtonBar/0//
edge 48 left 46
edge 48 right same
edge item:ButtonBar/0// up 5
edge item:ButtonBar/0// down 48
edge item:ButtonBar/0// left 44
edge item:ButtonBar/0// right same
press pageup focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
