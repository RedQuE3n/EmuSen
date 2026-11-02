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
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                  8 TextBlock at=96,60,1728,58.5 box=1152,39 by=panel caps=22.3984 face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              10 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                11 TextBox at=1554,136.5,270,66 by=panel focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 face=Barlow heard="Search games" in=12 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              14 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                15 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  16 ToggleButton at=96,136.5,99,66 by=panel focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    17 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=16 paints=#EEEEF0FF role=text size=20 text=ALL
                  18 ToggleButton at=207,136.5,102,66 by=panel focus heard=nes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=nes
                    19 TextBlock at=231,148.5,54,36 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=207,136.5,102,66 owner=18 paints=#D4D4D4FF role=text size=20 text=NES
                  20 ToggleButton at=321,136.5,99,66 by=panel focus heard=psx limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=psx
                    21 TextBlock at=345,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=PSX layout=33.78,24 limit=321,136.5,99,66 owner=20 paints=#D4D4D4FF role=text size=20 text=PSX
                  22 ToggleButton at=432,136.5,118.5,66 by=panel focus heard=snes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=snes
                    23 TextBlock at=456,148.5,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=432,136.5,118.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text=SNES
            24 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              25 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                26 Border at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  27 Grid at=96,220.5,1036.5,834 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    28 LunaTable at=96,220.5,1036.5,834 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,817.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,817.5
                        30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,81 focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          31 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,51 in=29 paints=#707070FF role=none
                            32 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,49.5 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 role=none
                              33 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,162,49.5 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ Scrape" in=29 layout=107.45,32.27 owner=30 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,252,604.5,48 box=403,32 caps=18.1987 ellipsis face=Barlow heard=Game in=29 layout=395.77,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,252,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=29 layout=58.08,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        36 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,318,1036.5,81 focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          37 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,51 in=29 paints=#707070FF role=none
                            38 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,49.5 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 role=none
                              39 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,162,49.5 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ Scrape" in=29 layout=107.45,32.27 owner=36 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              40 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,333,604.5,48 box=403,32 caps=18.1987 face=Barlow heard=Game in=29 layout=197.96,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              41 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,333,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=29 layout=58.03,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        42 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,399,1036.5,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          43 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,49.5 in=29 paints=#707070FF role=none
                            44 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 role=none
                              45 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,162,48 in=29 owner=42 paints=#4C9AE8FF role=text
                              46 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,414,604.5,48 box=403,32 caps=18.1987 face=Barlow heard=Game in=29 layout=273.65,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,414,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=29 layout=59.12,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                48 Border at=96,1054.5,1036.5,64.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  49 Grid at=102,1060.5,1024.5,52.5 by=panel limit=0,0,1920,1200 role=none
                    50 Grid at=102,1060.5,1024.5,40.5 by=panel limit=0,0,1920,1200 role=none
                      51 TextBlock at=114,1060.5,1012.5,40.5 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 games marked to scrape" layout=244.11,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 games marked to scrape"
              52 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                53 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  54 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    55 StackPanel at=1204.5,268.5,571.5,802.5 by=fixed-scroll in=54 limit=1204.5,268.5,571.5,802.5 role=none
                      56 TextBlock at=1204.5,268.5,571.5,120 box=381,80 by=fixed-scroll caps=16.0864 face="Barlow Medium" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=54 layout=333.56,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      57 TextBlock at=1204.5,397.5,571.5,120 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=54 layout=376.4,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      58 TextBlock at=1204.5,526.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="System: psx" in=54 layout=113.15,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      59 TextBlock at=1204.5,576,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="cover: ScreenScraper" in=54 layout=201.76,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      60 TextBlock at=1204.5,625.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="screenshot: missing" in=54 layout=188.21,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      61 TextBlock at=1204.5,675,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="marquee: missing" in=54 layout=167.42,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      62 TextBlock at=1204.5,724.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="miximage: the game's package" in=54 layout=287.54,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="miximage: the game's package"
            63 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              64 HintBar at=144,1135.5,1632,43.5 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          65 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveCover role=none
            66 Border at=420,258,1080,715.5 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              67 DockPanel at=469.5,307.5,981,616.5 by=panel limit=0,0,1920,1200 role=none
                68 TextBlock at=469.5,307.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                69 StackPanel at=469.5,756,981,168 by=panel limit=0,0,1920,1200 role=none
                  70 Button at=469.5,756,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    71 TextBlock at=511.5,772.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,756,981,81 owner=70 paints=#EEEEF0FF role=text size=26 text=No
                  72 Button at=469.5,843,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    73 TextBlock at=511.5,859.5,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,843,981,81 owner=72 paints=#D4D4D4FF role=text size=26 text=Yes
                74 StackPanel at=469.5,582,981,156 by=panel limit=0,0,1920,1200 role=none
                  75 ToggleSwitch at=469.5,582,981,48 by=panel drawn focus heard=Theme limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    76 TextBlock at=547.5,585,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,582,981,48 owner=75 paints=#D4D4D4FF role=text size=24 text=Off
                  77 ToggleSwitch at=469.5,636,981,48 by=panel drawn focus heard=Videos limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    78 TextBlock at=547.5,639,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,636,981,48 owner=77 paints=#D4D4D4FF role=text size=24 text=Off
                  79 ToggleSwitch at=469.5,690,981,48 by=panel drawn focus heard=Overwrite limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                    80 TextBlock at=547.5,693,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=469.5,690,981,48 owner=79 paints=#D4D4D4FF role=text size=24 text=Off
                81 ScrollViewer at=469.5,384,981,198 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,384,981,198
                  82 TextBlock at=469.5,384,981,198 box=654,132 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=81 layout=619.28,132 limit=469.5,384,981,198 paints=#D4D4D4FF role=text size=22 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
focus 70
edge 11 up same
edge 11 down 30
edge 11 left 75
edge 11 right same
edge 16 up same
edge 16 down 30
edge 16 left same
edge 16 right 18
edge 18 up same
edge 18 down 30
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 30
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 30
edge 22 left 20
edge 22 right 11
edge 30 up 16
edge 30 down 36
edge 30 left same
edge 30 right 11
edge 36 up 30
edge 36 down 42
edge 36 left same
edge 36 right 11
edge 42 up 36
edge 42 down 75
edge 42 left same
edge 42 right 11
edge 70 up 79
edge 70 down 72
edge 70 left 20
edge 70 right 11
edge 72 up 70
edge 72 down same
edge 72 left 20
edge 72 right 11
edge 75 up 42
edge 75 down 77
edge 75 left 20
edge 75 right 11
edge 77 up 75
edge 77 down 79
edge 77 left 20
edge 77 right 11
edge 79 up 77
edge 79 down 70
edge 79 left 20
edge 79 right 11
press pageup focus="Button  (No)" top=ScraperWindow
press pagedown focus="Button  (No)" top=ScraperWindow
press first focus="Button  (No)" top=ScraperWindow
press last focus="Button  (No)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
