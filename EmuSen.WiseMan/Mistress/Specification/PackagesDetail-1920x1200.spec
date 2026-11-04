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
                7 StackPanel at=1276.5,78,547.5,36 by=panel heard="1 changed · 263 KB in packages · 412 GB free" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=1276.5,78,126,36 box=84,24 by=panel caps=13.999 drawn face=Barlow heard="1 changed" layout=83.62,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="1 changed"
                  9 TextBlock at=1402.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=1422,78,244.5,36 box=163,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages" layout=162.46,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="263 KB in packages"
                  11 TextBlock at=1666.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1686,78,138,36 box=92,24 by=panel caps=13.999 drawn face=Barlow heard="412 GB free" layout=91.64,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="412 GB free"
                13 StackPanel at=96,66,1162.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  14 TextBlock at=96,66,1162.5,58.5 box=775,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            15 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              16 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                17 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  18 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    19 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=18 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              20 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                21 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  22 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    23 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=22 paints=#EEEEF0FF role=text size=20 text=ALL
                  24 SectionTab at=219,150,54,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=NES
                    25 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=24 paints=#D4D4D4FF role=text size=20 text=NES
                  26 SectionTab at=321,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    27 TextBlock at=321,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,150,171,66 owner=26 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  28 SectionTab at=540,150,111,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Sega CD"
                    29 TextBlock at=540,162,111,36 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=540,150,111,66 owner=28 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  30 SectionTab at=699,150,70.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=SNES
                    31 TextBlock at=699,162,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=699,150,70.5,66 owner=30 paints=#D4D4D4FF role=text size=20 text=SNES
            32 Grid at=96,240,1728,648 by=panel limit=0,0,1920,1200 role=none
              33 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                34 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 role=none
                  35 ScrollViewer at=96,240,1728,648 by=panel fades=t limit=0,0,1920,1200 overflow=355.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=96,240,1728,648
                    36 PackageDetail at=96,42,1728,1003.5 by=scroll in=35 limit=96,240,1728,648 name=PackagesDetail paints=#D4D4D4FF role=none seen=96,240,1728,648
                      37 FieldRow at=96,144,1728,126 by=scroll heard=Game in=35 limit=96,240,1728,648 paints=#D4D4D4FF role=group seen=96,240,1728,30
                        38 TextBlock at=96,190.5,1728,79.5 box=1152,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=35 layout=1036.71,52.8 limit=96,144,1728,126 paints=#D4D4D4FF role=text seen=96,240,1728,30 size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      39 FieldRow at=96,285,1728,87 by=scroll heard=State in=35 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        40 TextBlock at=96,285,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=35 layout=49.61,26.4 limit=96,285,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        41 TextBlock at=96,331.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=35 layout=128.52,26.4 limit=96,285,1728,87 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      42 FieldRow at=96,387,1728,87 by=scroll heard=Size in=35 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        43 TextBlock at=96,387,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=35 layout=39.36,26.4 limit=96,387,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        44 TextBlock at=96,433.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=35 layout=161.77,26.4 limit=96,387,1728,87 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      45 FieldRow at=96,489,1728,87 by=scroll heard=Format in=35 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                        46 TextBlock at=96,489,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=35 layout=68.02,26.4 limit=96,489,1728,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        47 TextBlock at=96,535.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=35 layout=376.05,26.4 limit=96,489,1728,87 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      48 GamePackageDetail at=96,591,1728,454.5 by=scroll in=35 limit=96,240,1728,648 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=96,591,1728,297
                        49 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        50 FieldRow at=96,723,1728,165 by=scroll heard=Package in=35 limit=96,240,1728,648 paints=#D4D4D4FF role=group
                          51 TextBlock at=96,723,1728,40.5 box=1152,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=35 layout=81.03,26.4 limit=96,723,1728,165 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          52 StackPanel at=96,769.5,1728,118.5 by=clip heard=Package in=35 limit=96,723,1728,165 role=none
                            53 TextBlock at=96,769.5,1728,40.5 box=1152,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=35 layout=171.23,26.4 limit=96,723,1728,165 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            54 ButtonBar at=1512,822,312,66 by=clip in=35 limit=96,723,1728,165 paints=#D4D4D4FF role=toolbar
                              55 Button at=1512,822,312,66 by=clip drawn focus heard="MAKE PACKAGE" in=35 limit=1512,822,312,66 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                56 TextBlock at=1537.5,831,261,48 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=35 layout=173.6,31.2 limit=1512,822,312,66 owner=55 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            57 Border at=72,888,1776,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              58 Grid at=72,915,1776,138 by=panel limit=0,0,1920,1200 role=none
                59 Grid at=72,915,1776,46.5 by=panel limit=0,0,1920,1200 role=none
                  60 TextBlock at=72,921,1776,40.5 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                61 StackPanel at=862.5,987,985.5,66 by=panel limit=0,0,1920,1200 role=none
                  62 Button at=862.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                    63 StackPanel at=888,996,204,48 by=clip limit=862.5,987,255,66 off role=none
                      64 PadGlyph at=888,1003.5,33,33 by=clip heard=Menu limit=862.5,987,255,66 off role=image
                      65 TextBlock at=936,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=862.5,987,255,66 off owner=62 paints=#707070FF role=text size=26 text="Apply…"
                  66 StackPanel at=1135.5,987,712.5,66 by=panel limit=0,0,1920,1200 role=none
                    67 ProgressBar at=1135.5,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    68 TextBlock at=1447.5,1002,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            69 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              70 HintBar at=96,1074,1728,45 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1920,1200 role=group
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
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK, Size: 1.0 MB)" top=PackagesWindow
