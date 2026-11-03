# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PackagesDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 PackagesPane at=72,42,1776,1098 by=panel heard=Packages limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard=PACKAGES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1414.5,78,409.5,36 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=96,66,1300.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1300.5,58.5 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  17 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=219,150,54,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=NES
                    20 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 SectionTab at=321,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    22 TextBlock at=321,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,150,171,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 SectionTab at=540,150,111,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Sega CD"
                    24 TextBlock at=540,162,111,36 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=540,150,111,66 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 SectionTab at=699,150,70.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=SNES
                    26 TextBlock at=699,162,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=699,150,70.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=96,240,1728,648 by=panel limit=0,0,1920,1200 role=none
              28 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                29 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 role=none
                  30 ScrollViewer at=96,240,1728,648 by=panel fades=t limit=0,0,1920,1200 overflow=355.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=96,240,1728,648
                    31 PackageDetail at=96,42,1728,1003.5 by=scroll in=30 limit=96,240,1728,648 name=PackagesDetail paints=#D4D4D4FF role=none seen=96,240,1728,648
                      32 FieldRow at=96,144,1728,126 by=scroll heard=Game in=30 limit=96,240,1728,648 paints=#D4D4D4FF role=group seen=96,240,1728,30
                        33 TextBlock at=96,190.5,1728,79.5 box=1152,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=1036.71,52.8 limit=96,144,1728,126 paints=#D4D4D4FF role=text seen=96,240,1728,30 size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      34 FieldRow at=96,285,1728,87 by=scroll heard=State in=30 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        35 TextBlock at=96,285,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=30 layout=49.61,26.4 limit=96,285,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        36 TextBlock at=96,331.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=30 layout=128.52,26.4 limit=96,285,1728,87 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      37 FieldRow at=96,387,1728,87 by=scroll heard=Size in=30 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        38 TextBlock at=96,387,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=30 layout=39.36,26.4 limit=96,387,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        39 TextBlock at=96,433.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=30 layout=161.77,26.4 limit=96,387,1728,87 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      40 FieldRow at=96,489,1728,87 by=scroll heard=Format in=30 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        41 TextBlock at=96,489,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=30 layout=68.02,26.4 limit=96,489,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        42 TextBlock at=96,535.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=30 layout=376.05,26.4 limit=96,489,1728,87 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      43 GamePackageDetail at=96,591,1728,454.5 by=scroll in=30 limit=96,240,1728,648 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=96,591,1728,297
                        44 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        45 FieldRow at=96,723,1728,165 by=scroll heard=Package in=30 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                          46 TextBlock at=96,723,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=30 layout=81.03,26.4 limit=96,723,1728,165 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          47 StackPanel at=96,769.5,1728,118.5 by=clip heard=Package in=30 limit=96,723,1728,165 role=none
                            48 TextBlock at=96,769.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=30 layout=171.23,26.4 limit=96,723,1728,165 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            49 ButtonBar at=1512,822,312,66 by=clip in=30 limit=96,723,1728,165 paints=#D4D4D4FF role=toolbar
                              50 Button at=1512,822,312,66 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1512,822,312,66 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                51 TextBlock at=1537.5,831,261,48 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=30 layout=173.6,31.2 limit=1512,822,312,66 owner=50 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            52 Border at=72,888,1776,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              53 Grid at=72,915,1776,138 by=panel limit=0,0,1920,1200 role=none
                54 Grid at=72,915,1776,46.5 by=panel limit=0,0,1920,1200 role=none
                  55 TextBlock at=72,921,1776,40.5 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                56 StackPanel at=862.5,987,985.5,66 by=panel limit=0,0,1920,1200 role=none
                  57 Button at=862.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                    58 StackPanel at=888,996,204,48 by=clip limit=862.5,987,255,66 off role=none
                      59 PadGlyph at=888,1003.5,33,33 by=clip heard=Menu limit=862.5,987,255,66 off role=image
                      60 TextBlock at=936,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=862.5,987,255,66 off owner=57 paints=#707070FF role=text size=26 text="Apply…"
                  61 StackPanel at=1135.5,987,712.5,66 by=panel limit=0,0,1920,1200 role=none
                    62 ProgressBar at=1135.5,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    63 TextBlock at=1447.5,1002,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            64 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              65 HintBar at=96,1074,1728,45 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1920,1200 role=group
focus 50
edge 12 up same
edge 12 down 50
edge 12 left 25
edge 12 right same
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
