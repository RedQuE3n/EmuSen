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
      2 PackagesPane at=48,28,1184,732 by=panel heard=Packages limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=943,52,273,24 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=64,44,867,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,867,39 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=146,100,36,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,100,114,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=360,100,74,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=360,108,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=360,100,74,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=466,100,47,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=466,108,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=466,100,47,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                29 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
                  30 ScrollViewer at=64,160,1152,432 by=panel fades=t limit=0,0,1280,800 overflow=237 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=64,160,1152,432
                    31 PackageDetail at=64,28,1152,669 by=scroll in=30 limit=64,160,1152,432 name=PackagesDetail paints=#D4D4D4FF role=none seen=64,160,1152,432
                      32 FieldRow at=64,96,1152,84 by=scroll heard=Game in=30 limit=64,160,1152,432 paints=#D4D4D4FF role=group seen=64,160,1152,20
                        33 TextBlock at=64,127,1152,53 box=1152,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=1036.71,52.8 limit=64,96,1152,84 paints=#D4D4D4FF role=text seen=64,160,1152,20 size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      34 FieldRow at=64,190,1152,58 by=scroll heard=State in=30 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        35 TextBlock at=64,190,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=30 layout=49.61,26.4 limit=64,190,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        36 TextBlock at=64,221,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=30 layout=128.52,26.4 limit=64,190,1152,58 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      37 FieldRow at=64,258,1152,58 by=scroll heard=Size in=30 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        38 TextBlock at=64,258,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=30 layout=39.36,26.4 limit=64,258,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        39 TextBlock at=64,289,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=30 layout=161.77,26.4 limit=64,258,1152,58 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      40 FieldRow at=64,326,1152,58 by=scroll heard=Format in=30 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        41 TextBlock at=64,326,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=30 layout=68.02,26.4 limit=64,326,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        42 TextBlock at=64,357,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=30 layout=376.05,26.4 limit=64,326,1152,58 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      43 GamePackageDetail at=64,394,1152,303 by=scroll in=30 limit=64,160,1152,432 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=64,394,1152,198
                        44 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        45 FieldRow at=64,482,1152,110 by=scroll heard=Package in=30 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                          46 TextBlock at=64,482,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=30 layout=81.03,26.4 limit=64,482,1152,110 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          47 StackPanel at=64,513,1152,79 by=clip heard=Package in=30 limit=64,482,1152,110 role=none
                            48 TextBlock at=64,513,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=30 layout=171.23,26.4 limit=64,482,1152,110 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            49 ButtonBar at=1008,548,208,44 by=clip in=30 limit=64,482,1152,110 paints=#D4D4D4FF role=toolbar
                              50 Button at=1008,548,208,44 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1008,548,208,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                51 TextBlock at=1025,554,174,32 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=30 layout=173.6,31.2 limit=1008,548,208,44 owner=50 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            52 Border at=48,592,1184,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              53 Grid at=48,610,1184,92 by=panel limit=0,0,1280,800 role=none
                54 Grid at=48,610,1184,31 by=panel limit=0,0,1280,800 role=none
                  55 TextBlock at=48,614,1184,27 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                56 StackPanel at=575,658,657,44 by=panel limit=0,0,1280,800 role=none
                  57 Button at=575,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                    58 StackPanel at=592,664,136,32 by=clip limit=575,658,170,44 off role=none
                      59 PadGlyph at=592,669,22,22 by=clip heard=Menu limit=575,658,170,44 off role=image
                      60 TextBlock at=624,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=575,658,170,44 off owner=57 paints=#707070FF role=text size=26 text="Apply…"
                  61 StackPanel at=757,658,475,44 by=panel limit=0,0,1280,800 role=none
                    62 ProgressBar at=757,673,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    63 TextBlock at=965,668,267,24 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            64 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              65 HintBar at=64,716,1152,30 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1280,800 role=group
focus 50
edge 12 up same
edge 12 down 50
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 50
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 50
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 50
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 50
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 50
edge 25 left 23
edge 25 right 12
edge 50 up 12
edge 50 down same
edge 50 left 25
edge 50 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
