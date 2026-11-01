# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Scraper-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case Scraper
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 ScraperPane at=72,42,1776,1107 by=panel heard=Scraper limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=SCRAPER limit=0,0,1920,1161 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                7 StackPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                  8 TextBlock at=96,60,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1161 role=none
              10 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                11 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=12 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              14 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                15 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 role=none
                  16 ToggleButton at=96,138,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    17 TextBlock at=121.5,147,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,138,97.5,66 owner=16 paints=#EEEEF0FF role=text size=26 text=ALL
                  18 ToggleButton at=205.5,138,100.5,66 by=panel drawn focus heard=nes limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=nes
                    19 TextBlock at=231,147,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=205.5,138,100.5,66 owner=18 paints=#FFFFFFFF role=text size=26 text=NES
                  20 ToggleButton at=318,138,99,66 by=panel drawn focus heard=psx limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=psx
                    21 TextBlock at=343.5,147,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PSX layout=31.04,31.2 limit=318,138,99,66 owner=20 paints=#FFFFFFFF role=text size=26 text=PSX
                  22 ToggleButton at=429,138,115.5,66 by=panel drawn focus heard=snes limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=snes
                    23 TextBlock at=454.5,147,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=429,138,115.5,66 owner=22 paints=#FFFFFFFF role=text size=26 text=SNES
            24 Grid at=96,223.5,1728,778.5 by=panel limit=0,0,1920,1161 role=none
              25 Grid at=96,223.5,1036.5,778.5 by=panel limit=0,0,1920,1161 role=none
                26 Border at=96,223.5,1036.5,645 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                  27 Grid at=96,223.5,1036.5,645 by=panel limit=0,0,1920,1161 role=none
                    28 LunaTable at=96,223.5,1036.5,645 by=panel heard=Rows limit=0,0,1920,1161 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      29 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=114,232.5,57,43.5 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                      30 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=162,229.5,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                      31 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=171,229.5,870,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                        32 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=171,229.5,72,48 role=none
                          33 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=171,229.5,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=31 paints=#D4D4D4FF role=text size=26 text=Game
                      34 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1032,229.5,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                      35 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1041,229.5,73.5,48 drawn focus heard="Media, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                        36 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1041,229.5,73.5,48 role=none
                          37 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1041,229.5,73.5,48 box=49,32 caps=18.1987 drawn face="Barlow Condensed" heard=Media layout=48.72,31.2 owner=35 paints=#D4D4D4FF role=text size=26 text=Media
                      38 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,288,1036.5,580.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,288,1036.5,580.5
                        39 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,288,1036.5,63 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                          40 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,295.5,1006.5,46.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=38 role=none
                            41 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,295.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=39 paints=#EEEEF0FF role=text size=24 text="☐"
                            42 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,297,876,43.5 box=584,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=38 layout=576,28.8 owner=39 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                            43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1044,297,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=38 layout=43.39,28.8 owner=39 paints=#EEEEF0FF role=edit size=24 text="2 of 4"
                        44 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,351,1036.5,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                          45 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,358.5,1006.5,46.5 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=38 role=none
                            46 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,358.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=44 paints=#D4D4D4FF role=text size=24 text="☐"
                            47 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,360,876,43.5 box=584,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=139.25,28.8 owner=44 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                            48 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1044,360,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=38 layout=43.94,28.8 owner=44 paints=#D4D4D4FF role=edit size=24 text="4 of 4"
                        49 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=96,414,1036.5,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                          50 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,421.5,1006.5,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=38 role=none
                            51 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=111,421.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=49 paints=#D4D4D4FF role=text size=24 text="☐"
                            52 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=168,423,876,43.5 box=584,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=192.29,28.8 owner=49 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                            53 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1044,423,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=38 layout=44.57,28.8 owner=49 paints=#D4D4D4FF role=edit size=24 text="0 of 4"
                54 Border at=96,868.5,1036.5,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
                  55 Grid at=102,874.5,1024.5,121.5 by=panel limit=0,0,1920,1161 role=none
                    56 Grid at=102,874.5,1024.5,43.5 by=panel limit=0,0,1920,1161 role=none
                      57 TextBlock at=114,874.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=613.94,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
                    58 StackPanel at=973.5,930,153,66 by=panel limit=0,0,1920,1161 role=none
                      59 Button at=973.5,930,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        60 TextBlock at=999,939,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=973.5,930,153,66 off owner=59 paints=#707070FF role=text size=26 text="APPLY…"
              61 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                62 Border at=1156.5,223.5,667.5,778.5 by=panel limit=0,0,1920,1161 role=none
                  63 ScrollViewer at=1168.5,235.5,643.5,754.5 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1168.5,235.5,643.5,754.5
                    64 StackPanel at=1168.5,235.5,643.5,754.5 by=fixed-scroll in=63 limit=1168.5,235.5,643.5,754.5 role=none
                      65 TextBlock at=1168.5,235.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=63 layout=361.46,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      66 TextBlock at=1168.5,331.5,643.5,87 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=63 layout=415.39,57.6 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      67 TextBlock at=1168.5,427.5,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=63 layout=93.91,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      68 TextBlock at=1168.5,480,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=63 layout=166.8,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                      69 TextBlock at=1168.5,532.5,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=63 layout=155.74,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                      70 TextBlock at=1168.5,585,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=63 layout=139.46,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                      71 TextBlock at=1168.5,637.5,643.5,43.5 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=63 layout=241.15,28.8 limit=1168.5,235.5,643.5,754.5 paints=#D4D4D4FF role=text size=24 text="miximage: the game's package"
            72 Border at=96,1014,1728,66 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              73 WrapPanel at=96,1014,1728,66 by=panel limit=0,0,1920,1161 role=none
                74 ActionButton at=96,1014,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  75 TextBlock at=121.5,1023,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1014,115.5,66 off owner=74 paints=#707070FF role=text size=26 text=STOP
            76 HintBar at=96,1104,1728,33 by=panel heard="Select, Back, Section, List, Apply…, Mark" limit=0,0,1920,1161 role=group
focus 30
edge 11 up same
edge 11 down 35
edge 11 left 22
edge 11 right same
edge 16 up same
edge 16 down 30
edge 16 left same
edge 16 right 18
edge 18 up same
edge 18 down 31
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 31
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 31
edge 22 left 20
edge 22 right 11
edge 30 up 16
edge 30 down 39
edge 30 left 16
edge 30 right 31
edge 31 up 18
edge 31 down 39
edge 31 left 30
edge 31 right 35
edge 35 up 11
edge 35 down 39
edge 35 left 31
edge 35 right 11
edge 39 up 30
edge 39 down 44
edge 39 left same
edge 39 right 11
edge 44 up 39
edge 44 down 49
edge 44 left same
edge 44 right 11
edge 49 up 44
edge 49 down same
edge 49 left same
edge 49 right 11
press pageup focus="GridSplitter  (Resize Mark)" top=ScraperWindow
press pagedown focus="GridSplitter  (Resize Mark)" top=ScraperWindow
press first focus="GridSplitter  (Resize Mark)" top=ScraperWindow
press last focus="GridSplitter  (Resize Mark)" top=ScraperWindow
press back top=none
