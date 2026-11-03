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
            5 AdaptiveHeaderRow at=48,28,1184,60 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 StackPanel at=851,52,365,24 by=panel heard="1 changed · 263 KB in packages · 412 GB free" limit=0,0,1280,800 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=851,52,84,24 box=84,24 by=panel caps=13.999 drawn face=Barlow heard="1 changed" layout=83.62,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="1 changed"
                  9 TextBlock at=935,52,13,24 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=948,52,163,24 box=163,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages" layout=162.46,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="263 KB in packages"
                  11 TextBlock at=1111,52,13,24 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1124,52,92,24 box=92,24 by=panel caps=13.999 drawn face=Barlow heard="412 GB free" layout=91.64,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="412 GB free"
                13 StackPanel at=64,44,775,39 by=panel limit=0,0,1280,800 role=none
                  14 TextBlock at=64,44,775,39 box=775,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            15 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              16 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                17 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  18 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    19 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=18 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              20 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                21 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  22 SectionTab at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    23 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=22 paints=#EEEEF0FF role=text size=20 text=ALL
                  24 SectionTab at=146,100,36,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=NES
                    25 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=24 paints=#D4D4D4FF role=text size=20 text=NES
                  26 SectionTab at=214,100,114,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    27 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=26 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  28 SectionTab at=360,100,74,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="Sega CD"
                    29 TextBlock at=360,108,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=360,100,74,44 owner=28 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  30 SectionTab at=466,100,47,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=SNES
                    31 TextBlock at=466,108,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=466,100,47,44 owner=30 paints=#D4D4D4FF role=text size=20 text=SNES
            32 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
              33 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                34 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
                  35 ScrollViewer at=64,160,1152,432 by=panel fades=t limit=0,0,1280,800 overflow=237 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=64,160,1152,432
                    36 PackageDetail at=64,28,1152,669 by=scroll in=35 limit=64,160,1152,432 name=PackagesDetail paints=#D4D4D4FF role=none seen=64,160,1152,432
                      37 FieldRow at=64,96,1152,84 by=scroll heard=Game in=35 limit=64,160,1152,432 paints=#D4D4D4FF role=group seen=64,160,1152,20
                        38 TextBlock at=64,127,1152,53 box=1152,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=35 layout=1036.71,52.8 limit=64,96,1152,84 paints=#D4D4D4FF role=text seen=64,160,1152,20 size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      39 FieldRow at=64,190,1152,58 by=scroll heard=State in=35 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        40 TextBlock at=64,190,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=35 layout=49.61,26.4 limit=64,190,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        41 TextBlock at=64,221,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=35 layout=128.52,26.4 limit=64,190,1152,58 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      42 FieldRow at=64,258,1152,58 by=scroll heard=Size in=35 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        43 TextBlock at=64,258,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=35 layout=39.36,26.4 limit=64,258,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        44 TextBlock at=64,289,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=35 layout=161.77,26.4 limit=64,258,1152,58 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      45 FieldRow at=64,326,1152,58 by=scroll heard=Format in=35 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                        46 TextBlock at=64,326,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=35 layout=68.02,26.4 limit=64,326,1152,58 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        47 TextBlock at=64,357,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=35 layout=376.05,26.4 limit=64,326,1152,58 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      48 GamePackageDetail at=64,394,1152,303 by=scroll in=35 limit=64,160,1152,432 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=64,394,1152,198
                        49 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        50 FieldRow at=64,482,1152,110 by=scroll heard=Package in=35 limit=64,160,1152,432 paints=#D4D4D4FF role=group
                          51 TextBlock at=64,482,1152,27 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=35 layout=81.03,26.4 limit=64,482,1152,110 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          52 StackPanel at=64,513,1152,79 by=clip heard=Package in=35 limit=64,482,1152,110 role=none
                            53 TextBlock at=64,513,1152,27 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=35 layout=171.23,26.4 limit=64,482,1152,110 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            54 ButtonBar at=1008,548,208,44 by=clip in=35 limit=64,482,1152,110 paints=#D4D4D4FF role=toolbar
                              55 Button at=1008,548,208,44 by=clip drawn focus heard="MAKE PACKAGE" in=35 limit=1008,548,208,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                56 TextBlock at=1025,554,174,32 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=35 layout=173.6,31.2 limit=1008,548,208,44 owner=55 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            57 Border at=48,592,1184,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              58 Grid at=48,610,1184,92 by=panel limit=0,0,1280,800 role=none
                59 Grid at=48,610,1184,31 by=panel limit=0,0,1280,800 role=none
                  60 TextBlock at=48,614,1184,27 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                61 StackPanel at=575,658,657,44 by=panel limit=0,0,1280,800 role=none
                  62 Button at=575,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                    63 StackPanel at=592,664,136,32 by=clip limit=575,658,170,44 off role=none
                      64 PadGlyph at=592,669,22,22 by=clip heard=Menu limit=575,658,170,44 off role=image
                      65 TextBlock at=624,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=575,658,170,44 off owner=62 paints=#707070FF role=text size=26 text="Apply…"
                  66 StackPanel at=757,658,475,44 by=panel limit=0,0,1280,800 role=none
                    67 ProgressBar at=757,673,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    68 TextBlock at=965,668,267,24 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            69 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              70 HintBar at=64,716,1152,30 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1280,800 role=group
focus 55
edge 17 up same
edge 17 down 55
edge 17 left 30
edge 17 right same
edge 22 up same
edge 22 down 55
edge 22 left same
edge 22 right 24
edge 24 up same
edge 24 down 55
edge 24 left 22
edge 24 right 26
edge 26 up same
edge 26 down 55
edge 26 left 24
edge 26 right 28
edge 28 up same
edge 28 down 55
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 55
edge 30 left 28
edge 30 right 17
edge 55 up 17
edge 55 down same
edge 55 left 30
edge 55 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
