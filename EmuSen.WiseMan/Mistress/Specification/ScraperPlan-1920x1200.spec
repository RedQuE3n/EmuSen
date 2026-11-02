# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case ScraperPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 ScraperPane at=72,42,1776,1146 by=panel heard=Scraper limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1494,72,330,36 box=220,24 by=panel caps=13.999 face=Barlow heard="3 games · 2 missing media" layout=219.66,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 games · 2 missing media"
                8 StackPanel at=96,60,1380,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1380,58.5 box=920,39 by=panel caps=22.3984 face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,102,66 by=panel focus heard=nes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=nes
                    20 TextBlock at=231,148.5,54,36 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=207,136.5,102,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=321,136.5,99,66 by=panel focus heard=psx limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=psx
                    22 TextBlock at=345,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=PSX layout=33.78,24 limit=321,136.5,99,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PSX
                  23 ToggleButton at=432,136.5,118.5,66 by=panel focus heard=snes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=snes
                    24 TextBlock at=456,148.5,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=432,136.5,118.5,66 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,220.5,1036.5,834 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,817.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,817.5
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,81 focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          32 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,52.5 in=30 paints=#707070FF role=none
                            33 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,51 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 role=none
                              34 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=31 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,253.5,603,48 box=402,32 caps=18.1987 ellipsis face=Barlow heard=Game in=30 layout=395.77,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              36 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,253.5,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.08,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        37 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,318,1036.5,81 focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=30 op paints=#D4D4D4FF role=listitem
                          38 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,52.5 in=30 paints=#707070FF role=none
                            39 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,51 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=30 role=none
                              40 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=37 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              41 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,334.5,603,48 box=402,32 caps=18.1987 face=Barlow heard=Game in=30 layout=197.96,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              42 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,334.5,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.03,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        43 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,399,1036.5,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 op paints=#D4D4D4FF role=listitem
                          44 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,49.5 in=30 paints=#707070FF role=none
                            45 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 role=none
                              46 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,163.5,48 in=30 owner=43 paints=#4C9AE8FF role=text
                              47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,414,603,48 box=402,32 caps=18.1987 face=Barlow heard=Game in=30 layout=273.65,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              48 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,414,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=59.12,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                49 Border at=96,1054.5,1036.5,64.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  50 Grid at=102,1060.5,1024.5,52.5 by=panel limit=0,0,1920,1200 role=none
                    51 Grid at=102,1060.5,1024.5,40.5 by=panel limit=0,0,1920,1200 role=none
                      52 TextBlock at=114,1060.5,1012.5,40.5 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 games marked to scrape" layout=244.11,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 games marked to scrape"
              53 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                54 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  55 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    56 StackPanel at=1204.5,268.5,571.5,802.5 by=fixed-scroll in=55 limit=1204.5,268.5,571.5,802.5 role=none
                      57 TextBlock at=1204.5,268.5,571.5,174 box=381,116 by=fixed-scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=55 layout=365.18,115.2 limit=1204.5,268.5,571.5,802.5 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      58 TextBlock at=1204.5,451.5,571.5,120 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=55 layout=376.4,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      59 TextBlock at=1204.5,580.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="System: psx" in=55 layout=113.15,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      60 TextBlock at=1204.5,630,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="cover: ScreenScraper" in=55 layout=201.76,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      61 TextBlock at=1204.5,679.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="screenshot: missing" in=55 layout=188.21,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      62 TextBlock at=1204.5,729,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="marquee: missing" in=55 layout=167.42,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      63 TextBlock at=1204.5,778.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="miximage: the game's package" in=55 layout=287.54,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="miximage: the game's package"
            64 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              65 HintBar at=144,1135.5,1632,43.5 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          66 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            67 Border at=420,258,1080,715.5 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              68 DockPanel at=469.5,307.5,981,616.5 by=panel limit=0,0,1920,1200 role=none
                69 TextBlock at=469.5,307.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                70 StackPanel at=469.5,756,981,168 by=panel limit=0,0,1920,1200 role=none
                  71 Button at=469.5,756,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    72 TextBlock at=511.5,772.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,756,981,81 owner=71 paints=#EEEEF0FF role=text size=26 text=No
                  73 Button at=469.5,843,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    74 TextBlock at=511.5,859.5,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,843,981,81 owner=73 paints=#D4D4D4FF role=text size=26 text=Yes
                75 StackPanel at=469.5,582,981,156 by=panel limit=0,0,1920,1200 role=none
                  76 ToggleSwitch at=469.5,582,981,48 by=panel drawn focus heard=Theme limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    77 TextBlock at=547.5,585,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,582,981,48 owner=76 paints=#D4D4D4FF role=text size=24 text=Off
                  78 ToggleSwitch at=469.5,636,981,48 by=panel drawn focus heard=Videos limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    79 TextBlock at=547.5,639,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,636,981,48 owner=78 paints=#D4D4D4FF role=text size=24 text=Off
                  80 ToggleSwitch at=469.5,690,981,48 by=panel drawn focus heard=Overwrite limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    81 TextBlock at=547.5,693,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,690,981,48 owner=80 paints=#D4D4D4FF role=text size=24 text=Off
                82 ScrollViewer at=469.5,384,981,198 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,384,981,198
                  83 TextBlock at=469.5,384,981,198 box=654,132 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=82 layout=619.28,132 limit=469.5,384,981,198 paints=#D4D4D4FF role=text size=22 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
focus 71
edge 12 up same
edge 12 down 31
edge 12 left 76
edge 12 right same
edge 17 up same
edge 17 down 31
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 31
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 31
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 31
edge 23 left 21
edge 23 right 12
edge 31 up 17
edge 31 down 37
edge 31 left same
edge 31 right 12
edge 37 up 31
edge 37 down 43
edge 37 left same
edge 37 right 12
edge 43 up 37
edge 43 down 76
edge 43 left same
edge 43 right 12
edge 71 up 80
edge 71 down 73
edge 71 left 21
edge 71 right 12
edge 73 up 71
edge 73 down same
edge 73 left 21
edge 73 right 12
edge 76 up 43
edge 76 down 78
edge 76 left 21
edge 76 right 12
edge 78 up 76
edge 78 down 80
edge 78 left 21
edge 78 right 12
edge 80 up 78
edge 80 down 71
edge 80 left 21
edge 80 right 12
press pageup focus="Button  (No)" top=ScraperWindow
press pagedown focus="Button  (No)" top=ScraperWindow
press first focus="Button  (No)" top=ScraperWindow
press last focus="Button  (No)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
