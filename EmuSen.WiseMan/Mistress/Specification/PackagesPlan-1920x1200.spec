# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PackagesPlan
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
                  8 TextBlock at=1276.5,78,126,36 box=84,24 by=panel caps=13.999 face=Barlow heard="1 changed" layout=83.62,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="1 changed"
                  9 TextBlock at=1402.5,78,19.5,36 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=1422,78,244.5,36 box=163,24 by=panel caps=13.999 face=Barlow heard="263 KB in packages" layout=162.46,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="263 KB in packages"
                  11 TextBlock at=1666.5,78,19.5,36 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1686,78,138,36 box=92,24 by=panel caps=13.999 face=Barlow heard="412 GB free" layout=91.64,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="412 GB free"
                13 StackPanel at=96,66,1162.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  14 TextBlock at=96,66,1162.5,58.5 box=775,39 by=panel caps=22.3984 face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            15 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              16 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                17 TextBox at=1464,150,360,66 by=panel focus heard=Search limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  18 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    19 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 face=Barlow heard="Search games" in=18 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              20 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                21 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  22 SectionTab at=120,150,51,66 by=panel focus heard=All limit=0,0,1920,1200 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    23 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=22 paints=#EEEEF0FF role=text size=20 text=ALL
                  24 SectionTab at=219,150,54,66 by=panel focus heard=NES limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text=NES
                    25 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=24 paints=#D4D4D4FF role=text size=20 text=NES
                  26 SectionTab at=321,150,171,66 by=panel focus heard=PlayStation limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text=PlayStation
                    27 TextBlock at=321,162,171,36 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,150,171,66 owner=26 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  28 SectionTab at=540,150,111,66 by=panel focus heard="Sega CD" limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text="Sega CD"
                    29 TextBlock at=540,162,111,36 box=74,24 by=clip caps=13.999 face=Barlow heard="SEGA CD" layout=73.12,24 limit=540,150,111,66 owner=28 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  30 SectionTab at=699,150,70.5,66 by=panel focus heard=SNES limit=0,0,1920,1200 paints=#D4D4D4FF role=tabitem text=SNES
                    31 TextBlock at=699,162,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=699,150,70.5,66 owner=30 paints=#D4D4D4FF role=text size=20 text=SNES
            32 Grid at=96,240,1728,648 by=panel limit=0,0,1920,1200 role=none
              33 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                34 Grid at=96,240,1728,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  35 LunaTable at=96,240,1728,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    36 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1728,648
                      37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,81 focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK, Size: 1.0 MB" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        38 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=36 role=none
                          39 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=36 paints=#0E0E10FF role=none
                            40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,240,1692,79.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK, Size: 1.0 MB" in=36 role=none
                              41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,255,909,48 box=606,32 caps=18.1987 ellipsis face=Barlow heard=Game in=36 layout=598.68,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,255,228,51 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=37 paints=#35A064FF role=edit size=26 text="✓ OK"
                              43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,255,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=68.22,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="1.0 MB"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK, Size: 256 KB" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        45 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=36 role=none
                          46 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=36 paints=#0E0E10FF role=none
                            47 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,321,1692,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK, Size: 256 KB" in=36 role=none
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,336,909,48 box=606,32 caps=18.1987 face=Barlow heard=Game in=36 layout=273.65,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,336,228,51 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=61.96,33.41 owner=44 paints=#35A064FF role=edit size=26 text="✓ OK"
                              50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,336,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=75.48,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="256 KB"
                      51 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,81 focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed, Size: 658 MB" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        52 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=36 role=none
                          53 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=36 paints=#0E0E10FF role=none
                            54 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,402,1692,79.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed, Size: 658 MB" in=36 role=none
                              55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,417,909,48 box=606,32 caps=18.1987 face=Barlow heard=Game in=36 layout=269.57,31.2 owner=51 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,417,228,51 box=152,34 caps=19.1128 face="Adwaita Mono" heard=State in=36 layout=128.28,33.41 owner=51 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                              57 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,417,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=78.49,31.2 owner=51 paints=#D4D4D4FF role=edit size=26 text="658 MB"
                      58 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,81 focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged, Size: 658 MB" in=36 paints=#333337FF,#EEEEF0FF role=listitem
                        59 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=36 role=none
                          60 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=36 paints=#26262BFF,#4C9AE8FF role=none
                            61 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,483,1692,79.5 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged, Size: 658 MB" in=36 role=none
                              62 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,498,292.5,51 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ convert to CHD" in=36 layout=194.09,33.41 owner=58 paints=#4C9AE8FF role=text size=26 text="✓ convert to CHD"
                              63 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,498,909,48 box=606,32 caps=18.1987 ellipsis face=Barlow heard=Game in=36 layout=601.74,31.2 owner=58 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              64 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,498,228,48 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=58 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                              65 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,498,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=78.49,31.2 owner=58 paints=#EEEEF0FF role=edit size=26 text="658 MB"
                      66 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,81 focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged, Size: 534 MB" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        67 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=36 role=none
                          68 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=36 paints=#0E0E10FF role=none
                            69 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,564,1692,79.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged, Size: 534 MB" in=36 role=none
                              70 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,579,909,48 box=606,32 caps=18.1987 face=Barlow heard=Game in=36 layout=201.32,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              71 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,579,228,48 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=66 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                              72 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,579,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=78.16,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="534 MB"
                      73 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,81 focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged, Size: 2.0 MB" in=36 paints=#333337FF,#D4D4D4FF role=listitem
                        74 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=36 role=none
                          75 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=36 paints=#0E0E10FF role=none
                            76 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,645,1692,79.5 heard="Mark: make, linked, Game: Amber Saga (USA), State: Not packaged, Size: 2.0 MB" in=36 role=none
                              77 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,660,292.5,51 box=195,34 caps=19.1128 face="Adwaita Mono" heard="✓ make, linked" in=36 layout=168.69,33.41 owner=73 paints=#4C9AE8FF role=text size=26 text="✓ make, linked"
                              78 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=460.5,660,909,48 box=606,32 caps=18.1987 face=Barlow heard=Game in=36 layout=197.96,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              79 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1405.5,660,228,48 box=152,32 caps=18.1987 face=Barlow heard=State in=36 layout=151.89,31.2 owner=73 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                              80 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1669.5,660,118.5,48 box=79,32 caps=18.1987 face=Barlow heard=Size in=36 layout=73.29,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="2.0 MB"
            81 Border at=72,888,1776,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              82 Grid at=72,915,1776,138 by=panel limit=0,0,1920,1200 role=none
                83 Grid at=72,915,1776,46.5 by=panel limit=0,0,1920,1200 role=none
                  84 TextBlock at=72,921,1776,40.5 box=1184,27 by=panel caps=15.3989 face=Barlow heard="2 marked: make, linked 1, convert to CHD 1 (658 MB)" layout=466.62,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 marked: make, linked 1, convert to CHD 1 (658 MB)"
                85 StackPanel at=862.5,987,985.5,66 by=panel limit=0,0,1920,1200 role=none
                  86 Button at=862.5,987,255,66 by=panel focus heard="Apply the marked changes" limit=0,0,1920,1200 paints=#707070FF,#D4D4D4FF role=button
                    87 StackPanel at=888,996,204,48 by=clip limit=862.5,987,255,66 role=none
                      88 PadGlyph at=888,1003.5,33,33 by=clip heard=Menu limit=862.5,987,255,66 role=image
                      89 TextBlock at=936,996,124.5,48 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=862.5,987,255,66 owner=86 paints=#D4D4D4FF role=text size=26 text="Apply…"
                  90 StackPanel at=1135.5,987,712.5,66 by=panel limit=0,0,1920,1200 role=none
                    91 ProgressBar at=1135.5,1009.5,300,21 by=panel heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    92 TextBlock at=1447.5,1002,400.5,36 box=267,24 by=panel caps=13.999 face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            93 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              94 HintBar at=96,1074,1728,45 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          95 Panel at=0,0,1920,1053 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            96 Border at=420,207,1080,639 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              97 DockPanel at=469.5,256.5,981,540 by=panel limit=0,0,1920,1200 role=none
                98 TextBlock at=469.5,256.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                99 StackPanel at=469.5,628.5,981,168 by=panel limit=0,0,1920,1200 role=none
                  100 Button at=469.5,628.5,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    101 TextBlock at=511.5,645,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,628.5,981,81 owner=100 paints=#EEEEF0FF role=text size=26 text=No
                  102 Button at=469.5,715.5,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    103 TextBlock at=511.5,732,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,715.5,981,81 owner=102 paints=#D4D4D4FF role=text size=26 text=Yes
                104 ScrollViewer at=469.5,333,981,277.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,333,981,277.5
                  105 StackPanel at=469.5,333,981,277.5 by=fixed-scroll in=104 limit=469.5,333,981,277.5 role=none
                    106 TextBlock at=469.5,333,981,277.5 box=654,185 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc (658 MB) to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=104 layout=605.51,184.8 limit=469.5,333,981,277.5 paints=#D4D4D4FF role=text size=22 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc (658 MB) to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
focus 100
edge 100 up same
edge 100 down 102
edge 100 left same
edge 100 right same
edge 102 up 100
edge 102 down same
edge 102 left same
edge 102 right same
press pageup focus="Button  (No)" top=PackagesWindow
press pagedown focus="Button  (No)" top=PackagesWindow
press first focus="Button  (No)" top=PackagesWindow
press last focus="Button  (No)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged, Size: 658 MB)" top=PackagesWindow
