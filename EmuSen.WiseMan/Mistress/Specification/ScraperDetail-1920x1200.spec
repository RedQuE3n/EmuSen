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
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=1537.5,78,286.5,36 by=panel heard="3 games · 3 without art" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=1537.5,78,106.5,36 box=71,24 by=panel caps=13.999 drawn face=Barlow heard="3 games" layout=70.48,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 games"
                  9 TextBlock at=1644,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=1663.5,78,160.5,36 box=107,24 by=panel caps=13.999 drawn face=Barlow heard="3 without art" layout=106.08,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="3 without art"
                11 StackPanel at=96,66,1423.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  12 TextBlock at=96,66,1423.5,58.5 box=949,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            13 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              14 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                15 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  16 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    17 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search games" in=16 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              18 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                19 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  20 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    21 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=20 paints=#EEEEF0FF role=text size=20 text=ALL
                  22 SectionTab at=219,150,54,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=NES
                    23 TextBlock at=219,162,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=219,150,54,66 owner=22 paints=#D4D4D4FF role=text size=20 text=NES
                  24 SectionTab at=321,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    25 TextBlock at=321,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,150,171,66 owner=24 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  26 SectionTab at=540,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    27 TextBlock at=540,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=540,150,231,66 owner=26 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
            28 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              29 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                30 Border at=96,240,1036.5,609 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  31 Grid at=96,240,1036.5,609 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    32 LunaTable at=96,240,1036.5,609 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      33 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,609 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,609
                        34 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=33 op paints=#333337FF,#EEEEF0FF role=listitem
                          35 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=33 role=none
                            36 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,79.5 in=33 paints=#26262BFF,#4C9AE8FF role=none
                              37 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,240,1000.5,79.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9" in=33 role=none
                                38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,838.5,48 box=559,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=33 layout=551.98,31.2 owner=34 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                                39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,255,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=33 layout=58.5,31.2 owner=34 paints=#EEEEF0FF role=edit size=26 text="2 of 9"
                        40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 9" in=33 op paints=#333337FF,#D4D4D4FF role=listitem
                          41 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=33 role=none
                            42 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,79.5 in=33 paints=#0E0E10FF role=none
                              43 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,321,1000.5,79.5 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 9" in=33 role=none
                                44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,838.5,48 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=33 layout=197.96,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                                45 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,336,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=33 layout=58.45,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="4 of 9"
                        46 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=33 op paints=#333337FF,#D4D4D4FF role=listitem
                          47 DockPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=33 role=none
                            48 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,79.5 in=33 paints=#0E0E10FF role=none
                              49 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,402,1000.5,79.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 9" in=33 role=none
                                50 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,417,838.5,48 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=33 layout=273.65,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                                51 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,417,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=33 layout=59.54,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="0 of 9"
                52 Border at=96,849,1036.5,204 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  53 Grid at=96,876,1036.5,177 by=panel limit=0,0,1920,1200 role=none
                    54 Grid at=96,876,1036.5,85.5 by=panel limit=0,0,1920,1200 role=none
                      55 TextBlock at=96,882,1036.5,79.5 box=691,53 by=panel caps=15.3989 drawn face=Barlow heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=625.86,52.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
                    56 StackPanel at=877.5,987,255,66 by=panel limit=0,0,1920,1200 role=none
                      57 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        58 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          59 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          60 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=57 paints=#707070FF role=text size=26 text="Apply…"
              61 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                62 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  63 ScrollViewer at=1192.5,276,595.5,741 by=panel fades=b limit=0,0,1920,1200 overflow=57 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=1192.5,276,595.5,741
                    64 StackPanel at=1192.5,276,595.5,798 by=scroll in=63 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=none seen=1192.5,276,595.5,741
                      65 TextBlock at=1192.5,276,595.5,174 box=397,116 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=63 layout=365.18,115.2 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      66 TextBlock at=1192.5,459,595.5,120 box=397,80 by=scroll caps=15.3989 drawn face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=63 layout=376.4,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      67 TextBlock at=1192.5,588,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="Console: PlayStation" in=63 layout=190.92,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Console: PlayStation"
                      68 TextBlock at=1192.5,637.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="cover: ScreenScraper" in=63 layout=201.76,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      69 TextBlock at=1192.5,687,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="screenshot: missing" in=63 layout=188.21,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      70 TextBlock at=1192.5,736.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="marquee: missing" in=63 layout=167.42,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      71 TextBlock at=1192.5,786,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="titlescreen: the game's package" in=63 layout=294.65,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="titlescreen: the game's package"
                      72 TextBlock at=1192.5,835.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="miximage: missing" in=63 layout=176.37,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="miximage: missing"
                      73 TextBlock at=1192.5,885,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="backcover: missing" in=63 layout=180.91,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="backcover: missing"
                      74 TextBlock at=1192.5,934.5,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="3dbox: missing" in=63 layout=140.65,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="3dbox: missing"
                      75 TextBlock at=1192.5,984,595.5,40.5 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="physicalmedia: missing" in=63 layout=219.67,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text seen=1192.5,984,595.5,33 size=22 text="physicalmedia: missing"
            76 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              77 HintBar at=96,1074,1728,45 by=panel heard="Select, Back, Console, Mark, List, Apply…" limit=0,0,1920,1200 role=group
focus 34
edge 15 up same
edge 15 down 34
edge 15 left 26
edge 15 right same
edge 20 up same
edge 20 down 34
edge 20 left same
edge 20 right 22
edge 22 up same
edge 22 down 34
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 34
edge 24 left 22
edge 24 right 26
edge 26 up same
edge 26 down 34
edge 26 left 24
edge 26 right 15
edge 34 up 20
edge 34 down 40
edge 34 left same
edge 34 right 15
edge 40 up 34
edge 40 down 46
edge 40 left same
edge 40 right 15
edge 46 up 40
edge 46 down same
edge 46 left same
edge 46 right 15
press pageup top=ScraperWindow
press pagedown top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 9)" top=ScraperWindow
