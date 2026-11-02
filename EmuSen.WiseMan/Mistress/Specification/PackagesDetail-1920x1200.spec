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
      2 PackagesPane at=72,42,1776,1146 by=panel heard=Packages limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=PACKAGES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1414.5,72,409.5,36 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=96,60,1300.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1300.5,58.5 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,102,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=231,148.5,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=207,136.5,102,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=321,136.5,219,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=345,148.5,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,136.5,219,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=552,136.5,159,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=576,148.5,111,36 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=552,136.5,159,66 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=723,136.5,118.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=747,148.5,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=723,136.5,118.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 role=none
              28 Border at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                29 Border at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 role=none
                  30 ScrollViewer at=108,232.5,1704,774 by=panel fades=t limit=0,0,1920,1200 overflow=229.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=108,232.5,1704,774
                    31 PackageDetail at=108,160.5,1704,1003.5 by=scroll in=30 limit=108,232.5,1704,774 name=PackagesDetail role=none seen=108,232.5,1704,774
                      32 FieldRow at=108,160.5,1704,87 by=scroll heard=Package in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group seen=108,232.5,1704,15
                        33 TextBlock at=108,207,1704,40.5 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=30 layout=1078.37,26.4 limit=108,160.5,1704,87 paints=#D4D4D4FF role=text seen=108,232.5,1704,15 size=22 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                      34 FieldRow at=108,262.5,1704,126 by=scroll heard=Game in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group
                        35 TextBlock at=108,262.5,1704,40.5 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Game in=30 layout=53.77,26.4 limit=108,262.5,1704,126 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Game
                        36 TextBlock at=108,309,1704,79.5 box=1136,53 by=clip caps=15.3989 drawn face=Barlow heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=30 layout=1036.71,52.8 limit=108,262.5,1704,126 paints=#D4D4D4FF role=text size=22 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                      37 FieldRow at=108,403.5,1704,87 by=scroll heard=State in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group
                        38 TextBlock at=108,403.5,1704,40.5 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=State in=30 layout=49.61,26.4 limit=108,403.5,1704,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=State
                        39 TextBlock at=108,450,1704,40.5 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not packaged" in=30 layout=128.52,26.4 limit=108,403.5,1704,87 paints=#D4D4D4FF role=text size=22 text="Not packaged"
                      40 FieldRow at=108,505.5,1704,87 by=scroll heard=Size in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group
                        41 TextBlock at=108,505.5,1704,40.5 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Size in=30 layout=39.36,26.4 limit=108,505.5,1704,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Size
                        42 TextBlock at=108,552,1704,40.5 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="The game 658 MB" in=30 layout=161.77,26.4 limit=108,505.5,1704,87 paints=#D4D4D4FF role=text size=22 text="The game 658 MB"
                      43 FieldRow at=108,607.5,1704,87 by=scroll heard=Format in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group
                        44 TextBlock at=108,607.5,1704,40.5 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Format in=30 layout=68.02,26.4 limit=108,607.5,1704,87 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Format
                        45 TextBlock at=108,654,1704,40.5 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="CUE/BIN, which can be converted to CHD" in=30 layout=376.05,26.4 limit=108,607.5,1704,87 paints=#D4D4D4FF role=text size=22 text="CUE/BIN, which can be converted to CHD"
                      46 GamePackageDetail at=108,709.5,1704,454.5 by=scroll in=30 limit=108,232.5,1704,774 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=108,709.5,1704,297
                        47 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                        48 FieldRow at=108,841.5,1704,165 by=scroll heard=Package in=30 limit=108,232.5,1704,774 paints=#D4D4D4FF role=group
                          49 TextBlock at=108,841.5,1704,40.5 box=1136,27 by=clip caps=16.0864 drawn face="Barlow Medium" heard=Package in=30 layout=81.03,26.4 limit=108,841.5,1704,165 name=PART_Label paints=#D4D4D4FF part role=text size=22 text=Package
                          50 StackPanel at=108,888,1704,118.5 by=clip heard=Package in=30 limit=108,841.5,1704,165 role=none
                            51 TextBlock at=108,888,1704,40.5 box=1136,27 by=clip caps=15.3989 drawn face=Barlow heard="Not a package yet." in=30 layout=171.23,26.4 limit=108,841.5,1704,165 name=GamePackagePlace paints=#D4D4D4FF role=text size=22 text="Not a package yet."
                            52 ButtonBar at=1500,940.5,312,66 by=clip in=30 limit=108,841.5,1704,165 paints=#D4D4D4FF role=toolbar
                              53 Button at=1500,940.5,312,66 by=clip drawn focus heard="MAKE PACKAGE" in=30 limit=1500,940.5,312,66 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                                54 TextBlock at=1525.5,949.5,261,48 box=174,32 by=clip caps=18.1987 drawn face=Barlow heard="MAKE PACKAGE" in=30 layout=173.6,31.2 limit=1500,940.5,312,66 owner=53 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
            55 Border at=72,1018.5,1776,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              56 Grid at=78,1024.5,1764,88.5 by=panel limit=0,0,1920,1200 role=none
                57 Grid at=78,1024.5,1764,40.5 by=panel limit=0,0,1920,1200 role=none
                  58 TextBlock at=90,1024.5,1752,40.5 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                59 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                  60 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                    61 ProgressBar at=1129.5,1084.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    62 TextBlock at=1441.5,1077,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            63 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              64 HintBar at=144,1135.5,1632,43.5 by=panel heard="Select, Back, Console, List, Apply…, Mark" limit=0,0,1920,1200 role=group
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
