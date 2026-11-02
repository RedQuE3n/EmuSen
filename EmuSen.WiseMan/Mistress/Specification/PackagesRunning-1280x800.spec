# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PackagesRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 PackagesPane at=48,28,1184,764 by=panel heard=Packages limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=943,48,273,24 box=273,24 by=panel caps=13.999 drawn face=Barlow heard="6 games · 3 packaged · 1 problem" layout=272.26,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problem"
                8 StackPanel at=64,40,867,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,867,39 box=867,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=154,99,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,91,68,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,91,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=230,99,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,91,146,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=368,91,106,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=384,99,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=368,91,106,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=482,91,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=498,99,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=482,91,79,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,147,1152,532 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,147,1152,532 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                29 Grid at=64,147,1152,532 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  30 LunaTable at=64,147,1152,532 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,158,1152,521 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,1152,521
                      32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,158,1152,54 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        33 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,168,1104,34 in=31 paints=#707070FF role=none
                          34 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,168,1104,33 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK" in=31 role=none
                            35 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,168,194,32 drawn in=31 owner=32 paints=#4C9AE8FF role=text
                            36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,168,450,32 box=450,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=439.09,31.2 owner=32 paints=#D4D4D4FF role=edit size=26 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                            37 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,168,388,33 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=61.06,32.27 owner=32 paints=#35A064FF role=edit size=26 text="✓ OK"
                      38 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,212,1152,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 op paints=#D4D4D4FF role=listitem
                        39 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,222,1104,34 in=31 paints=#707070FF role=none
                          40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,222,1104,33 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: ✓ OK" in=31 role=none
                            41 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,222,194,32 drawn in=31 owner=38 paints=#4C9AE8FF role=text
                            42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,222,450,32 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=273.65,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                            43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,222,388,33 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=61.06,32.27 owner=38 paints=#35A064FF role=edit size=26 text="✓ OK"
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,266,1152,54 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 op paints=#D4D4D4FF role=listitem
                        45 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,276,1104,34 in=31 paints=#707070FF role=none
                          46 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,276,1104,33 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: ⚠ Changed" in=31 role=none
                            47 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,276,194,32 drawn in=31 owner=44 paints=#4C9AE8FF role=text
                            48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,276,450,32 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=269.57,31.2 owner=44 paints=#D4D4D4FF role=edit size=26 text="Ruby Lantern (Synthetic)"
                            49 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,276,388,33 box=388,33 caps=18.916 drawn face="Adwaita Sans" heard=State in=31 layout=131.06,32.27 owner=44 paints=#DAA520FF role=edit size=26 text="⚠ Changed"
                      50 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,320,1152,54 drawn focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        51 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,330,1104,34 in=31 paints=#707070FF role=none
                          52 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,330,1104,33 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                            53 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,330,194,33 box=194,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ convert to CHD" in=31 layout=193.19,32.27 owner=50 paints=#4C9AE8FF role=text size=26 text="✓ convert to CHD"
                            54 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,330,450,32 box=450,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=31 layout=446.34,31.2 owner=50 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                            55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,330,388,32 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=50 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      56 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,374,1152,54 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#D4D4D4FF role=listitem
                        57 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,384,1104,33 in=31 paints=#707070FF role=none
                          58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,384,1104,32 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                            59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,384,194,32 drawn in=31 owner=56 paints=#4C9AE8FF role=text
                            60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,384,450,32 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=201.32,31.2 owner=56 paints=#D4D4D4FF role=edit size=26 text="Ember Choir (USA)"
                            61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,384,388,32 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=151.89,31.2 owner=56 paints=#8D8D91FF role=edit size=26 text="Not packaged"
                      62 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=64,428,1152,54 drawn focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 op paints=#D4D4D4FF role=listitem
                        63 Border allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,438,1104,34 in=31 paints=#707070FF role=none
                          64 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,438,1104,33 heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 role=none
                            65 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=88,438,194,33 box=194,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ make, linked" in=31 layout=167.79,32.27 owner=62 paints=#4C9AE8FF role=text size=26 text="✓ make, linked"
                            66 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=306,438,450,32 box=450,32 caps=18.1987 drawn face=Barlow heard=Game in=31 layout=197.96,31.2 owner=62 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                            67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=780,438,388,32 box=388,32 caps=18.1987 drawn face=Barlow heard=State in=31 layout=387.63,31.2 owner=62 paints=#8D8D91FF role=edit size=26 text="Made, linked to the game where it is"
            68 Border at=48,679,1184,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              69 Grid at=52,683,1176,59 by=panel limit=0,0,1280,800 role=none
                70 Grid at=52,683,1176,27 by=panel limit=0,0,1280,800 role=none
                  71 ProgressBar at=52,695,120,4 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                  72 TextBlock at=180,683,1048,27 box=1048,27 by=panel caps=15.3989 drawn face=Barlow heard="Making packages: 1 of 2, Amber Saga (USA)" layout=393.23,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Making packages: 1 of 2, Amber Saga (USA)"
                73 StackPanel at=753,718,475,24 by=panel limit=0,0,1280,800 role=none
                  74 StackPanel at=753,718,475,24 by=panel limit=0,0,1280,800 role=none
                    75 ProgressBar at=753,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    76 TextBlock at=961,718,267,24 box=267,24 by=panel caps=13.999 drawn face=Barlow heard="263 KB in packages · 412 GB free" layout=266.3,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="263 KB in packages · 412 GB free"
            77 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              78 HintBar at=96,757,1088,29 by=panel heard="Open, Back, Console, List, Apply…, Mark" limit=0,0,1280,800 role=group
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
edge 32 right same
edge 38 up 32
edge 38 down 44
edge 38 left same
edge 38 right same
edge 44 up 38
edge 44 down 50
edge 44 left same
edge 44 right same
edge 50 up 44
edge 50 down 56
edge 50 left same
edge 50 right same
edge 56 up 50
edge 56 down 62
edge 56 left same
edge 56 right same
edge 62 up 56
edge 62 down same
edge 62 left same
edge 62 right same
press pageup focus="TextBox Search games (Search)" top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" top=PackagesWindow
press first focus="TextBox Search games (Search)" top=PackagesWindow
press last focus="TextBox Search games (Search)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
