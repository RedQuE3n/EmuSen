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
                7 StackPanel at=851,52,365,24 by=panel heard="1 changed · 263 KB in packages · 412 GB free" limit=0,0,1280,800 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=851,52,84,24 box=84,24 by=panel caps=13.999 face=Barlow heard="1 changed" layout=83.62,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="1 changed"
                  9 TextBlock at=935,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=948,52,163,24 box=163,24 by=panel caps=13.999 face=Barlow heard="263 KB in packages" layout=162.46,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="263 KB in packages"
                  11 TextBlock at=1111,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1124,52,92,24 box=92,24 by=panel caps=13.999 face=Barlow heard="412 GB free" layout=91.64,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="412 GB free"
                13 StackPanel at=64,44,775,39 by=panel limit=0,0,1280,800 role=none
                  14 TextBlock at=64,44,775,39 box=775,39 by=panel caps=22.3984 face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            15 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              16 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                17 TextBox at=976,100,240,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  18 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    19 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 face=Barlow heard="Search games" in=18 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              20 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                21 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  22 SectionTab at=80,100,34,44 by=panel focus heard=All limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    23 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=22 paints=#EEEEF0FF role=text size=20 text=ALL
                  24 SectionTab at=146,100,36,44 by=panel focus heard=NES limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=NES
                    25 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=24 paints=#D4D4D4FF role=text size=20 text=NES
                  26 SectionTab at=214,100,114,44 by=panel focus heard=PlayStation limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=PlayStation
                    27 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=26 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  28 SectionTab at=360,100,74,44 by=panel focus heard="Sega CD" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Sega CD"
                    29 TextBlock at=360,108,74,24 box=74,24 by=clip caps=13.999 face=Barlow heard="SEGA CD" layout=73.12,24 limit=360,100,74,44 owner=28 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  30 SectionTab at=466,100,47,44 by=panel focus heard=SNES limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=SNES
                    31 TextBlock at=466,108,47,24 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=466,100,47,44 owner=30 paints=#D4D4D4FF role=text size=20 text=SNES
            32 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 role=none
              33 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                34 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  35 LunaTable at=64,160,1152,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    36 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,1152,432
                      37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,54 focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        38 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=36 role=none
                          39 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=36 paints=#0E0E10FF role=none
                            40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,160,1128,53 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=36 role=none
                              41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,170,709,32 box=709,32 caps=18.1987 ellipsis face=Barlow heard=Game in=36 layout=707.02,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,170,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=37 paints=#35A064FF role=edit size=26 text="✓ OK"
                      43 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,54 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        44 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=36 role=none
                          45 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=36 paints=#0E0E10FF role=none
                            46 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,214,1128,53 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=36 role=none
                              47 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,224,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=36 layout=273.65,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,224,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=43 paints=#35A064FF role=edit size=26 text="✓ OK"
                      49 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,54 focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        50 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=36 role=none
                          51 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=36 paints=#0E0E10FF role=none
                            52 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,268,1128,53 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=36 role=none
                              53 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,278,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=36 layout=269.57,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,278,152,34 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=128.28,33.41 owner=49 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      55 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,54 focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=36 paints=#333337FF,#EEEEF0FF role=listitem
                        56 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=36 role=none
                          57 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=36 paints=#26262BFF,#4C9AE8FF role=none
                            58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,322,1128,53 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=36 role=none
                              59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,332,195,34 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ convert to CHD" in=36 layout=194.09,33.41 owner=55 paints=#4C9AE8FF role=text size=26 text="✓ convert to CHD"
                              60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,332,709,32 box=709,32 caps=18.1987 ellipsis face=Barlow heard=Game in=36 layout=701.58,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,332,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=55 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      62 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,54 focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        63 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=36 role=none
                          64 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=36 paints=#0E0E10FF role=none
                            65 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,376,1128,53 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=36 role=none
                              66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,386,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=36 layout=201.32,31.2 owner=62 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,386,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=62 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      68 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,54 focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        69 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=36 role=none
                          70 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=36 paints=#0E0E10FF role=none
                            71 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,430,1128,53 heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged" in=36 role=none
                              72 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,440,195,34 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ make, linked" in=36 layout=168.69,33.41 owner=68 paints=#4C9AE8FF role=text size=26 text="✓ make, linked"
                              73 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=307,440,709,32 box=709,32 caps=18.1987 face=Barlow heard=Game in=36 layout=197.96,31.2 owner=68 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              74 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,440,152,32 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=68 paints=#8D8D91FF role=edit size=26 text="Not packaged"
            75 Border at=48,592,1184,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              76 Grid at=48,610,1184,92 by=panel limit=0,0,1280,800 role=none
                77 Grid at=48,610,1184,31 by=panel limit=0,0,1280,800 role=none
                  78 TextBlock at=48,614,1184,27 box=1184,27 by=panel caps=15.3989 face=Barlow heard="2 marked: make, linked 1, convert to CHD 1" layout=384.41,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 marked: make, linked 1, convert to CHD 1"
                79 StackPanel at=575,658,657,44 by=panel limit=0,0,1280,800 role=none
                  80 Button at=575,658,170,44 by=panel focus heard="Apply the marked changes" limit=0,0,1280,800 paints=#707070FF,#D4D4D4FF role=button
                    81 StackPanel at=592,664,136,32 by=clip limit=575,658,170,44 role=none
                      82 PadGlyph at=592,669,22,22 by=clip heard=Menu limit=575,658,170,44 role=image
                      83 TextBlock at=624,664,83,32 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=575,658,170,44 owner=80 paints=#D4D4D4FF role=text size=26 text="Apply…"
                  84 StackPanel at=757,658,475,44 by=panel limit=0,0,1280,800 role=none
                    85 ProgressBar at=757,673,200,14 by=panel heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    86 TextBlock at=965,668,267,24 box=267,24 by=panel caps=13.999 face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            87 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              88 HintBar at=64,716,1152,30 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          89 Panel at=0,0,1280,702 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            90 Border at=280,138,720,426 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              91 DockPanel at=313,171,654,360 by=panel limit=0,0,1280,800 role=none
                92 TextBlock at=313,171,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                93 StackPanel at=313,419,654,112 by=panel limit=0,0,1280,800 role=none
                  94 Button at=313,419,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    95 TextBlock at=341,430,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,419,654,54 owner=94 paints=#EEEEF0FF role=text size=26 text=No
                  96 Button at=313,477,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    97 TextBlock at=341,488,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,477,654,54 owner=96 paints=#D4D4D4FF role=text size=26 text=Yes
                98 ScrollViewer at=313,222,654,185 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,222,654,185
                  99 StackPanel at=313,222,654,185 by=fixed-scroll in=98 limit=313,222,654,185 role=none
                    100 TextBlock at=313,222,654,185 box=654,185 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=98 layout=633.69,184.8 limit=313,222,654,185 paints=#D4D4D4FF role=text size=22 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
focus 94
edge 94 up same
edge 94 down 96
edge 94 left same
edge 94 right same
edge 96 up 94
edge 96 down same
edge 96 left same
edge 96 right same
press pageup focus="Button  (No)" top=PackagesWindow
press pagedown focus="Button  (No)" top=PackagesWindow
press first focus="Button  (No)" top=PackagesWindow
press last focus="Button  (No)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged)" top=PackagesWindow
