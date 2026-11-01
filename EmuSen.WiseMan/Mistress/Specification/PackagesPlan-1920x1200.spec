# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Packages
case PackagesPlan
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 PackagesPane at=386,205,1149,804 by=panel heard=Packages limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,99,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=410,224.5,99,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=521,224.5,103.5,66 by=panel drawn focus heard=NES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=546.5,233.5,52.5,48 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=34.79,31.2 limit=521,224.5,103.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=636.5,224.5,223.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=662,233.5,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=636.5,224.5,223.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=872,224.5,160.5,66 by=panel drawn focus heard="Sega CD" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=897.5,233.5,109.5,48 box=73,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=72.96,31.2 limit=872,224.5,160.5,66 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=1044.5,224.5,120,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=1070,233.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=1044.5,224.5,120,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
          21 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            22 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              23 Border at=437,310,1045.5,418.5 by=panel limit=350.48,187,1219.05,822 paints=#1B1B1EFF,#55555AFF role=none
                24 DockPanel at=462.5,335.5,994.5,367.5 by=panel limit=350.48,187,1219.05,822 role=none
                  25 TextBlock at=462.5,335.5,994.5,43.5 box=663,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=175.61,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Apply these changes?"
                  26 StackPanel at=1157,637,300,66 by=panel limit=350.48,187,1219.05,822 role=none
                    27 Button at=1157,637,154.5,66 by=panel drawn focus heard=Cancel limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
                      28 TextBlock at=1182.5,646,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1157,637,154.5,66 owner=27 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    29 Button at=1323.5,637,133.5,66 by=panel drawn focus heard=Apply limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Apply
                      30 TextBlock at=1349,646,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=54.29,31.2 limit=1323.5,637,133.5,66 owner=29 paints=#9C9CA0FF role=text size=26 text=APPLY
                  31 ScrollViewer at=462.5,397,994.5,222 by=panel fades=b limit=350.48,187,1219.05,822 overflow=81 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=462.5,397,994.5,222
                    32 TextBlock at=462.5,397,994.5,303 box=663,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=31 layout=657.86,201.6 limit=462.5,397,994.5,222 paints=#9C9CA0FF role=text seen=462.5,397,994.5,222 size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
        33 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          34 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            35 ActionButton at=410,740.5,123,66 by=panel drawn focus heard=MAKE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Make
              36 TextBlock at=435.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=47.11,31.2 limit=410,740.5,123,66 owner=35 paints=#9C9CA0FF role=text size=26 text=MAKE
            37 ActionButton at=551,740.5,139.5,66 by=panel drawn focus heard=CHECK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Check
              38 TextBlock at=576.5,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=58.19,31.2 limit=551,740.5,139.5,66 owner=37 paints=#9C9CA0FF role=text size=26 text=CHECK
            39 ActionButton at=708.5,740.5,93,66 by=panel drawn focus heard=ZIP limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
              40 TextBlock at=734,749.5,42,48 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=27.35,31.2 limit=708.5,740.5,93,66 owner=39 paints=#9C9CA0FF role=text size=26 text=ZIP
            41 ActionButton at=819.5,740.5,276,66 by=panel drawn focus heard="CONVERT TO CHD" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
              42 TextBlock at=845,749.5,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=149.58,31.2 limit=819.5,740.5,276,66 owner=41 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
            43 ActionButton at=1113.5,740.5,169.5,66 by=panel drawn focus heard=REFRESH limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
              44 TextBlock at=1139,749.5,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=1113.5,740.5,169.5,66 owner=43 paints=#9C9CA0FF role=text size=26 text=REFRESH
            45 ActionButton at=1301,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              46 TextBlock at=1326.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1301,740.5,123,66 owner=45 paints=#9C9CA0FF role=text size=26 text=MARK
        47 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          48 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            49 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              50 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=340.44,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
            51 StackPanel at=648.5,880,880.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              52 Button at=648.5,880,157.5,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                53 TextBlock at=674,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=648.5,880,157.5,66 owner=52 paints=#9C9CA0FF role=text size=26 text="APPLY…"
              54 StackPanel at=824,880,705,66 by=panel limit=350.48,187,1219.05,822 role=none
                55 ProgressBar at=824,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                56 TextBlock at=1136,892,393,43.5 box=262,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=261.82,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="263 KB in packages · 412 GB free"
        57 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 27
edge 5 up same
edge 5 down 27
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 35
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 37
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 37
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 41
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 41
edge 18 left 16
edge 18 right 5
edge 27 up 5
edge 27 down 43
edge 27 left 41
edge 27 right 29
edge 29 up 5
edge 29 down 45
edge 29 left 27
edge 29 right same
edge 35 up 10
edge 35 down 52
edge 35 left same
edge 35 right 37
edge 37 up 12
edge 37 down 52
edge 37 left 35
edge 37 right 39
edge 39 up 14
edge 39 down 52
edge 39 left 37
edge 39 right 41
edge 41 up 16
edge 41 down 52
edge 41 left 39
edge 41 right 43
edge 43 up 27
edge 43 down 52
edge 43 left 41
edge 43 right 45
edge 45 up 29
edge 45 down 52
edge 45 left 43
edge 45 right same
edge 52 up 37
edge 52 down same
edge 52 left 35
edge 52 right 41
press pageup focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press first focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press last focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
