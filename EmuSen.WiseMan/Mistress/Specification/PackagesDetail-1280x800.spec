# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PackagesDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 PackagesPane at=48,28,1184,764 by=panel heard=Packages limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=943,48,273,24 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=64,40,867,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,867,39 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=154,99,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,91,68,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,91,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=230,99,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,91,146,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=368,91,106,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=384,99,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=368,91,106,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=482,91,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=498,99,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=482,91,79,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,147,1152,532 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,147,1152,532 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                29 Border at=64,147,1152,532 by=panel limit=0,0,1280,800 role=none
                  30 ScrollViewer at=72,155,1136,516 by=panel fades=t limit=0,0,1280,800 overflow=153 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=72,155,1136,516
                    31 PackageDetail at=72,107,1136,669 by=scroll in=30 limit=72,155,1136,516 name=PackagesDetail role=none seen=72,155,1136,516
                      32 FieldRow at=72,107,1136,58 by=scroll heard=Package in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group seen=72,155,1136,10
                        33 TextBlock at=72,138,1136,27 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=30 layout=1078.37,26.4 limit=72,107,1136,58 paints=#D4D4D4FF role=text seen=72,155,1136,10 size=22 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                      34 FieldRow at=72,175,1136,84 by=scroll heard=Game in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group
                        35 TextBlock at=72,175,1136,27 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Game in=30 layout=53.77,26.4 limit=72,175,1136,84 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Game
                        36 TextBlock at=72,206,1136,53 box=1136,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=1036.71,52.8 limit=72,175,1136,84 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      37 FieldRow at=72,269,1136,58 by=scroll heard=State in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group
                        38 TextBlock at=72,269,1136,27 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=30 layout=49.61,26.4 limit=72,269,1136,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        39 TextBlock at=72,300,1136,27 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=30 layout=128.52,26.4 limit=72,269,1136,58 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      40 FieldRow at=72,337,1136,58 by=scroll heard=Size in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group
                        41 TextBlock at=72,337,1136,27 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=30 layout=39.36,26.4 limit=72,337,1136,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        42 TextBlock at=72,368,1136,27 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=30 layout=161.77,26.4 limit=72,337,1136,58 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      43 FieldRow at=72,405,1136,58 by=scroll heard=Format in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group
                        44 TextBlock at=72,405,1136,27 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=30 layout=68.02,26.4 limit=72,405,1136,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        45 TextBlock at=72,436,1136,27 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=30 layout=376.05,26.4 limit=72,405,1136,58 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      46 GamePackageDetail at=72,473,1136,303 by=scroll in=30 limit=72,155,1136,516 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=72,473,1136,198
                        47 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        48 FieldRow at=72,561,1136,110 by=scroll heard=Package in=30 limit=72,155,1136,516 paints=#D4D4D4FF role=group
                          49 TextBlock at=72,561,1136,27 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=30 layout=81.03,26.4 limit=72,561,1136,110 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          50 StackPanel at=72,592,1136,79 by=clip heard=Package in=30 limit=72,561,1136,110 role=none
                            51 TextBlock at=72,592,1136,27 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=30 layout=171.23,26.4 limit=72,561,1136,110 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            52 ButtonBar at=1000,627,208,44 by=clip in=30 limit=72,561,1136,110 paints=#D4D4D4FF role=toolbar
                              53 Button at=1000,627,208,44 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1000,627,208,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                54 TextBlock at=1017,633,174,32 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=30 layout=173.6,31.2 limit=1000,627,208,44 owner=53 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            55 Border at=48,679,1184,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              56 Grid at=52,683,1176,59 by=panel limit=0,0,1280,800 role=none
                57 Grid at=52,683,1176,27 by=panel limit=0,0,1280,800 role=none
                  58 TextBlock at=60,683,1168,27 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                59 StackPanel at=753,718,475,24 by=panel limit=0,0,1280,800 role=none
                  60 StackPanel at=753,718,475,24 by=panel limit=0,0,1280,800 role=none
                    61 ProgressBar at=753,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    62 TextBlock at=961,718,267,24 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            63 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              64 HintBar at=96,757,1088,29 by=panel heard="Select, Back, Console, List, Apply…, Mark" limit=0,0,1280,800 role=group
focus 53
edge 12 up same
edge 12 down 53
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 53
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 53
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 53
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 53
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 53
edge 25 left 23
edge 25 right 12
edge 53 up 12
edge 53 down same
edge 53 left 25
edge 53 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
