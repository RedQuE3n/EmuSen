# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case ScraperPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 ScraperPane at=48,28,1184,732 by=panel heard=Scraper limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard=SCRAPER limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=996,52,220,24 box=220,24 by=panel caps=13.999 face=Barlow heard="3 games · 3 missing media" layout=219.44,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="3 games · 3 missing media"
                8 StackPanel at=64,44,920,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,920,39 box=920,39 by=panel caps=22.3984 face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  17 SectionTab at=80,100,34,44 by=panel focus heard=All limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=146,100,36,44 by=panel focus heard=NES limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=NES
                    20 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 SectionTab at=214,100,114,44 by=panel focus heard=PlayStation limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=PlayStation
                    22 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 SectionTab at=360,100,154,44 by=panel focus heard="Super Nintendo" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    24 TextBlock at=360,108,154,24 box=154,24 by=clip caps=13.999 face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=360,100,154,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
            25 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,160,691,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,160,691,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,160,691,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,432
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,54 focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 paints=#333337FF,#EEEEF0FF role=listitem
                          32 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,53 in=30 role=none
                            33 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,53 in=30 paints=#26262BFF,#4C9AE8FF role=none
                              34 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,160,667,53 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 role=none
                                35 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,170,109,34 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=31 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                                36 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=221,170,426,32 box=426,32 caps=18.1987 ellipsis face=Barlow heard=Game in=30 layout=423.36,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                37 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,170,60,32 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.5,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 9"
                        38 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,54 focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 9" in=30 paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,53 in=30 role=none
                            40 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,53 in=30 paints=#0E0E10FF role=none
                              41 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,214,667,53 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 9" in=30 role=none
                                42 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,224,109,34 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ Scrape" in=30 layout=108.34,33.41 owner=38 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                                43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=221,224,426,32 box=426,32 caps=18.1987 face=Barlow heard=Game in=30 layout=197.96,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,224,60,32 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=58.45,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="4 of 9"
                        45 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,54 focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 paints=#333337FF,#D4D4D4FF role=listitem
                          46 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,53 in=30 role=none
                            47 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,53 in=30 paints=#0E0E10FF role=none
                              48 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,268,667,53 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 role=none
                                49 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=221,278,426,32 box=426,32 caps=18.1987 face=Barlow heard=Game in=30 layout=273.65,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                                50 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,278,60,32 box=60,32 caps=18.1987 face=Barlow heard=Media in=30 layout=59.54,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="0 of 9"
                51 Border at=64,592,691,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  52 Grid at=64,610,691,92 by=panel limit=0,0,1280,800 role=none
                    53 Grid at=64,610,691,31 by=panel limit=0,0,1280,800 role=none
                      54 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 face=Barlow heard="2 games marked to scrape" layout=244.11,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 games marked to scrape"
                    55 StackPanel at=585,658,170,44 by=panel limit=0,0,1280,800 role=none
                      56 Button at=585,658,170,44 by=panel focus heard="Apply the marked changes" limit=0,0,1280,800 paints=#707070FF,#D4D4D4FF role=button
                        57 StackPanel at=602,664,136,32 by=clip limit=585,658,170,44 role=none
                          58 PadGlyph at=602,669,22,22 by=clip heard=Menu limit=585,658,170,44 role=image
                          59 TextBlock at=634,664,83,32 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=585,658,170,44 owner=56 paints=#D4D4D4FF role=text size=26 text="Apply…"
              60 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                61 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  62 ScrollViewer at=795,184,397,494 by=panel fades=b limit=0,0,1280,800 overflow=38 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=795,184,397,494
                    63 StackPanel at=795,184,397,532 by=scroll in=62 limit=795,184,397,494 paints=#D4D4D4FF role=none seen=795,184,397,494
                      64 TextBlock at=795,184,397,116 box=397,116 by=scroll caps=22.3984 face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=62 layout=365.18,115.2 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      65 TextBlock at=795,306,397,80 box=397,80 by=scroll caps=15.3989 face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=62 layout=376.4,79.2 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      66 TextBlock at=795,392,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="Console: PlayStation" in=62 layout=190.92,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Console: PlayStation"
                      67 TextBlock at=795,425,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="cover: ScreenScraper" in=62 layout=201.76,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      68 TextBlock at=795,458,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="screenshot: missing" in=62 layout=188.21,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      69 TextBlock at=795,491,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="marquee: missing" in=62 layout=167.42,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      70 TextBlock at=795,524,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="titlescreen: the game's package" in=62 layout=294.65,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="titlescreen: the game's package"
                      71 TextBlock at=795,557,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="miximage: missing" in=62 layout=176.37,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="miximage: missing"
                      72 TextBlock at=795,590,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="backcover: missing" in=62 layout=180.91,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="backcover: missing"
                      73 TextBlock at=795,623,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="3dbox: missing" in=62 layout=140.65,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="3dbox: missing"
                      74 TextBlock at=795,656,397,27 box=397,27 by=scroll caps=15.3989 face=Barlow heard="physicalmedia: missing" in=62 layout=219.67,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text seen=795,656,397,22 size=22 text="physicalmedia: missing"
            75 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              76 HintBar at=64,716,1152,30 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          77 Panel at=0,0,1280,702 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            78 Border at=280,47,720,608 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              79 DockPanel at=313,80,654,542 by=panel limit=0,0,1280,800 role=none
                80 TextBlock at=313,80,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                81 StackPanel at=313,510,654,112 by=panel limit=0,0,1280,800 role=none
                  82 Button at=313,510,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    83 TextBlock at=341,521,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,510,654,54 owner=82 paints=#EEEEF0FF role=text size=26 text=No
                  84 Button at=313,568,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    85 TextBlock at=341,579,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,568,654,54 owner=84 paints=#D4D4D4FF role=text size=26 text=Yes
                86 StackPanel at=313,290,654,208 by=panel limit=0,0,1280,800 role=none
                  87 DockPanel at=313,290,654,54 by=panel limit=0,0,1280,800 role=none
                    88 ToggleSwitch at=892,301,75,32 by=panel drawn focus heard=Videos limit=0,0,1280,800 op paints=#8D8D91FF,#D4D4D4FF role=button
                      89 TextBlock at=944,303,23,29 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=892,301,75,32 owner=88 paints=#D4D4D4FF role=text size=24 text=Off
                    90 TextBlock at=313,301,579,32 box=579,32 by=panel caps=18.1987 drawn face=Barlow heard=Videos layout=74.07,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Videos
                  91 TextBlock at=337,348,630,24 box=630,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Also fetches each game's video. Videos are large, so this stays off unless you turn it on." layout=563.26,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="Also fetches each game's video. Videos are large, so this stays off unless you turn it on."
                  92 DockPanel at=313,384,654,54 by=panel limit=0,0,1280,800 role=none
                    93 ToggleSwitch at=892,395,75,32 by=panel drawn focus heard=Overwrite limit=0,0,1280,800 op paints=#8D8D91FF,#D4D4D4FF role=button
                      94 TextBlock at=944,397,23,29 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=892,395,75,32 owner=93 paints=#D4D4D4FF role=text size=24 text=Off
                    95 TextBlock at=313,395,579,32 box=579,32 by=panel caps=18.1987 drawn face=Barlow heard=Overwrite layout=106.21,31.2 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=26 text=Overwrite
                  96 TextBlock at=337,442,630,48 box=630,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Fetches again and replaces the pictures and details you already have; off fetches only what is missing." layout=606.3,48 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="Fetches again and replaces the pictures and details you already have; off fetches only what is missing."
                97 ScrollViewer at=313,131,654,159 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,131,654,159
                  98 StackPanel at=313,131,654,159 by=fixed-scroll in=97 limit=313,131,654,159 role=none
                    99 TextBlock at=313,131,654,159 box=654,159 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, titlescreen, miximage, backcover, 3dbox, physicalmedia, fanart.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=97 layout=638.11,158.4 limit=313,131,654,159 paints=#D4D4D4FF role=text size=22 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, titlescreen, miximage, backcover, 3dbox, physicalmedia, fanart.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
focus 82
edge 82 up 93
edge 82 down 84
edge 82 left same
edge 82 right same
edge 84 up 82
edge 84 down same
edge 84 left same
edge 84 right same
edge 88 up same
edge 88 down 93
edge 88 left same
edge 88 right same
edge 93 up 88
edge 93 down 82
edge 93 left same
edge 93 right same
press pageup focus="Button  (No)" top=ScraperWindow
press pagedown focus="Button  (No)" top=ScraperWindow
press first focus="Button  (No)" top=ScraperWindow
press last focus="Button  (No)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9)" top=ScraperWindow
