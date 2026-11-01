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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,66,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=274,149,66,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=348,149,69,44 by=panel drawn focus heard=NES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=365,155,35,32 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=34.79,31.2 limit=348,149,69,44 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=425,149,149,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=442,155,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=425,149,149,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=582,149,107,44 by=panel drawn focus heard="Sega CD" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=599,155,73,32 box=73,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=72.96,31.2 limit=582,149,107,44 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=697,149,80,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=714,155,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=697,149,80,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          21 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            22 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              23 Border at=292,206,697,279 by=panel limit=234,124,812.35,548 paints=#1B1B1EFF,#55555AFF role=none
                24 DockPanel at=309,223,663,245 by=panel limit=234,124,812.35,548 role=none
                  25 TextBlock at=309,223,663,29 box=663,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=175.61,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="Apply these changes?"
                  26 StackPanel at=772,424,200,44 by=panel limit=234,124,812.35,548 role=none
                    27 Button at=772,424,103,44 by=panel drawn focus heard=Cancel limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
                      28 TextBlock at=789,430,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=772,424,103,44 owner=27 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    29 Button at=883,424,89,44 by=panel drawn focus heard=Apply limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Apply
                      30 TextBlock at=900,430,55,32 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=54.29,31.2 limit=883,424,89,44 owner=29 paints=#9C9CA0FF role=text size=26 text=APPLY
                  31 ScrollViewer at=309,264,663,148 by=panel fades=b limit=234,124,812.35,548 overflow=54 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=309,264,663,148
                    32 TextBlock at=309,264,663,202 box=663,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=31 layout=657.86,201.6 limit=309,264,663,148 paints=#9C9CA0FF role=text seen=309,264,663,148 size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
        33 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          34 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            35 ActionButton at=274,493,82,44 by=panel drawn focus heard=MAKE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Make
              36 TextBlock at=291,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=47.11,31.2 limit=274,493,82,44 owner=35 paints=#9C9CA0FF role=text size=26 text=MAKE
            37 ActionButton at=368,493,93,44 by=panel drawn focus heard=CHECK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Check
              38 TextBlock at=385,499,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=58.19,31.2 limit=368,493,93,44 owner=37 paints=#9C9CA0FF role=text size=26 text=CHECK
            39 ActionButton at=473,493,62,44 by=panel drawn focus heard=ZIP limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
              40 TextBlock at=490,499,28,32 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=27.35,31.2 limit=473,493,62,44 owner=39 paints=#9C9CA0FF role=text size=26 text=ZIP
            41 ActionButton at=547,493,184,44 by=panel drawn focus heard="CONVERT TO CHD" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
              42 TextBlock at=564,499,150,32 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=149.58,31.2 limit=547,493,184,44 owner=41 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
            43 ActionButton at=743,493,113,44 by=panel drawn focus heard=REFRESH limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
              44 TextBlock at=760,499,79,32 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=743,493,113,44 owner=43 paints=#9C9CA0FF role=text size=26 text=REFRESH
            45 ActionButton at=868,493,82,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              46 TextBlock at=885,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=868,493,82,44 owner=45 paints=#9C9CA0FF role=text size=26 text=MARK
        47 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          48 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            49 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              50 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=340.44,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
            51 StackPanel at=432,586,587,44 by=panel limit=234,124,812.35,548 role=none
              52 Button at=432,586,105,44 by=panel drawn focus heard="Apply the marked changes" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                53 TextBlock at=449,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=432,586,105,44 owner=52 paints=#9C9CA0FF role=text size=26 text="APPLY…"
              54 StackPanel at=549,586,470,44 by=panel limit=234,124,812.35,548 role=none
                55 ProgressBar at=549,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                56 TextBlock at=757,594,262,29 box=262,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=261.82,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="263 KB in packages · 412 GB free"
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
