# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PackagesPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 PackagesPane at=48,28,1184,732 by=panel heard=Packages limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 AdaptiveHeaderRow at=48,28,1184,60 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=943,52,273,24 box=273,24 by=panel caps=13.999 face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=64,44,867,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,867,39 box=867,39 by=panel caps=22.3984 face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  17 SectionTab at=80,100,34,44 by=panel focus heard=All limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=146,100,36,44 by=panel focus heard=NES limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=NES
                    20 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 SectionTab at=214,100,114,44 by=panel focus heard=PlayStation limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=PlayStation
                    22 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 SectionTab at=360,100,74,44 by=panel focus heard="Sega CD" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Sega CD"
                    24 TextBlock at=360,108,74,24 box=74,24 by=clip caps=13.999 face=Barlow heard="SEGA CD" layout=73.12,24 limit=360,100,74,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 SectionTab at=466,100,47,44 by=panel focus heard=SNES limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=SNES
                    26 TextBlock at=466,108,47,24 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=466,100,47,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                29 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  30 LunaTable at=64,160,1152,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,1152,432
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,54 focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 paints=#333337FF,#D4D4D4FF role=listitem
                        33 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=31 role=none
                          34 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=31 paints=#0E0E10FF role=none
                            35 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,160,1128,53 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 role=none
                              36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,170,709,32 box=709,32 caps=18.1987 ellipsis face=Barlow heard=Game in=31 layout=707.02,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              37 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,170,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      38 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,54 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 paints=#333337FF,#D4D4D4FF role=listitem
                        39 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=31 role=none
                          40 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=31 paints=#0E0E10FF role=none
                            41 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,214,1128,53 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 role=none
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,224,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=31 layout=273.65,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,224,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=38 paints=#35A064FF role=edit size=26 text="✓ OK"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,54 focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 paints=#333337FF,#D4D4D4FF role=listitem
                        45 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=31 role=none
                          46 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=31 paints=#0E0E10FF role=none
                            47 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,268,1128,53 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 role=none
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,278,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=31 layout=269.57,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,278,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=31 layout=128.28,33.41 owner=44 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      50 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,54 focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 paints=#333337FF,#EEEEF0FF role=listitem
                        51 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=31 role=none
                          52 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=31 paints=#26262BFF,#4C9AE8FF role=none
                            53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,322,1128,53 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                              54 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,332,195,34 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ convert to CHD" in=31 layout=194.09,33.41 owner=50 paints=#4C9AE8FF role=text size=26 text="✓ convert to CHD"
                              55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,332,709,32 box=709,32 caps=18.1987 ellipsis face=Barlow heard=Game in=31 layout=701.58,31.2 owner=50 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,332,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=31 layout=151.89,31.2 owner=50 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      57 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,54 focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 paints=#333337FF,#D4D4D4FF role=listitem
                        58 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=31 role=none
                          59 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=31 paints=#0E0E10FF role=none
                            60 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,376,1128,53 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                              61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,386,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=31 layout=201.32,31.2 owner=57 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              62 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,386,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=31 layout=151.89,31.2 owner=57 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      63 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,54 focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged" in=31 paints=#333337FF,#D4D4D4FF role=listitem
                        64 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=31 role=none
                          65 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=31 paints=#0E0E10FF role=none
                            66 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,430,1128,53 heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged" in=31 role=none
                              67 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,440,195,34 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ make, linked" in=31 layout=168.69,33.41 owner=63 paints=#4C9AE8FF role=text size=26 text="✓ make, linked"
                              68 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,440,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=31 layout=197.96,31.2 owner=63 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              69 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,440,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=31 layout=151.89,31.2 owner=63 paints=#8D8D91FF role=edit size=26 text="Not packaged"
            70 Border at=48,592,1184,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              71 Grid at=48,610,1184,92 by=panel limit=0,0,1280,800 role=none
                72 Grid at=48,610,1184,31 by=panel limit=0,0,1280,800 role=none
                  73 TextBlock at=48,614,1184,27 box=1184,27 by=panel caps=15.3989 face=Barlow heard="2 marked: make, linked 1, convert to CHD 1" layout=384.41,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 marked: make, linked 1, convert to CHD 1"
                74 StackPanel at=575,658,657,44 by=panel limit=0,0,1280,800 role=none
                  75 Button at=575,658,170,44 by=panel focus heard="Apply the marked changes" limit=0,0,1280,800 paints=#707070FF,#D4D4D4FF role=button
                    76 StackPanel at=592,664,136,32 by=clip limit=575,658,170,44 role=none
                      77 PadGlyph at=592,669,22,22 by=clip heard=Menu limit=575,658,170,44 role=image
                      78 TextBlock at=624,664,83,32 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=575,658,170,44 owner=75 paints=#D4D4D4FF role=text size=26 text="Apply…"
                  79 StackPanel at=757,658,475,44 by=panel limit=0,0,1280,800 role=none
                    80 ProgressBar at=757,673,200,14 by=panel heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    81 TextBlock at=965,668,267,24 box=267,24 by=panel caps=13.999 face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            82 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              83 HintBar at=64,716,1152,30 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          84 Panel at=0,0,1280,702 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            85 Border at=280,138,720,426 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              86 DockPanel at=313,171,654,360 by=panel limit=0,0,1280,800 role=none
                87 TextBlock at=313,171,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                88 StackPanel at=313,419,654,112 by=panel limit=0,0,1280,800 role=none
                  89 Button at=313,419,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    90 TextBlock at=341,430,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,419,654,54 owner=89 paints=#EEEEF0FF role=text size=26 text=No
                  91 Button at=313,477,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    92 TextBlock at=341,488,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,477,654,54 owner=91 paints=#D4D4D4FF role=text size=26 text=Yes
                93 ScrollViewer at=313,222,654,185 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,222,654,185
                  94 StackPanel at=313,222,654,185 by=fixed-scroll in=93 limit=313,222,654,185 role=none
                    95 TextBlock at=313,222,654,185 box=654,185 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=93 layout=633.69,184.8 limit=313,222,654,185 paints=#D4D4D4FF role=text size=22 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
focus 89
edge 89 up same
edge 89 down 91
edge 89 left same
edge 89 right same
edge 91 up 89
edge 91 down same
edge 91 left same
edge 91 right same
press pageup focus="Button  (No)" top=PackagesWindow
press pagedown focus="Button  (No)" top=PackagesWindow
press first focus="Button  (No)" top=PackagesWindow
press last focus="Button  (No)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged)" top=PackagesWindow
