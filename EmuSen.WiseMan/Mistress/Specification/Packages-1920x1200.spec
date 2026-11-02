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
              28 Border at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                29 Grid at=96,220.5,1728,798 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  30 LunaTable at=96,220.5,1728,798 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,237,1728,781.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1728,781.5
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,237,1728,81 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        33 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,252,1656,52.5 in=31 paints=#707070FF role=none
                          34 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,252,1656,51 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 paints=#4C9AE8FF role=none
                            35 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,253.5,1320,48 box=880,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=872.22,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                            36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,252,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,318,1728,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        38 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,333,1656,52.5 in=31 paints=#707070FF role=none
                          39 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,333,1656,51 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 paints=#4C9AE8FF role=none
                            40 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,334.5,1320,48 box=880,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=273.65,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                            41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,333,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=61.96,33.41 owner=37 paints=#35A064FF role=edit size=26 text="✓ OK"
                      42 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,399,1728,81 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 op paints=#D4D4D4FF role=listitem
                        43 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,414,1656,52.5 in=31 paints=#707070FF role=none
                          44 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,414,1656,51 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 paints=#4C9AE8FF role=none
                            45 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,415.5,1320,48 box=880,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=269.57,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                            46 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,414,228,51 box=152,34 caps=19.1128 drawn face="Adwaita Mono" heard=State in=31 layout=128.28,33.41 owner=42 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      47 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,480,1728,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        48 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,495,1656,49.5 in=31 paints=#707070FF role=none
                          49 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,495,1656,48 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 paints=#4C9AE8FF role=none
                            50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,495,1320,48 box=880,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=873.5,31.2 owner=47 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                            51 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,495,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=47 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      52 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,561,1728,81 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#D4D4D4FF role=listitem
                        53 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,576,1656,49.5 in=31 paints=#707070FF role=none
                          54 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,576,1656,48 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 paints=#4C9AE8FF role=none
                            55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,576,1320,48 box=880,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=201.32,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                            56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,576,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=52 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      57 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,642,1728,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 op paints=#D4D4D4FF role=listitem
                        58 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,657,1656,49.5 in=31 paints=#707070FF role=none
                          59 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=132,657,1656,48 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 paints=#4C9AE8FF role=none
                            60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,657,1320,48 box=880,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=197.96,31.2 owner=57 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                            61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1524,657,228,48 box=152,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=57 paints=#8D8D91FF role=edit size=26 text="Not packaged"
            62 Border at=72,1018.5,1776,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              63 Grid at=78,1024.5,1764,88.5 by=panel limit=0,0,1920,1200 role=none
                64 Grid at=78,1024.5,1764,40.5 by=panel limit=0,0,1920,1200 role=none
                  65 TextBlock at=90,1024.5,1752,40.5 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                66 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                  67 StackPanel at=1129.5,1077,712.5,36 by=panel limit=0,0,1920,1200 role=none
                    68 ProgressBar at=1129.5,1084.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    69 TextBlock at=1441.5,1077,400.5,36 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            70 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              71 HintBar at=144,1135.5,1632,43.5 by=panel heard="Open, Back, Console, List, Apply…, Mark" limit=0,0,1920,1200 role=group
focus 47
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
edge 32 down 37
edge 32 left same
edge 32 right 12
edge 37 up 32
edge 37 down 42
edge 37 left same
edge 37 right 12
edge 42 up 37
edge 42 down 47
edge 42 left same
edge 42 right 12
edge 47 up 42
edge 47 down 52
edge 47 left same
edge 47 right 12
edge 52 up 47
edge 52 down 57
edge 52 left same
edge 52 right 12
edge 57 up 52
edge 57 down same
edge 57 left same
edge 57 right 12
press pageup top=PackagesWindow
press pagedown top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
