# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case ScraperRunning
entered down
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
            9 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1200 role=none
              10 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                11 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=12 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              14 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                15 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 role=none
                  16 ToggleButton at=96,138,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    17 TextBlock at=120,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,138,99,66 owner=16 paints=#EEEEF0FF role=text size=20 text=ALL
                  18 ToggleButton at=207,138,102,66 by=panel drawn focus heard=nes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=nes
                    19 TextBlock at=231,150,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=207,138,102,66 owner=18 paints=#D4D4D4FF role=text size=20 text=NES
                  20 ToggleButton at=321,138,99,66 by=panel drawn focus heard=psx limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=psx
                    21 TextBlock at=345,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=PSX layout=33.78,24 limit=321,138,99,66 owner=20 paints=#D4D4D4FF role=text size=20 text=PSX
                  22 ToggleButton at=432,138,118.5,66 by=panel drawn focus heard=snes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=snes
                    23 TextBlock at=456,150,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=432,138,118.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text=SNES
            24 Grid at=96,223.5,1728,814.5 by=panel limit=0,0,1920,1200 role=none
              25 Grid at=96,223.5,1036.5,814.5 by=panel limit=0,0,1920,1200 role=none
                26 Border at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  27 Grid at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 role=none
                    28 LunaTable at=96,223.5,1036.5,747 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,730.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,730.5
                        30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,240,1036.5,81 drawn focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          31 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,964.5,51 in=29 paints=#707070FF role=none
                            32 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,964.5,49.5 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 role=none
                              33 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,255,162,49.5 box=108,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ Scrape" in=29 layout=107.45,32.27 owner=30 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,255,676.5,48 box=451,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=29 layout=446.34,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,255,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.08,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        36 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,321,1036.5,81 drawn focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          37 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,964.5,51 in=29 paints=#707070FF role=none
                            38 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,964.5,49.5 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 role=none
                              39 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,336,162,49.5 box=108,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ Scrape" in=29 layout=107.45,32.27 owner=36 paints=#4C9AE8FF role=text size=26 text="✓ Scrape"
                              40 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,336,676.5,48 box=451,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=197.96,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              41 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,336,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.03,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        42 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,402,1036.5,81 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          43 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,417,964.5,49.5 in=29 paints=#707070FF role=none
                            44 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,417,964.5,48 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 role=none
                              45 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=132,417,162,48 drawn in=29 owner=42 paints=#4C9AE8FF role=text
                              46 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=330,417,676.5,48 box=451,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=273.65,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1006.5,417,90,48 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=59.12,31.2 owner=42 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                48 Border at=96,970.5,1036.5,67.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  49 Grid at=102,976.5,1024.5,55.5 by=panel limit=0,0,1920,1200 role=none
                    50 Grid at=102,976.5,1024.5,43.5 by=panel limit=0,0,1920,1200 role=none
                      51 ProgressBar at=102,994.5,180,6 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                      52 TextBlock at=294,976.5,832.5,43.5 box=555,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Scraping: 1 of 2 games" layout=174.98,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="Scraping: 1 of 2 games"
              53 Border at=1156.5,223.5,667.5,814.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                54 Border at=1192.5,259.5,595.5,742.5 by=panel limit=0,0,1920,1200 role=none
                  55 ScrollViewer at=1204.5,271.5,571.5,718.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,271.5,571.5,718.5
                    56 StackPanel at=1204.5,271.5,571.5,718.5 by=fixed-scroll in=55 limit=1204.5,271.5,571.5,718.5 role=none
                      57 TextBlock at=1204.5,271.5,571.5,87 box=381,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=55 layout=361.46,57.6 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      58 TextBlock at=1204.5,367.5,571.5,130.5 box=381,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=55 layout=344.14,86.4 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      59 TextBlock at=1204.5,507,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=55 layout=93.91,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      60 TextBlock at=1204.5,559.5,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=55 layout=166.8,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                      61 TextBlock at=1204.5,612,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=55 layout=155.74,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                      62 TextBlock at=1204.5,664.5,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=55 layout=139.46,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                      63 TextBlock at=1204.5,717,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=55 layout=241.15,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="miximage: the game's package"
            64 Border at=96,1050,1728,66 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              65 WrapPanel at=96,1050,1728,66 by=panel limit=0,0,1920,1200 role=none
                66 ActionButton at=96,1050,115.5,66 by=panel drawn focus heard=STOP limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                  67 TextBlock at=121.5,1059,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1050,115.5,66 owner=66 paints=#D4D4D4FF role=text size=26 text=STOP
            68 HintBar at=96,1140,1728,36 by=panel heard="Select, Back, System, List, Apply…, Mark" limit=0,0,1920,1200 role=group
focus 11
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
edge 30 down 36
edge 30 left same
edge 30 right 11
edge 36 up 30
edge 36 down 42
edge 36 left same
edge 36 right 11
edge 42 up 36
edge 42 down 66
edge 42 left same
edge 42 right 11
edge 66 up 42
edge 66 down same
edge 66 left same
edge 66 right 11
press pageup focus="TextBox Search games (Search)" top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
