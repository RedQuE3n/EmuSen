# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case ScraperDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 ScraperPane at=48,28,1184,764 by=panel heard=Scraper limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=SCRAPER limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 StackPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                  8 TextBlock at=64,40,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,800 role=none
              10 Border at=1036,91,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                11 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=12 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              14 Border at=64,91,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                15 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,800 role=none
                  16 ToggleButton at=64,92,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    17 TextBlock at=80,100,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,92,66,44 owner=16 paints=#EEEEF0FF role=text size=20 text=ALL
                  18 ToggleButton at=138,92,68,44 by=panel drawn focus heard=nes limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=nes
                    19 TextBlock at=154,100,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,92,68,44 owner=18 paints=#D4D4D4FF role=text size=20 text=NES
                  20 ToggleButton at=214,92,66,44 by=panel drawn focus heard=psx limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=psx
                    21 TextBlock at=230,100,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=PSX layout=33.78,24 limit=214,92,66,44 owner=20 paints=#D4D4D4FF role=text size=20 text=PSX
                  22 ToggleButton at=288,92,79,44 by=panel drawn focus heard=snes limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=snes
                    23 TextBlock at=304,100,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=288,92,79,44 owner=22 paints=#D4D4D4FF role=text size=20 text=SNES
            24 Grid at=64,149,1152,543 by=panel limit=0,0,1280,800 role=none
              25 Grid at=64,149,691,543 by=panel limit=0,0,1280,800 role=none
                26 Border at=64,149,691,498 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  27 Grid at=64,149,691,498 by=panel limit=0,0,1280,800 role=none
                    28 LunaTable at=64,149,691,498 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,487 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,487
                        30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,160,691,54 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          31 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,170,643,33 in=29 paints=#707070FF role=none
                            32 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,170,643,32 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 paints=#4C9AE8FF role=none
                              33 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,170,559,32 box=559,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=29 layout=551.98,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,170,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.08,31.2 owner=30 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        35 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,214,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          36 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,224,643,33 in=29 paints=#707070FF role=none
                            37 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,224,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 paints=#4C9AE8FF role=none
                              38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,224,559,32 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=197.96,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,224,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=58.03,31.2 owner=35 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,268,691,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#D4D4D4FF role=listitem
                          41 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,278,643,33 in=29 paints=#707070FF role=none
                            42 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,278,643,32 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 paints=#4C9AE8FF role=none
                              43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,278,559,32 box=559,32 caps=18.1987 drawn face=Barlow heard=Game in=29 layout=273.65,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=671,278,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=29 layout=59.12,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                45 Border at=64,647,691,45 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  46 Grid at=68,651,683,37 by=panel limit=0,0,1280,800 role=none
                    47 Grid at=68,651,683,29 by=panel limit=0,0,1280,800 role=none
                      48 TextBlock at=76,651,675,29 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=613.94,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
              49 Border at=771,149,445,543 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                50 Border at=795,173,397,495 by=panel limit=0,0,1280,800 role=none
                  51 ScrollViewer at=803,181,381,479 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=803,181,381,479
                    52 StackPanel at=803,181,381,479 by=fixed-scroll in=51 limit=803,181,381,479 role=none
                      53 TextBlock at=803,181,381,58 box=381,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=51 layout=361.46,57.6 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      54 TextBlock at=803,245,381,87 box=381,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=51 layout=344.14,86.4 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      55 TextBlock at=803,338,381,29 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=51 layout=93.91,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      56 TextBlock at=803,373,381,29 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=51 layout=166.8,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                      57 TextBlock at=803,408,381,29 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=51 layout=155.74,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                      58 TextBlock at=803,443,381,29 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=51 layout=139.46,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                      59 TextBlock at=803,478,381,29 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=51 layout=241.15,28.8 limit=803,181,381,479 paints=#D4D4D4FF role=text size=24 text="miximage: the game's package"
            60 Border at=64,700,1152,44 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              61 WrapPanel at=64,700,1152,44 by=panel limit=0,0,1280,800 role=none
                62 ActionButton at=64,700,77,44 by=panel drawn heard=STOP limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  63 TextBlock at=81,706,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=64,700,77,44 off owner=62 paints=#707070FF role=text size=26 text=STOP
            64 HintBar at=64,760,1152,24 by=panel heard="Select, Back, System, List, Apply…, Mark" limit=0,0,1280,800 role=group
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
