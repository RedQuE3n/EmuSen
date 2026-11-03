# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Packages
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
              33 Border at=64,160,1152,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                34 Grid at=64,160,1152,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  35 LunaTable at=64,160,1152,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    36 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,1152,432
                      37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,54 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=36 op paints=#333337FF,#D4D4D4FF role=listitem
                        38 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=36 role=none
                          39 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,160,1152,53 in=36 paints=#0E0E10FF role=none
                            40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,160,1128,53 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=36 role=none
                              41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,170,928,32 box=928,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=36 layout=925.86,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,170,152,34 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=37 paints=#35A064FF role=edit size=26 text="✓ OK"
                      43 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=36 op paints=#333337FF,#D4D4D4FF role=listitem
                        44 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=36 role=none
                          45 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,214,1152,53 in=36 paints=#0E0E10FF role=none
                            46 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,214,1128,53 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=36 role=none
                              47 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,224,928,32 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=36 layout=273.65,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,224,152,34 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=43 paints=#35A064FF role=edit size=26 text="✓ OK"
                      49 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,54 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=36 op paints=#333337FF,#D4D4D4FF role=listitem
                        50 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=36 role=none
                          51 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,268,1152,53 in=36 paints=#0E0E10FF role=none
                            52 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,268,1128,53 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=36 role=none
                              53 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,278,928,32 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=36 layout=269.57,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,278,152,34 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=36 layout=128.28,33.41 owner=49 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      55 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,54 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=36 op paints=#333337FF,#EEEEF0FF role=listitem
                        56 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=36 role=none
                          57 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,322,1152,53 in=36 paints=#26262BFF,#4C9AE8FF role=none
                            58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,322,1128,53 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=36 role=none
                              59 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,332,928,32 box=928,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=36 layout=926.74,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,332,152,32 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=36 layout=151.89,31.2 owner=55 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      61 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,54 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=36 op paints=#333337FF,#D4D4D4FF role=listitem
                        62 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=36 role=none
                          63 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,376,1152,53 in=36 paints=#0E0E10FF role=none
                            64 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,376,1128,53 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=36 role=none
                              65 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,386,928,32 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=36 layout=201.32,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,386,152,32 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=36 layout=151.89,31.2 owner=61 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      67 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=36 op paints=#333337FF,#D4D4D4FF role=listitem
                        68 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=36 role=none
                          69 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,430,1152,53 in=36 paints=#0E0E10FF role=none
                            70 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,430,1128,53 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=36 role=none
                              71 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,440,928,32 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=36 layout=197.96,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              72 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1040,440,152,32 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=36 layout=151.89,31.2 owner=67 paints=#8D8D91FF role=edit size=26 text="Not packaged"
            73 Border at=48,592,1184,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              74 Grid at=48,610,1184,92 by=panel limit=0,0,1280,800 role=none
                75 Grid at=48,610,1184,31 by=panel limit=0,0,1280,800 role=none
                  76 TextBlock at=48,614,1184,27 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                77 StackPanel at=575,658,657,44 by=panel limit=0,0,1280,800 role=none
                  78 Button at=575,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                    79 StackPanel at=592,664,136,32 by=clip limit=575,658,170,44 off role=none
                      80 PadGlyph at=592,669,22,22 by=clip heard=Menu limit=575,658,170,44 off role=image
                      81 TextBlock at=624,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=575,658,170,44 off owner=78 paints=#707070FF role=text size=26 text="Apply…"
                  82 StackPanel at=757,658,475,44 by=panel limit=0,0,1280,800 role=none
                    83 ProgressBar at=757,673,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    84 TextBlock at=965,668,267,24 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            85 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              86 HintBar at=64,716,1152,30 by=panel heard="Open, Back, Console, Mark, List, Apply…" limit=0,0,1280,800 role=group
focus 55
edge 17 up same
edge 17 down 37
edge 17 left 30
edge 17 right same
edge 22 up same
edge 22 down 37
edge 22 left same
edge 22 right 24
edge 24 up same
edge 24 down 37
edge 24 left 22
edge 24 right 26
edge 26 up same
edge 26 down 37
edge 26 left 24
edge 26 right 28
edge 28 up same
edge 28 down 37
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 37
edge 30 left 28
edge 30 right 17
edge 37 up 22
edge 37 down 43
edge 37 left same
edge 37 right same
edge 43 up 37
edge 43 down 49
edge 43 left same
edge 43 right same
edge 49 up 43
edge 49 down 55
edge 49 left same
edge 49 right same
edge 55 up 49
edge 55 down 61
edge 55 left same
edge 55 right same
edge 61 up 55
edge 61 down 67
edge 61 left same
edge 61 right same
edge 67 up 61
edge 67 down same
edge 67 left same
edge 67 right same
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
