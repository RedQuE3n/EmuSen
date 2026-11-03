# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PackagesRunning
entered down
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
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
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
            27 Grid at=96,240,1728,636 by=panel limit=0,0,1920,1200 role=none
              28 Border at=96,240,1728,636 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                29 Grid at=96,240,1728,636 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  30 LunaTable at=96,240,1728,636 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,636 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1728,636
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,81 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        33 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=31 role=none
                          34 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=31 paints=#0E0E10FF role=none
                            35 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,240,1692,79.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 role=none
                              36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,255,1038,48 box=692,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=679.41,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              37 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,255,582,51 box=388,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      38 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        39 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=31 role=none
                          40 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=31 paints=#0E0E10FF role=none
                            41 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,321,1692,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 role=none
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,336,1038,48 box=692,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=273.65,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,336,582,51 box=388,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=38 paints=#35A064FF role=edit size=26 text="✓ OK"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,81 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        45 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=31 role=none
                          46 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=31 paints=#0E0E10FF role=none
                            47 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,402,1692,79.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 role=none
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,417,1038,48 box=692,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=269.57,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,417,582,51 box=388,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=128.28,33.41 owner=44 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      50 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,81 drawn focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#333337FF,#EEEEF0FF role=listitem
                        51 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=31 role=none
                          52 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=31 paints=#26262BFF,#4C9AE8FF role=none
                            53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,483,1692,79.5 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                              54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,498,1038,48 box=692,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=687.62,31.2 owner=50 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,498,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=50 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      56 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,81 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        57 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=31 role=none
                          58 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=31 paints=#0E0E10FF role=none
                            59 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,564,1692,79.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                              60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,579,1038,48 box=692,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=201.32,31.2 owner=56 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,579,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=56 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      62 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,81 drawn focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        63 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=31 role=none
                          64 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=31 paints=#0E0E10FF role=none
                            65 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,645,1692,79.5 heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 role=none
                              66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,660,1038,48 box=692,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=197.96,31.2 owner=62 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1206,660,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=387.63,31.2 owner=62 paints=#8D8D91FF role=edit size=26 text="Made, linked to the game where it is"
            68 Border at=72,876,1776,177 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              69 Grid at=72,903,1776,150 by=panel limit=0,0,1920,1200 role=none
                70 Grid at=72,903,1776,58.5 by=panel limit=0,0,1920,1200 role=none
                  71 ProgressBar at=72,903,1776,12 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                  72 TextBlock at=72,921,1776,40.5 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Making packages: 1 of 2, Amber Saga (USA)" layout=393.23,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Making packages: 1 of 2, Amber Saga (USA)"
                73 StackPanel at=862.5,987,985.5,66 by=panel limit=0,0,1920,1200 role=none
                  74 Button at=862.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                    75 StackPanel at=888,996,204,48 by=clip limit=862.5,987,255,66 off role=none
                      76 PadGlyph at=888,1003.5,33,33 by=clip heard=Menu limit=862.5,987,255,66 off role=image
                      77 TextBlock at=936,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=862.5,987,255,66 off owner=74 paints=#707070FF role=text size=26 text="Apply…"
                  78 StackPanel at=1135.5,987,712.5,66 by=panel limit=0,0,1920,1200 role=none
                    79 ProgressBar at=1135.5,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    80 TextBlock at=1447.5,1002,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            81 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              82 HintBar at=96,1074,1728,45 by=panel heard="Open, Back, Console, Mark, List" limit=0,0,1920,1200 role=group
focus 12
edge 12 up same
edge 12 down 32
edge 12 left 25
edge 12 right same
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
edge 37 right 17
edge 43 up 37
edge 43 down 49
edge 43 left same
edge 43 right 17
edge 49 up 43
edge 49 down 55
edge 49 left same
edge 49 right 17
edge 55 up 49
edge 55 down 61
edge 55 left same
edge 55 right 17
edge 61 up 55
edge 61 down 67
edge 61 left same
edge 61 right 17
edge 67 up 61
edge 67 down same
edge 67 left same
edge 67 right 17
press pageup focus="TextBox Search games (Search)" top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
