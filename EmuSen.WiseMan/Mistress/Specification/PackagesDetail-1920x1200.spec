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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 PackagesPane at=386,205,1149,804 by=panel heard=Packages limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,97.5,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=410,224.5,97.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=519.5,224.5,100.5,66 by=panel drawn focus heard=NES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=545,233.5,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=519.5,224.5,100.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=632,224.5,211.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=657.5,233.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=632,224.5,211.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=855.5,224.5,156,66 by=panel drawn focus heard="Sega CD" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=881,233.5,105,48 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=855.5,224.5,156,66 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=1023.5,224.5,115.5,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=1049,233.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=1023.5,224.5,115.5,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
          21 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            22 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              23 ScrollViewer at=422,322,1077,394.5 by=panel fades=t limit=350.48,187,1219.05,822 overflow=732 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,394.5
                24 PackageDetail at=422,-213.5,1077,1126.5 by=scroll in=23 limit=422,322,1077,394.5 name=PackagesDetail paints=#D4D4D4FF role=none seen=422,322,1077,394.5
                  25 FieldRow at=422,305.5,1077,93 by=scroll heard=Format in=23 limit=422,322,1077,394.5 paints=#D4D4D4FF role=group seen=422,322,1077,76.5
                    26 TextBlock at=422,305.5,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Format in=23 layout=54.29,28.8 limit=422,305.5,1077,93 name=PART_Label paints=#D4D4D4FF part role=text seen=422,322,1077,27 size=24 text=Format
                    27 TextBlock at=422,355,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CUE/BIN, which can be converted to CHD" in=23 layout=312.96,28.8 limit=422,305.5,1077,93 paints=#D4D4D4FF role=text size=24 text="CUE/BIN, which can be converted to CHD"
                  28 GamePackageDetail at=422,413.5,1077,499.5 by=scroll in=23 limit=422,322,1077,394.5 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=422,413.5,1077,303
                    29 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                    30 FieldRow at=422,545.5,1077,171 by=scroll heard=Package in=23 limit=422,322,1077,394.5 paints=#D4D4D4FF role=group
                      31 TextBlock at=422,545.5,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=23 layout=65.81,28.8 limit=422,545.5,1077,171 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
                      32 StackPanel at=422,595,1077,121.5 by=clip heard=Package in=23 limit=422,545.5,1077,171 role=none
                        33 TextBlock at=422,595,1077,43.5 box=718,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=23 layout=144.72,28.8 limit=422,545.5,1077,171 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
                        34 ButtonBar at=1259,650.5,240,66 by=clip in=23 limit=422,545.5,1077,171 paints=#D4D4D4FF role=toolbar
                          35 Button at=1259,650.5,240,66 by=clip drawn focus heard="MAKE PACKAGE" in=23 limit=1259,650.5,240,66 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                            36 TextBlock at=1284.5,659.5,189,48 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=23 layout=125.29,31.2 limit=1259,650.5,240,66 owner=35 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
        37 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          38 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            39 ActionButton at=410,740.5,120,66 by=panel drawn focus heard=MAKE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Make
              40 TextBlock at=435.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=45.21,31.2 limit=410,740.5,120,66 owner=39 paints=#D4D4D4FF role=text size=26 text=MAKE
            41 ActionButton at=548,740.5,133.5,66 by=panel drawn focus heard=CHECK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Check
              42 TextBlock at=573.5,749.5,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=54.55,31.2 limit=548,740.5,133.5,66 owner=41 paints=#D4D4D4FF role=text size=26 text=CHECK
            43 ActionButton at=699.5,740.5,90,66 by=panel drawn focus heard=ZIP limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Zip
              44 TextBlock at=725,749.5,39,48 box=26,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=25.45,31.2 limit=699.5,740.5,90,66 owner=43 paints=#D4D4D4FF role=text size=26 text=ZIP
            45 ActionButton at=807.5,740.5,264,66 by=panel drawn focus heard="CONVERT TO CHD" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Convert to CHD"
              46 TextBlock at=833,749.5,213,48 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=141.08,31.2 limit=807.5,740.5,264,66 owner=45 paints=#D4D4D4FF role=text size=26 text="CONVERT TO CHD"
            47 ActionButton at=1089.5,740.5,163.5,66 by=panel drawn focus heard=REFRESH limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Refresh
              48 TextBlock at=1115,749.5,112.5,48 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=1089.5,740.5,163.5,66 owner=47 paints=#D4D4D4FF role=text size=26 text=REFRESH
            49 ActionButton at=1271,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              50 TextBlock at=1296.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1271,740.5,120,66 owner=49 paints=#D4D4D4FF role=text size=26 text=MARK
        51 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          52 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            53 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              54 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            55 StackPanel at=666.5,880,862.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              56 Button at=666.5,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                57 TextBlock at=692,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=666.5,880,153,66 off owner=56 paints=#707070FF role=text size=26 text="APPLY…"
              58 StackPanel at=837.5,880,691.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                59 ProgressBar at=837.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                60 TextBlock at=1149.5,892,379.5,43.5 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
        61 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 35
edge 5 up same
edge 5 down 35
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 39
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 41
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 41
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 45
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 45
edge 18 left 16
edge 18 right 5
edge 35 up 5
edge 35 down 49
edge 35 left 47
edge 35 right same
edge 39 up 10
edge 39 down same
edge 39 left same
edge 39 right 41
edge 41 up 12
edge 41 down same
edge 41 left 39
edge 41 right 43
edge 43 up 14
edge 43 down same
edge 43 left 41
edge 43 right 45
edge 45 up 16
edge 45 down same
edge 45 left 43
edge 45 right 47
edge 47 up 18
edge 47 down same
edge 47 left 45
edge 47 right 49
edge 49 up 35
edge 49 down same
edge 49 left 47
edge 49 right same
press pageup title=Packages top=PackagesWindow
press pagedown title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
