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
      2 PackagesPane at=72,42,1776,1146 by=panel heard=Packages limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=PACKAGES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1414.5,72,409.5,36 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=96,60,1300.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1300.5,58.5 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
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
              28 Border at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                29 Grid at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  30 LunaTable at=96,220.5,1728,798 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,237,1728,781.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1728,781.5
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,237,1728,81 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        33 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,252,1656,51 in=31 paints=#707070FF role=none
                          34 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,252,1656,49.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 role=none
                            35 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,252,291,48 drawn in=31 owner=32 paints=#4C9AE8FF role=text
                            36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,252,675,48 box=450,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=439.09,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                            37 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,252,582,49.5 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=61.06,32.27 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      38 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,318,1728,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        39 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,333,1656,51 in=31 paints=#707070FF role=none
                          40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,333,1656,49.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 role=none
                            41 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,333,291,48 drawn in=31 owner=38 paints=#4C9AE8FF role=text
                            42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,333,675,48 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=273.65,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                            43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,333,582,49.5 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=61.06,32.27 owner=38 paints=#35A064FF role=edit size=26 text="✓ OK"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,399,1728,81 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 op paints=#D4D4D4FF role=listitem
                        45 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,414,1656,51 in=31 paints=#707070FF role=none
                          46 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,414,1656,49.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 role=none
                            47 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,414,291,48 drawn in=31 owner=44 paints=#4C9AE8FF role=text
                            48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,414,675,48 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=269.57,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                            49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,414,582,49.5 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=131.06,32.27 owner=44 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      50 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,480,1728,81 drawn focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        51 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,495,1656,51 in=31 paints=#707070FF role=none
                          52 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,495,1656,49.5 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                            53 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,495,291,49.5 box=194,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ convert to CHD" in=31 layout=193.19,32.27 owner=50 paints=#4C9AE8FF role=text size=26 text="✓ convert to CHD"
                            54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,495,675,48 box=450,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=446.34,31.2 owner=50 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                            55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,495,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=50 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      56 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,561,1728,81 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#D4D4D4FF role=listitem
                        57 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,576,1656,49.5 in=31 paints=#707070FF role=none
                          58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,576,1656,48 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                            59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,576,291,48 drawn in=31 owner=56 paints=#4C9AE8FF role=text
                            60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,576,675,48 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=201.32,31.2 owner=56 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                            61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,576,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=56 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      62 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,642,1728,81 drawn focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 op paints=#D4D4D4FF role=listitem
                        63 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,657,1656,51 in=31 paints=#707070FF role=none
                          64 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,657,1656,49.5 heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 role=none
                            65 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,657,291,49.5 box=194,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ make, linked" in=31 layout=167.79,32.27 owner=62 paints=#4C9AE8FF role=text size=26 text="✓ make, linked"
                            66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=459,657,675,48 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=197.96,31.2 owner=62 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                            67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1170,657,582,48 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=387.63,31.2 owner=62 paints=#8D8D91FF role=edit size=26 text="Made, linked to the game where it is"
            68 Border at=72,1018.5,1776,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              69 Grid at=78,1024.5,1764,88.5 by=panel limit=0,0,1920,1200 role=none
                70 Grid at=78,1024.5,1764,40.5 by=panel limit=0,0,1920,1200 role=none
                  71 ProgressBar at=78,1042.5,180,6 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                  72 TextBlock at=270,1024.5,1572,40.5 box=1048,27 by=panel caps=15.3989 drawn face=Barlow heard="Making packages: 1 of 2, Amber Saga (USA)" layout=393.23,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Making packages: 1 of 2, Amber Saga (USA)"
                73 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                  74 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                    75 ProgressBar at=1129.5,1084.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    76 TextBlock at=1441.5,1077,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            77 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              78 HintBar at=144,1135.5,1632,43.5 by=panel heard="Open, Back, Console, List, Apply…, Mark" limit=0,0,1920,1200 role=group
focus 12
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
press pageup focus="TextBox Search games (Search)" top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
