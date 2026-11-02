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
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=991,48,225,24 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=64,40,915,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,915,39 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=64,91,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,92,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,100,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,92,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,92,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=154,100,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,92,68,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,92,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=230,100,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,92,146,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=368,92,106,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=384,100,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=368,92,106,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=482,92,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=498,100,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=482,92,79,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,149,1152,529 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,149,1152,529 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                29 Border at=64,149,1152,529 by=panel limit=0,0,1280,800 role=none
                  30 ScrollViewer at=72,157,1136,513 by=panel fades=t limit=0,0,1280,800 overflow=132 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=72,157,1136,513
                    31 PackageDetail at=72,132,1136,645 by=scroll in=30 limit=72,157,1136,513 name=PackagesDetail role=none seen=72,157,1136,513
                      32 FieldRow at=72,132,1136,62 by=scroll heard=Package in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group seen=72,157,1136,37
                        33 TextBlock at=72,132,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=30 layout=65.81,28.8 limit=72,132,1136,62 name=PART_Label paints=#D4D4D4FF part role=text seen=72,157,1136,4 size=24 text=Package
                        34 TextBlock at=72,165,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=30 layout=915.82,28.8 limit=72,132,1136,62 paints=#D4D4D4FF role=text size=24 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                      35 FieldRow at=72,204,1136,62 by=scroll heard=Game in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group
                        36 TextBlock at=72,204,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Game in=30 layout=44.21,28.8 limit=72,204,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Game
                        37 TextBlock at=72,237,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=977.64,28.8 limit=72,204,1136,62 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      38 FieldRow at=72,276,1136,62 by=scroll heard=State in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group
                        39 TextBlock at=72,276,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=State in=30 layout=39.53,28.8 limit=72,276,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=State
                        40 TextBlock at=72,309,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not packaged" in=30 layout=107.38,28.8 limit=72,276,1136,62 paints=#D4D4D4FF role=text size=24 text="Not packaged"
                      41 FieldRow at=72,348,1136,62 by=scroll heard=Size in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group
                        42 TextBlock at=72,348,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Size in=30 layout=31.06,28.8 limit=72,348,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Size
                        43 TextBlock at=72,381,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The game 658 MB" in=30 layout=136.73,28.8 limit=72,348,1136,62 paints=#D4D4D4FF role=text size=24 text="The game 658 MB"
                      44 FieldRow at=72,420,1136,62 by=scroll heard=Format in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group
                        45 TextBlock at=72,420,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Format in=30 layout=54.29,28.8 limit=72,420,1136,62 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Format
                        46 TextBlock at=72,453,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CUE/BIN, which can be converted to CHD" in=30 layout=312.96,28.8 limit=72,420,1136,62 paints=#D4D4D4FF role=text size=24 text="CUE/BIN, which can be converted to CHD"
                      47 GamePackageDetail at=72,492,1136,285 by=scroll in=30 limit=72,157,1136,513 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=72,492,1136,178
                        48 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        49 FieldRow at=72,556,1136,114 by=scroll heard=Package in=30 limit=72,157,1136,513 paints=#D4D4D4FF role=group
                          50 TextBlock at=72,556,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=30 layout=65.81,28.8 limit=72,556,1136,114 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
                          51 StackPanel at=72,589,1136,81 by=clip heard=Package in=30 limit=72,556,1136,114 role=none
                            52 TextBlock at=72,589,1136,29 box=1136,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=30 layout=144.72,28.8 limit=72,556,1136,114 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
                            53 ButtonBar at=1048,626,160,44 by=clip in=30 limit=72,556,1136,114 paints=#D4D4D4FF role=toolbar
                              54 Button at=1048,626,160,44 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1048,626,160,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                55 TextBlock at=1065,632,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=30 layout=125.29,31.2 limit=1048,626,160,44 owner=54 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            56 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              57 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                58 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  59 TextBlock at=60,682,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                60 StackPanel at=767,719,461,29 by=panel limit=0,0,1280,800 role=none
                  61 StackPanel at=767,719,461,29 by=panel limit=0,0,1280,800 role=none
                    62 ProgressBar at=767,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    63 TextBlock at=975,719,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            64 HintBar at=64,760,1152,24 by=panel heard="Select, Back, Console, List, Apply…, Mark" limit=0,0,1280,800 role=group
focus 54
edge 12 up same
edge 12 down 54
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 54
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 54
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 54
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 54
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 54
edge 25 left 23
edge 25 right 12
edge 54 up 12
edge 54 down same
edge 54 left 25
edge 54 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
