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
      2 ScraperPane at=72,42,1776,1098 by=panel heard=Scraper limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1494,78,330,36 box=220,24 by=panel caps=13.999 face=Barlow heard="3 games · 2 missing media" layout=219.66,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 games · 2 missing media"
                8 StackPanel at=96,66,1380,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1380,58.5 box=920,39 by=panel caps=22.3984 face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel focus heard=Search limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=120,150,51,66 by=panel focus heard=All limit=0,0,1920,1200 paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=219,150,54,66 by=panel focus heard=nes limit=0,0,1920,1200 paints=#D4D4D4FF role=button text=nes
                    20 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=321,150,51,66 by=panel focus heard=psx limit=0,0,1920,1200 paints=#D4D4D4FF role=button text=psx
                    22 TextBlock at=321,162,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=PSX layout=33.78,24 limit=321,150,51,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PSX
                  23 ToggleButton at=420,150,70.5,66 by=panel focus heard=snes limit=0,0,1920,1200 paints=#D4D4D4FF role=button text=snes
                    24 TextBlock at=420,162,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=420,150,70.5,66 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,81 focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 paints=#333337FF,#EEEEF0FF role=listitem
                          32 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=30 role=none
                            33 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=30 paints=#26262BFF,#4C9AE8FF role=none
                              34 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 role=none
                                35 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=31 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                                36 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,255,639,48 box=426,32 caps=18.1987 ellipsis face=Barlow heard=Game in=30 layout=423.36,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                37 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,255,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.08,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        38 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,81 focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=30 paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=30 role=none
                            40 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=30 paints=#0E0E10FF role=none
                              41 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=30 role=none
                                42 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=38 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                                43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,336,639,48 box=426,32 caps=18.1987 face=Barlow heard=Game in=30 layout=197.96,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,336,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.03,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        45 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,81 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 paints=#333337FF,#D4D4D4FF role=listitem
                          46 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=30 role=none
                            47 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=30 paints=#0E0E10FF role=none
                              48 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 role=none
                                49 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=331.5,417,639,48 box=426,32 caps=18.1987 face=Barlow heard=Game in=30 layout=273.65,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                                50 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,417,90,48 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=59.12,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                51 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  52 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    53 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      54 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 face=Barlow heard="2 games marked to scrape" layout=244.11,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 games marked to scrape"
                    55 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      56 Button at=877.5,987,255,66 by=panel focus heard="Apply the marked changes" limit=0,0,1920,1200 paints=#707070FF,#D4D4D4FF role=button
                        57 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 role=none
                          58 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 role=image
                          59 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 owner=56 paints=#D4D4D4FF role=text size=26 text="Apply…"
              60 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                61 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  62 ScrollViewer at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1192.5,276,595.5,741
                    63 StackPanel at=1192.5,276,595.5,741 by=fixed-scroll in=62 limit=1192.5,276,595.5,741 role=none
                      64 TextBlock at=1192.5,276,595.5,174 box=397,116 by=fixed-scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=62 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      65 TextBlock at=1192.5,459,595.5,120 box=397,80 by=fixed-scroll caps=15.3989 face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=62 layout=376.4,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      66 TextBlock at=1192.5,588,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="System: psx" in=62 layout=113.15,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      67 TextBlock at=1192.5,637.5,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="cover: ScreenScraper" in=62 layout=201.76,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      68 TextBlock at=1192.5,687,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="screenshot: missing" in=62 layout=188.21,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      69 TextBlock at=1192.5,736.5,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="marquee: missing" in=62 layout=167.42,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      70 TextBlock at=1192.5,786,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="miximage: the game's package" in=62 layout=287.54,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="miximage: the game's package"
            71 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              72 HintBar at=96,1074,1728,45 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          73 Panel at=0,0,1920,1053 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            74 Border at=420,90,1080,871.5 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              75 DockPanel at=469.5,139.5,981,772.5 by=panel limit=0,0,1920,1200 role=none
                76 TextBlock at=469.5,139.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                77 StackPanel at=469.5,744,981,168 by=panel limit=0,0,1920,1200 role=none
                  78 Button at=469.5,744,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    79 TextBlock at=511.5,760.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,744,981,81 owner=78 paints=#EEEEF0FF role=text size=26 text=No
                  80 Button at=469.5,831,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    81 TextBlock at=511.5,847.5,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,831,981,81 owner=80 paints=#D4D4D4FF role=text size=26 text=Yes
                82 StackPanel at=469.5,414,981,312 by=panel limit=0,0,1920,1200 role=none
                  83 DockPanel at=469.5,414,981,81 by=panel limit=0,0,1920,1200 role=none
                    84 ToggleSwitch at=1338,430.5,112.5,48 by=panel drawn focus heard=Videos limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                      85 TextBlock at=1416,433.5,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=1338,430.5,112.5,48 owner=84 paints=#D4D4D4FF role=text size=24 text=Off
                    86 TextBlock at=469.5,430.5,868.5,48 box=579,32 by=panel caps=18.1987 drawn face=Barlow heard=Videos layout=74.07,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Videos
                  87 TextBlock at=505.5,501,945,36 box=630,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Also fetches each game's video. Videos are large, so this stays off unless you turn it on." layout=563.26,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="Also fetches each game's video. Videos are large, so this stays off unless you turn it on."
                  88 DockPanel at=469.5,555,981,81 by=panel limit=0,0,1920,1200 role=none
                    89 ToggleSwitch at=1338,571.5,112.5,48 by=panel drawn focus heard=Overwrite limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                      90 TextBlock at=1416,574.5,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=1338,571.5,112.5,48 owner=89 paints=#D4D4D4FF role=text size=24 text=Off
                    91 TextBlock at=469.5,571.5,868.5,48 box=579,32 by=panel caps=18.1987 drawn face=Barlow heard=Overwrite layout=106.21,31.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=26 text=Overwrite
                  92 TextBlock at=505.5,642,945,72 box=630,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Fetches again and replaces the pictures and details you already have; off fetches only what is missing." layout=606.3,48 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="Fetches again and replaces the pictures and details you already have; off fetches only what is missing."
                93 ScrollViewer at=469.5,216,981,198 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,216,981,198
                  94 StackPanel at=469.5,216,981,198 by=fixed-scroll in=93 limit=469.5,216,981,198 role=none
                    95 TextBlock at=469.5,216,981,198 box=654,132 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=93 layout=619.28,132 limit=469.5,216,981,198 paints=#D4D4D4FF role=text size=22 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
focus 78
edge 78 up 89
edge 78 down 80
edge 78 left same
edge 78 right same
edge 80 up 78
edge 80 down same
edge 80 left same
edge 80 right same
edge 84 up same
edge 84 down 89
edge 84 left same
edge 84 right same
edge 89 up 84
edge 89 down 78
edge 89 left same
edge 89 right same
press pageup focus="Button  (No)" top=ScraperWindow
press pagedown focus="Button  (No)" top=ScraperWindow
press first focus="Button  (No)" top=ScraperWindow
press last focus="Button  (No)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
