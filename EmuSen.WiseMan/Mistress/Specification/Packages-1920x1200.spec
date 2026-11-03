# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Packages
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 PackagesPane at=72,42,1776,1098 by=panel heard=Packages limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard=PACKAGES limit=0,0,1920,1200 name=AdaptiveHeader role=none
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
              28 Border at=96,240,1728,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                29 Grid at=96,240,1728,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  30 LunaTable at=96,240,1728,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1728,648
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,81 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        33 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=31 role=none
                          34 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,240,1728,79.5 in=31 paints=#0E0E10FF role=none
                            35 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,240,1692,79.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 role=none
                              36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,255,1392,48 box=928,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=925.86,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                              37 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,255,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      38 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        39 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=31 role=none
                          40 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,321,1728,79.5 in=31 paints=#0E0E10FF role=none
                            41 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,321,1692,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 role=none
                              42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,336,1392,48 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=273.65,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,336,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=38 paints=#35A064FF role=edit size=26 text="✓ OK"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,81 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        45 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=31 role=none
                          46 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,402,1728,79.5 in=31 paints=#0E0E10FF role=none
                            47 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,402,1692,79.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 role=none
                              48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,417,1392,48 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=269.57,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                              49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,417,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=128.28,33.41 owner=44 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      50 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#333337FF,#EEEEF0FF role=listitem
                        51 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=31 role=none
                          52 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,483,1728,79.5 in=31 paints=#26262BFF,#4C9AE8FF role=none
                            53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,483,1692,79.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                              54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,498,1392,48 box=928,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=926.74,31.2 owner=50 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,498,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=50 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      56 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,81 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        57 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=31 role=none
                          58 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,564,1728,79.5 in=31 paints=#0E0E10FF role=none
                            59 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,564,1692,79.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                              60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,579,1392,48 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=201.32,31.2 owner=56 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                              61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,579,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=56 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      62 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                        63 DockPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=31 role=none
                          64 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,645,1728,79.5 in=31 paints=#0E0E10FF role=none
                            65 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,645,1692,79.5 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 role=none
                              66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,660,1392,48 box=928,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=197.96,31.2 owner=62 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1560,660,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=62 paints=#8D8D91FF role=edit size=26 text="Not packaged"
            68 Border at=72,888,1776,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              69 Grid at=72,915,1776,138 by=panel limit=0,0,1920,1200 role=none
                70 Grid at=72,915,1776,46.5 by=panel limit=0,0,1920,1200 role=none
                  71 TextBlock at=72,921,1776,40.5 box=1184,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                72 StackPanel at=862.5,987,985.5,66 by=panel limit=0,0,1920,1200 role=none
                  73 Button at=862.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                    74 StackPanel at=888,996,204,48 by=clip limit=862.5,987,255,66 off role=none
                      75 PadGlyph at=888,1003.5,33,33 by=clip heard=Menu limit=862.5,987,255,66 off role=image
                      76 TextBlock at=936,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=862.5,987,255,66 off owner=73 paints=#707070FF role=text size=26 text="Apply…"
                  77 StackPanel at=1135.5,987,712.5,66 by=panel limit=0,0,1920,1200 role=none
                    78 ProgressBar at=1135.5,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    79 TextBlock at=1447.5,1002,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            80 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              81 HintBar at=96,1074,1728,45 by=panel heard="Open, Back, Console, Mark, List, Apply…" limit=0,0,1920,1200 role=group
focus 50
edge 12 up same
edge 12 down 32
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 32
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 32
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 32
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 32
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 32
edge 25 left 23
edge 25 right 12
edge 32 up 17
edge 32 down 38
edge 32 left same
edge 32 right 12
edge 38 up 32
edge 38 down 44
edge 38 left same
edge 38 right 12
edge 44 up 38
edge 44 down 50
edge 44 left same
edge 44 right 12
edge 50 up 44
edge 50 down 56
edge 50 left same
edge 50 right 12
edge 56 up 50
edge 56 down 62
edge 56 left same
edge 56 right 12
edge 62 up 56
edge 62 down same
edge 62 left same
edge 62 right 12
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
