# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Scraper-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Scraper
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
                  8 TextBlock at=96,60,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              10 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                11 TextBox at=1554,136.5,270,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 drawn face=Barlow heard="Search games" in=12 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              14 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                15 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  16 ToggleButton at=96,136.5,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    17 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=16 paints=#EEEEF0FF role=text size=20 text=ALL
                  18 ToggleButton at=207,136.5,102,66 by=panel drawn focus heard=nes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=nes
                    19 TextBlock at=231,148.5,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=207,136.5,102,66 owner=18 paints=#D4D4D4FF role=text size=20 text=NES
                  20 ToggleButton at=321,136.5,99,66 by=panel drawn focus heard=psx limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=psx
                    21 TextBlock at=345,148.5,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=PSX layout=33.78,24 limit=321,136.5,99,66 owner=20 paints=#D4D4D4FF role=text size=20 text=PSX
                  22 ToggleButton at=432,136.5,118.5,66 by=panel drawn focus heard=snes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=snes
                    23 TextBlock at=456,148.5,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=432,136.5,118.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text=SNES
            24 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              25 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                26 Border at=96,220.5,1036.5,795 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  27 Grid at=96,220.5,1036.5,795 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    28 LunaTable at=96,220.5,1036.5,795 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,778.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,778.5
                        30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,237,1036.5,81 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          31 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,49.5 in=29 paints=#707070FF role=none
                            32 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,252,964.5,48 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 paints=#4C9AE8FF role=none
                              33 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,252,766.5,48 box=511,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=29 layout=501.54,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,252,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.08,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        35 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,318,1036.5,81 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,49.5 in=29 paints=#707070FF role=none
                            37 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,333,964.5,48 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 paints=#4C9AE8FF role=none
                              38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,333,766.5,48 box=511,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,333,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.03,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,399,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          41 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,49.5 in=29 paints=#707070FF role=none
                            42 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,414,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 paints=#4C9AE8FF role=none
                              43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,414,766.5,48 box=511,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=273.65,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=970.5,414,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=59.12,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                45 Border at=96,1015.5,1036.5,103.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  46 Grid at=102,1021.5,1024.5,91.5 by=panel limit=0,0,1920,1200 role=none
                    47 Grid at=102,1021.5,1024.5,79.5 by=panel limit=0,0,1920,1200 role=none
                      48 TextBlock at=114,1021.5,1012.5,79.5 box=675,53 by=panel caps=15.3989 drawn face=Barlow heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=625.86,52.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
              49 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                50 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  51 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    52 StackPanel at=1204.5,268.5,571.5,802.5 by=fixed-scroll in=51 limit=1204.5,268.5,571.5,802.5 role=none
                      53 TextBlock at=1204.5,268.5,571.5,120 box=381,80 by=fixed-scroll caps=16.0864 drawn face="Barlow Medium" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=51 layout=333.56,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      54 TextBlock at=1204.5,397.5,571.5,120 box=381,80 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=51 layout=376.4,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      55 TextBlock at=1204.5,526.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=51 layout=113.15,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      56 TextBlock at=1204.5,576,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="cover: ScreenScraper" in=51 layout=201.76,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      57 TextBlock at=1204.5,625.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="screenshot: missing" in=51 layout=188.21,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      58 TextBlock at=1204.5,675,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="marquee: missing" in=51 layout=167.42,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      59 TextBlock at=1204.5,724.5,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="miximage: the game's package" in=51 layout=287.54,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="miximage: the game's package"
            60 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              61 HintBar at=144,1135.5,1632,43.5 by=panel heard="Select, Back, System, List, Apply…, Mark" limit=0,0,1920,1200 role=group
focus 30
edge 11 up same
edge 11 down 30
edge 11 left 22
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
edge 30 down 35
edge 30 left same
edge 30 right 11
edge 35 up 30
edge 35 down 40
edge 35 left same
edge 35 right 11
edge 40 up 35
edge 40 down same
edge 40 left same
edge 40 right 11
press pageup top=ScraperWindow
press pagedown top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
