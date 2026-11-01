# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Packages
case PackagesPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,65,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,149,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,149,67,44 by=panel drawn focus heard=NES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=364,155,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=347,149,67,44 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=422,149,141,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=439,155,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=422,149,141,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=571,149,104,44 by=panel drawn focus heard="Sega CD" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=588,155,70,32 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=571,149,104,44 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=683,149,77,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=700,155,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=683,149,77,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          21 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            22 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              23 Border at=284,206,713,279 by=panel limit=234,124,812.35,548 paints=#1B1B1EFF,#707070FF role=none
                24 DockPanel at=301,223,679,245 by=panel limit=234,124,812.35,548 role=none
                  25 TextBlock at=301,223,679,29 box=679,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  26 StackPanel at=788,424,192,44 by=panel limit=234,124,812.35,548 role=none
                    27 Button at=788,424,99,44 by=panel drawn focus heard=Cancel limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      28 TextBlock at=805,430,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=788,424,99,44 owner=27 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    29 Button at=895,424,85,44 by=panel drawn focus heard=Apply limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      30 TextBlock at=912,430,51,32 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=895,424,85,44 owner=29 paints=#D4D4D4FF role=text size=26 text=APPLY
                  31 ScrollViewer at=301,264,679,148 by=panel fades=b limit=234,124,812.35,548 overflow=54 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=301,264,679,148
                    32 TextBlock at=301,264,679,202 box=679,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=31 layout=673.85,201.6 limit=301,264,679,148 paints=#D4D4D4FF role=text seen=301,264,679,148 size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
        33 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          34 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            35 ActionButton at=274,493,80,44 by=panel drawn focus heard=MAKE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Make
              36 TextBlock at=291,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=45.21,31.2 limit=274,493,80,44 owner=35 paints=#D4D4D4FF role=text size=26 text=MAKE
            37 ActionButton at=366,493,89,44 by=panel drawn focus heard=CHECK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Check
              38 TextBlock at=383,499,55,32 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=54.55,31.2 limit=366,493,89,44 owner=37 paints=#D4D4D4FF role=text size=26 text=CHECK
            39 ActionButton at=467,493,60,44 by=panel drawn focus heard=ZIP limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Zip
              40 TextBlock at=484,499,26,32 box=26,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=25.45,31.2 limit=467,493,60,44 owner=39 paints=#D4D4D4FF role=text size=26 text=ZIP
            41 ActionButton at=539,493,176,44 by=panel drawn focus heard="CONVERT TO CHD" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Convert to CHD"
              42 TextBlock at=556,499,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=141.08,31.2 limit=539,493,176,44 owner=41 paints=#D4D4D4FF role=text size=26 text="CONVERT TO CHD"
            43 ActionButton at=727,493,109,44 by=panel drawn focus heard=REFRESH limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Refresh
              44 TextBlock at=744,499,75,32 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=727,493,109,44 owner=43 paints=#D4D4D4FF role=text size=26 text=REFRESH
            45 ActionButton at=848,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              46 TextBlock at=865,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=848,493,80,44 owner=45 paints=#D4D4D4FF role=text size=26 text=MARK
        47 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          48 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            49 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              50 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=326.38,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
            51 StackPanel at=444,586,575,44 by=panel limit=234,124,812.35,548 role=none
              52 Button at=444,586,102,44 by=panel drawn focus heard="Apply the marked changes" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                53 TextBlock at=461,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=444,586,102,44 owner=52 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              54 StackPanel at=558,586,461,44 by=panel limit=234,124,812.35,548 role=none
                55 ProgressBar at=558,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                56 TextBlock at=766,594,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
        57 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=234,124,812.35,548 role=group
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
edge 45 up 27
edge 45 down 52
edge 45 left 43
edge 45 right same
edge 52 up 39
edge 52 down same
edge 52 left 35
edge 52 right 43
press pageup focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press first focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press last focus="Button 'Cancel' (Cancel)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
