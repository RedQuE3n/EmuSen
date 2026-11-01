# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case PackagesDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 PackagesPane at=48,28,1184,738 by=panel heard=Packages limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 Border at=48,28,1184,51 by=panel heard=PACKAGES limit=0,0,1280,774 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,774 role=none
                7 TextBlock at=991,48,225,24 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1280,774 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=64,40,915,39 by=panel limit=0,0,1280,774 role=none
                  9 TextBlock at=64,40,915,39 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,774 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=64,91,972,46 by=panel limit=0,0,1280,774 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,774 role=none
                  17 ToggleButton at=64,92,65,44 by=panel drawn focus heard=All limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=All
                    18 TextBlock at=81,98,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=64,92,65,44 owner=17 paints=#EEEEF0FF role=text size=26 text=ALL
                  19 ToggleButton at=137,92,67,44 by=panel drawn focus heard=NES limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=NES
                    20 TextBlock at=154,98,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=137,92,67,44 owner=19 paints=#FFFFFFFF role=text size=26 text=NES
                  21 ToggleButton at=212,92,141,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=PlayStation
                    22 TextBlock at=229,98,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=212,92,141,44 owner=21 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  23 ToggleButton at=361,92,104,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="Sega CD"
                    24 TextBlock at=378,98,70,32 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=361,92,104,44 owner=23 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
                  25 ToggleButton at=473,92,77,44 by=panel drawn focus heard=SNES limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=SNES
                    26 TextBlock at=490,98,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=473,92,77,44 owner=25 paints=#FFFFFFFF role=text size=26 text=SNES
            27 Grid at=64,149,1152,490 by=panel limit=0,0,1280,774 role=none
              28 Border at=64,149,1152,490 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
                29 Border at=64,149,1152,490 by=panel limit=0,0,1280,774 role=none
                  30 ScrollViewer at=72,157,1136,474 by=panel fades=t limit=0,0,1280,774 overflow=171 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=72,157,1136,474
                    31 PackageDetail at=72,93,1136,645 by=scroll in=30 limit=72,157,1136,474 name=PackagesDetail paints=#D4D4D4FF role=none seen=72,157,1136,474
                      32 FieldRow at=72,165,1136,62 by=scroll heard=Game in=30 limit=72,157,1136,474 paints=#D4D4D4FF role=group
                        33 TextBlock at=72,165,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=44.21,28.8 limit=72,165,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Game
                        34 TextBlock at=72,198,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=977.64,28.8 limit=72,165,1136,62 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      35 FieldRow at=72,237,1136,62 by=scroll heard=State in=30 limit=72,157,1136,474 paints=#D4D4D4FF role=group
                        36 TextBlock at=72,237,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=39.53,28.8 limit=72,237,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=State
                        37 TextBlock at=72,270,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not packaged" in=30 layout=107.38,28.8 limit=72,237,1136,62 paints=#D4D4D4FF role=text size=24 text="Not packaged"
                      38 FieldRow at=72,309,1136,62 by=scroll heard=Size in=30 limit=72,157,1136,474 paints=#D4D4D4FF role=group
                        39 TextBlock at=72,309,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Size in=30 layout=31.06,28.8 limit=72,309,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Size
                        40 TextBlock at=72,342,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The game 658 MB" in=30 layout=136.73,28.8 limit=72,309,1136,62 paints=#D4D4D4FF role=text size=24 text="The game 658 MB"
                      41 FieldRow at=72,381,1136,62 by=scroll heard=Format in=30 limit=72,157,1136,474 paints=#D4D4D4FF role=group
                        42 TextBlock at=72,381,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Format in=30 layout=54.29,28.8 limit=72,381,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Format
                        43 TextBlock at=72,414,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CUE/BIN, which can be converted to CHD" in=30 layout=312.96,28.8 limit=72,381,1136,62 paints=#D4D4D4FF role=text size=24 text="CUE/BIN, which can be converted to CHD"
                      44 GamePackageDetail at=72,453,1136,285 by=scroll in=30 limit=72,157,1136,474 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=72,453,1136,178
                        45 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        46 FieldRow at=72,517,1136,114 by=scroll heard=Package in=30 limit=72,157,1136,474 paints=#D4D4D4FF role=group
                          47 TextBlock at=72,517,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=30 layout=65.81,28.8 limit=72,517,1136,114 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
                          48 StackPanel at=72,550,1136,81 by=clip heard=Package in=30 limit=72,517,1136,114 role=none
                            49 TextBlock at=72,550,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=30 layout=144.72,28.8 limit=72,517,1136,114 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
                            50 ButtonBar at=1048,587,160,44 by=clip in=30 limit=72,517,1136,114 paints=#D4D4D4FF role=toolbar
                              51 Button at=1048,587,160,44 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1048,587,160,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                52 TextBlock at=1065,593,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=30 layout=125.29,31.2 limit=1048,587,160,44 owner=51 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            53 Border at=48,639,1184,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
              54 Grid at=52,643,1176,81 by=panel limit=0,0,1280,774 role=none
                55 Grid at=52,643,1176,29 by=panel limit=0,0,1280,774 role=none
                  56 TextBlock at=60,643,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                57 StackPanel at=653,680,575,44 by=panel limit=0,0,1280,774 role=none
                  58 Button at=653,680,102,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    59 TextBlock at=670,686,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=653,680,102,44 off owner=58 paints=#707070FF role=text size=26 text="APPLY…"
                  60 StackPanel at=767,680,461,44 by=panel limit=0,0,1280,774 role=none
                    61 ProgressBar at=767,695,200,14 by=panel drawn heard=Space limit=0,0,1280,774 paints=#333337FF,#4C9AE8FF role=progressbar
                    62 TextBlock at=975,688,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            63 HintBar at=64,736,1152,22 by=panel heard="Select, Back, Section, List, Apply…, Mark" limit=0,0,1280,774 role=group
focus 51
edge 12 up same
edge 12 down 51
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 51
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 51
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 51
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 51
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 51
edge 25 left 23
edge 25 right 12
edge 51 up 12
edge 51 down same
edge 51 left 25
edge 51 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" top=PackagesWindow
