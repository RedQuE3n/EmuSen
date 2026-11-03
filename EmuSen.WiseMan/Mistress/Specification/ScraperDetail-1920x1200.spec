# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case ScraperDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 ScraperPane at=72,42,1776,1098 by=panel heard=Scraper limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1494,78,330,36 box=220,24 by=panel caps=13.999 drawn face=Barlow heard="3 games · 3 missing media" layout=219.44,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 games · 3 missing media"
                8 StackPanel at=96,66,1380,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1380,58.5 box=920,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=219,150,54,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=321,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=321,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,150,171,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=540,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Super Nintendo"
                    24 TextBlock at=540,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=540,150,231,66 owner=23 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
            25 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,240,1036.5,609 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,240,1036.5,609 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,240,1036.5,609 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,609 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,609
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 op paints=#333337FF,#EEEEF0FF role=listitem
                          32 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=30 role=none
                            33 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=30 paints=#26262BFF,#4C9AE8FF role=none
                              34 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=30 role=none
                                35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,838.5,48 box=559,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=30 layout=551.98,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                36 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,255,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.5,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 9"
                        37 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 9" in=30 op paints=#333337FF,#D4D4D4FF role=listitem
                          38 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=30 role=none
                            39 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=30 paints=#0E0E10FF role=none
                              40 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 9" in=30 role=none
                                41 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,838.5,48 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=197.96,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                42 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,336,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.45,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="4 of 9"
                        43 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 op paints=#333337FF,#D4D4D4FF role=listitem
                          44 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=30 role=none
                            45 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=30 paints=#0E0E10FF role=none
                              46 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=30 role=none
                                47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,417,838.5,48 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=273.65,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                                48 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,417,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=59.54,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="0 of 9"
                49 Border at=96,849,1036.5,204 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  50 Grid at=96,876,1036.5,177 by=panel limit=0,0,1920,1200 role=none
                    51 Grid at=96,876,1036.5,85.5 by=panel limit=0,0,1920,1200 role=none
                      52 TextBlock at=96,882,1036.5,79.5 box=691,53 by=panel caps=15.3989 drawn face=Barlow heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=625.86,52.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
                    53 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      54 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        55 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          56 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          57 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=54 paints=#707070FF role=text size=26 text="Apply…"
              58 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                59 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  60 ScrollViewer at=1192.5,276,595.5,741 by=panel fades=b limit=0,0,1920,1200 overflow=57 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1192.5,276,595.5,741
                    61 StackPanel at=1192.5,276,595.5,798 by=scroll in=60 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=none seen=1192.5,276,595.5,741
                      62 TextBlock at=1192.5,276,595.5,174 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=60 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      63 TextBlock at=1192.5,459,595.5,120 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=60 layout=376.4,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      64 TextBlock at=1192.5,588,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Console: PlayStation" in=60 layout=190.92,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Console: PlayStation"
                      65 TextBlock at=1192.5,637.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="cover: ScreenScraper" in=60 layout=201.76,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      66 TextBlock at=1192.5,687,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="screenshot: missing" in=60 layout=188.21,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      67 TextBlock at=1192.5,736.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="marquee: missing" in=60 layout=167.42,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      68 TextBlock at=1192.5,786,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="titlescreen: the game's package" in=60 layout=294.65,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="titlescreen: the game's package"
                      69 TextBlock at=1192.5,835.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="miximage: missing" in=60 layout=176.37,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="miximage: missing"
                      70 TextBlock at=1192.5,885,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="backcover: missing" in=60 layout=180.91,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="backcover: missing"
                      71 TextBlock at=1192.5,934.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="3dbox: missing" in=60 layout=140.65,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="3dbox: missing"
                      72 TextBlock at=1192.5,984,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="physicalmedia: missing" in=60 layout=219.67,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text seen=1192.5,984,595.5,33 size=22 text="physicalmedia: missing"
            73 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              74 HintBar at=96,1074,1728,45 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1920,1200 role=group
focus 31
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
edge 43 down same
edge 43 left same
edge 43 right 12
press pageup top=ScraperWindow
press pagedown top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9)" top=ScraperWindow
