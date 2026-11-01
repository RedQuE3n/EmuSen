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
          21 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            22 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              23 ScrollViewer at=422,322,1077,394.5 by=panel fades=t limit=350.48,187,1219.05,822 overflow=768 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,394.5
                24 PackageDetail at=422,-249.5,1077,1162.5 by=scroll in=23 limit=422,322,1077,394.5 name=PackagesDetail paints=#9C9CA0FF role=none seen=422,322,1077,394.5
                  25 FieldRow at=422,269.5,1077,93 by=scroll heard=Format in=23 limit=422,322,1077,394.5 paints=#9C9CA0FF role=group seen=422,322,1077,40.5
                    26 TextBlock at=422,319,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CUE/BIN, which can be converted to CHD" in=23 layout=329.71,28.8 limit=422,269.5,1077,93 paints=#9C9CA0FF role=text seen=422,322,1077,40.5 size=24 text="CUE/BIN, which can be converted to CHD"
                  27 GamePackageDetail at=422,377.5,1077,535.5 by=scroll in=23 limit=422,322,1077,394.5 name=GamePackageDetail paints=#86868AFF,#9C9CA0FF role=none seen=422,377.5,1077,339
                    28 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
                    29 FieldRow at=422,545.5,1077,171 by=scroll heard=Package in=23 limit=422,322,1077,394.5 paints=#9C9CA0FF role=group
                      30 TextBlock at=422,545.5,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=23 layout=69.48,28.8 limit=422,545.5,1077,171 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
                      31 StackPanel at=422,595,1077,121.5 by=clip heard=Package in=23 limit=422,545.5,1077,171 role=none
                        32 TextBlock at=422,595,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=23 layout=150.96,28.8 limit=422,545.5,1077,171 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Not a package yet."
                        33 ButtonBar at=1251.5,650.5,247.5,66 by=clip in=23 limit=422,545.5,1077,171 paints=#9C9CA0FF role=toolbar
                          34 Button at=1251.5,650.5,247.5,66 by=clip drawn focus heard="MAKE PACKAGE" in=23 limit=1251.5,650.5,247.5,66 name=GamePackageMake op paints=#55555AFF,#9C9CA0FF role=button text="Make Package"
                            35 TextBlock at=1277,659.5,196.5,48 box=131,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=23 layout=130.65,31.2 limit=1251.5,650.5,247.5,66 owner=34 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
        36 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          37 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            38 ActionButton at=410,740.5,123,66 by=panel drawn focus heard=MAKE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Make
              39 TextBlock at=435.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=47.11,31.2 limit=410,740.5,123,66 owner=38 paints=#9C9CA0FF role=text size=26 text=MAKE
            40 ActionButton at=551,740.5,139.5,66 by=panel drawn focus heard=CHECK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Check
              41 TextBlock at=576.5,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=58.19,31.2 limit=551,740.5,139.5,66 owner=40 paints=#9C9CA0FF role=text size=26 text=CHECK
            42 ActionButton at=708.5,740.5,93,66 by=panel drawn focus heard=ZIP limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
              43 TextBlock at=734,749.5,42,48 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=27.35,31.2 limit=708.5,740.5,93,66 owner=42 paints=#9C9CA0FF role=text size=26 text=ZIP
            44 ActionButton at=819.5,740.5,276,66 by=panel drawn focus heard="CONVERT TO CHD" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
              45 TextBlock at=845,749.5,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=149.58,31.2 limit=819.5,740.5,276,66 owner=44 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
            46 ActionButton at=1113.5,740.5,169.5,66 by=panel drawn focus heard=REFRESH limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
              47 TextBlock at=1139,749.5,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=1113.5,740.5,169.5,66 owner=46 paints=#9C9CA0FF role=text size=26 text=REFRESH
            48 ActionButton at=1301,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              49 TextBlock at=1326.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1301,740.5,123,66 owner=48 paints=#9C9CA0FF role=text size=26 text=MARK
        50 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          51 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            52 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              53 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            54 StackPanel at=648.5,880,880.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              55 Button at=648.5,880,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                56 TextBlock at=674,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=648.5,880,157.5,66 off owner=55 paints=#55555AFF role=text size=26 text="APPLY…"
              57 StackPanel at=824,880,705,66 by=panel limit=350.48,187,1219.05,822 role=none
                58 ProgressBar at=824,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                59 TextBlock at=1136,892,393,43.5 box=262,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=261.82,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="263 KB in packages · 412 GB free"
        60 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 34
edge 5 up same
edge 5 down 34
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 38
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 40
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 40
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 44
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 44
edge 18 left 16
edge 18 right 5
edge 34 up 5
edge 34 down 48
edge 34 left 44
edge 34 right same
edge 38 up 10
edge 38 down same
edge 38 left same
edge 38 right 40
edge 40 up 12
edge 40 down same
edge 40 left 38
edge 40 right 42
edge 42 up 14
edge 42 down same
edge 42 left 40
edge 42 right 44
edge 44 up 16
edge 44 down same
edge 44 left 42
edge 44 right 46
edge 46 up 18
edge 46 down same
edge 46 left 44
edge 46 right 48
edge 48 up 34
edge 48 down same
edge 48 left 46
edge 48 right same
press pageup title=Packages top=PackagesWindow
press pagedown title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
