# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case ScraperRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 ScraperPane at=48,28,1184,732 by=panel heard=Scraper limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard=SCRAPER limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=996,52,220,24 box=220,24 by=panel caps=13.999 drawn face=Barlow heard="3 games · 3 missing media" layout=219.44,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="3 games · 3 missing media"
                8 StackPanel at=64,44,920,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,920,39 box=920,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=146,100,36,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=146,108,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=146,100,36,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,100,114,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=214,108,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=360,100,154,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Super Nintendo"
                    24 TextBlock at=360,108,154,24 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=360,100,154,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
            25 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,160,691,424 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,160,691,424 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,160,691,424 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,424 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,424
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,54 drawn focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 op paints=#333337FF,#EEEEF0FF role=listitem
                          32 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,53 in=30 role=none
                            33 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,53 in=30 paints=#26262BFF,#4C9AE8FF role=none
                              34 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,160,667,53 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 role=none
                                35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,170,559,32 box=559,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=30 layout=551.98,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                36 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,170,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.5,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 9"
                        37 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,54 drawn focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 9" in=30 op paints=#333337FF,#D4D4D4FF role=listitem
                          38 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,53 in=30 role=none
                            39 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,53 in=30 paints=#0E0E10FF role=none
                              40 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,214,667,53 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 9" in=30 role=none
                                41 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,224,559,32 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=197.96,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                42 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,224,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.45,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="4 of 9"
                        43 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 op paints=#333337FF,#D4D4D4FF role=listitem
                          44 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,53 in=30 role=none
                            45 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,53 in=30 paints=#0E0E10FF role=none
                              46 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,268,667,53 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 role=none
                                47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,278,559,32 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=273.65,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                                48 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,278,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=59.54,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="0 of 9"
                49 Border at=64,584,691,118 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  50 Grid at=64,602,691,100 by=panel limit=0,0,1280,800 role=none
                    51 Grid at=64,602,691,39 by=panel limit=0,0,1280,800 role=none
                      52 ProgressBar at=64,602,691,8 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                      53 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Scraping: 1 of 2 games" layout=206.6,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Scraping: 1 of 2 games"
                    54 StackPanel at=496,658,259,44 by=panel limit=0,0,1280,800 role=none
                      55 Button at=496,658,77,44 by=panel drawn focus heard=STOP limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                        56 TextBlock at=513,664,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=496,658,77,44 owner=55 paints=#D4D4D4FF role=text size=26 text=STOP
                      57 Button at=585,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                        58 StackPanel at=602,664,136,32 by=clip limit=585,658,170,44 off role=none
                          59 PadGlyph at=602,669,22,22 by=clip heard=Menu limit=585,658,170,44 off role=image
                          60 TextBlock at=634,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=585,658,170,44 off owner=57 paints=#707070FF role=text size=26 text="Apply…"
              61 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                62 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  63 ScrollViewer at=795,184,397,494 by=panel fades=b limit=0,0,1280,800 overflow=38 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=795,184,397,494
                    64 StackPanel at=795,184,397,532 by=scroll in=63 limit=795,184,397,494 paints=#D4D4D4FF role=none seen=795,184,397,494
                      65 TextBlock at=795,184,397,116 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=63 layout=365.18,115.2 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      66 TextBlock at=795,306,397,80 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=63 layout=376.4,79.2 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      67 TextBlock at=795,392,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Console: PlayStation" in=63 layout=190.92,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Console: PlayStation"
                      68 TextBlock at=795,425,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="cover: ScreenScraper" in=63 layout=201.76,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      69 TextBlock at=795,458,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="screenshot: missing" in=63 layout=188.21,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      70 TextBlock at=795,491,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="marquee: missing" in=63 layout=167.42,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      71 TextBlock at=795,524,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="titlescreen: the game's package" in=63 layout=294.65,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="titlescreen: the game's package"
                      72 TextBlock at=795,557,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="miximage: missing" in=63 layout=176.37,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="miximage: missing"
                      73 TextBlock at=795,590,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="backcover: missing" in=63 layout=180.91,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="backcover: missing"
                      74 TextBlock at=795,623,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="3dbox: missing" in=63 layout=140.65,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="3dbox: missing"
                      75 TextBlock at=795,656,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="physicalmedia: missing" in=63 layout=219.67,26.4 limit=795,184,397,494 paints=#D4D4D4FF role=text seen=795,656,397,22 size=22 text="physicalmedia: missing"
            76 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              77 HintBar at=64,716,1152,30 by=panel heard="Select, Back, Console, Mark, List" limit=0,0,1280,800 role=group
focus 12
edge 12 up same
edge 12 down 31
edge 12 left 23
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
edge 43 down 55
edge 43 left same
edge 43 right 12
edge 55 up 43
edge 55 down same
edge 55 left 21
edge 55 right 12
press pageup focus="TextBox Search games (Search)" top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9)" top=ScraperWindow
