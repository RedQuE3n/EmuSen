# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Packages
case PackagesDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 ScrollViewer at=274,148,553,46 by=panel limit=234,124,812.35,548 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=274,148,553,46
              10 StackPanel at=274,148,553,46 by=fixed-scroll in=9 limit=274,148,553,46 role=none
                11 ToggleButton at=274,149,66,44 by=fixed-scroll drawn focus heard=All in=9 limit=274,148,553,46 op paints=#FFFFFFFF role=button text=All
                  12 TextBlock at=291,155,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL in=9 layout=31.25,31.2 limit=274,149,66,44 owner=11 paints=#EEEEF0FF role=text size=26 text=ALL
                13 ToggleButton at=348,149,125,44 by=fixed-scroll drawn focus heard=Packaged in=9 limit=274,148,553,46 op paints=#FFFFFFFF role=button text=Packaged
                  14 TextBlock at=365,155,91,32 box=91,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PACKAGED in=9 layout=90.43,31.2 limit=348,149,125,44 owner=13 paints=#FFFFFFFF role=text size=26 text=PACKAGED
                15 ToggleButton at=481,149,108,44 by=fixed-scroll drawn focus heard="Not yet" in=9 limit=274,148,553,46 op paints=#FFFFFFFF role=button text="Not yet"
                  16 TextBlock at=498,155,74,32 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="NOT YET" in=9 layout=73.5,31.2 limit=481,149,108,44 owner=15 paints=#FFFFFFFF role=text size=26 text="NOT YET"
                17 ToggleButton at=597,149,127,44 by=fixed-scroll drawn focus heard=Problems in=9 limit=274,148,553,46 op paints=#FFFFFFFF role=button text=Problems
                  18 TextBlock at=614,155,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PROBLEMS in=9 layout=92.35,31.2 limit=597,149,127,44 owner=17 paints=#FFFFFFFF role=text size=26 text=PROBLEMS
        19 Grid at=274,206,733,316 by=panel limit=234,124,812.35,548 role=none
          20 Border at=274,206,733,316 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            21 Border at=274,206,733,316 by=panel limit=234,124,812.35,548 role=none
              22 ScrollViewer at=282,214,717,300 by=panel fades=b limit=234,124,812.35,548 overflow=108 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,300
                23 PackageDetail at=282,214,717,408 by=scroll in=22 limit=282,214,717,300 name=PackagesDetail paints=#9C9CA0FF role=none seen=282,214,717,300
                  24 FieldRow at=282,214,717,91 by=scroll heard=Package in=22 limit=282,214,717,300 paints=#9C9CA0FF role=group
                    25 TextBlock at=282,214,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=22 layout=69.48,28.8 limit=282,214,717,91 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
                    26 TextBlock at=282,247,717,58 box=717,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=22 layout=675.02,57.6 limit=282,214,717,91 paints=#9C9CA0FF role=text size=24 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                  27 FieldRow at=282,315,717,91 by=scroll heard=Game in=22 limit=282,214,717,300 paints=#9C9CA0FF role=group
                    28 TextBlock at=282,315,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Game in=22 layout=46.06,28.8 limit=282,315,717,91 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Game
                    29 TextBlock at=282,348,717,58 box=717,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=22 layout=674.42,57.6 limit=282,315,717,91 paints=#9C9CA0FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                  30 FieldRow at=282,416,717,62 by=scroll heard=State in=22 limit=282,214,717,300 paints=#9C9CA0FF role=group
                    31 TextBlock at=282,416,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=State in=22 layout=42.48,28.8 limit=282,416,717,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=State
                    32 TextBlock at=282,449,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not packaged" in=22 layout=112.3,28.8 limit=282,416,717,62 paints=#9C9CA0FF role=text size=24 text="Not packaged"
                  33 FieldRow at=282,488,717,62 by=scroll heard=Size in=22 limit=282,214,717,300 paints=#9C9CA0FF role=group seen=282,488,717,26
                    34 TextBlock at=282,488,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Size in=22 layout=33.41,28.8 limit=282,488,717,62 name=PART_Label paints=#9C9CA0FF part role=text seen=282,488,717,26 size=24 text=Size
        35 Border at=274,530,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          36 ScrollViewer at=274,530,733,44 by=panel limit=234,124,812.35,548 overflow=0 paints=#9C9CA0FF role=scrollviewer vfixed viewport=274,530,733,44
            37 StackPanel at=274,530,733,44 by=fixed-scroll in=36 limit=274,530,733,44 role=none
              38 ActionButton at=274,530,82,44 by=fixed-scroll drawn focus heard=MAKE in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text=Make
                39 TextBlock at=291,536,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE in=36 layout=47.11,31.2 limit=274,530,82,44 owner=38 paints=#9C9CA0FF role=text size=26 text=MAKE
              40 ActionButton at=368,530,93,44 by=fixed-scroll drawn focus heard=CHECK in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text=Check
                41 TextBlock at=385,536,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK in=36 layout=58.19,31.2 limit=368,530,93,44 owner=40 paints=#9C9CA0FF role=text size=26 text=CHECK
              42 ActionButton at=473,530,62,44 by=fixed-scroll drawn focus heard=ZIP in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
                43 TextBlock at=490,536,28,32 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP in=36 layout=27.35,31.2 limit=473,530,62,44 owner=42 paints=#9C9CA0FF role=text size=26 text=ZIP
              44 ActionButton at=547,530,184,44 by=fixed-scroll drawn focus heard="CONVERT TO CHD" in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
                45 TextBlock at=564,536,150,32 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" in=36 layout=149.58,31.2 limit=547,530,184,44 owner=44 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
              46 ActionButton at=743,530,113,44 by=fixed-scroll drawn focus heard=REFRESH in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
                47 TextBlock at=760,536,79,32 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH in=36 layout=78.94,31.2 limit=743,530,113,44 owner=46 paints=#9C9CA0FF role=text size=26 text=REFRESH
              48 ActionButton at=868,530,82,44 by=fixed-scroll drawn focus heard=MARK in=36 limit=274,530,733,44 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
                49 TextBlock at=885,536,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK in=36 layout=47.48,31.2 limit=868,530,82,44 owner=48 paints=#9C9CA0FF role=text size=26 text=MARK
        50 Border at=258,582,765,52 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          51 DockPanel at=262,586,757,44 by=panel limit=234,124,812.35,548 role=none
            52 StackPanel at=507,586,512,44 by=panel limit=234,124,812.35,548 role=none
              53 Button at=507,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                54 TextBlock at=524,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=507,586,105,44 off owner=53 paints=#55555AFF role=text size=26 text="APPLY…"
              55 MeterRow at=624,586,395,44 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#9C9CA0FF role=progressbar
                56 TextBlock at=624,594,140,29 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Space layout=49.58,28.8 limit=624,586,395,44 paints=#B4B4B8FF part role=text size=24 text=Space
                57 ProgressBar at=764,601,200,14 drawn name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                58 TextBlock at=972,594,47,29 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=60% layout=39.24,28.8 limit=624,586,395,44 paints=#B4B4B8FF part role=text size=24 text=60%
            59 StackPanel at=262,586,245,44 by=panel limit=234,124,812.35,548 role=none
              60 TextBlock at=262,594,131,29 box=131,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
        61 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=234,124,812.35,548 role=group
focus 5
edge 5 up same
edge 5 down 46
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
edge 38 down same
edge 38 left same
edge 38 right 40
edge 40 up 13
edge 40 down same
edge 40 left 38
edge 40 right 42
edge 42 up 15
edge 42 down same
edge 42 left 40
edge 42 right 44
edge 44 up 15
edge 44 down same
edge 44 left 42
edge 44 right 46
edge 46 up 5
edge 46 down same
edge 46 left 44
edge 46 right 48
edge 48 up 5
edge 48 down same
edge 48 left 46
edge 48 right same
press pageup focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
