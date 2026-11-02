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
                7 TextBlock at=996,48,220,24 box=220,24 by=panel caps=13.999 drawn face=Barlow heard="3 games · 2 missing media" layout=219.66,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="3 games · 2 missing media"
                8 StackPanel at=64,40,920,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,920,39 box=920,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search games" in=13 layout=119.88,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search games"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,68,44 by=panel drawn focus heard=nes limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=nes
                    20 TextBlock at=154,99,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,91,68,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,91,66,44 by=panel drawn focus heard=psx limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=psx
                    22 TextBlock at=230,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=PSX layout=33.78,24 limit=214,91,66,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PSX
                  23 ToggleButton at=288,91,79,44 by=panel drawn focus heard=snes limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=snes
                    24 TextBlock at=304,99,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=288,91,79,44 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,147,691,530 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,147,691,530 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,147,691,530 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,158,691,519 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,519
                        31 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,158,691,54 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          32 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,168,643,33 in=30 paints=#707070FF role=none
                            33 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,168,643,32 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=30 paints=#4C9AE8FF role=none
                              34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,168,511,32 box=511,32 caps=18.1987 drawn ellipsis face=Barlow heard=Game in=30 layout=501.54,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                              35 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=647,168,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.08,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="2 of 4"
                        36 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,212,691,54 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=30 op paints=#D4D4D4FF role=listitem
                          37 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,222,643,33 in=30 paints=#707070FF role=none
                            38 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,222,643,32 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=30 paints=#4C9AE8FF role=none
                              39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,222,511,32 box=511,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=197.96,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="Amber Saga (USA)"
                              40 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=647,222,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=58.03,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="4 of 4"
                        41 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=64,266,691,54 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 op paints=#D4D4D4FF role=listitem
                          42 Border allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,276,643,33 in=30 paints=#707070FF role=none
                            43 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=88,276,643,32 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=30 paints=#4C9AE8FF role=none
                              44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=112,276,511,32 box=511,32 caps=18.1987 drawn face=Barlow heard=Game in=30 layout=273.65,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="Cobalt Harbor (Synthetic)"
                              45 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=647,276,60,32 box=60,32 caps=18.1987 drawn face=Barlow heard=Media in=30 layout=59.12,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="0 of 4"
                46 Border at=64,677,691,69 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  47 Grid at=68,681,683,61 by=panel limit=0,0,1280,800 role=none
                    48 Grid at=68,681,683,53 by=panel limit=0,0,1280,800 role=none
                      49 TextBlock at=76,681,675,53 box=675,53 by=panel caps=15.3989 drawn face=Barlow heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=625.86,52.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
              50 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                51 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  52 ScrollViewer at=803,179,381,535 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=803,179,381,535
                    53 StackPanel at=803,179,381,535 by=fixed-scroll in=52 limit=803,179,381,535 role=none
                      54 TextBlock at=803,179,381,116 box=381,116 by=fixed-scroll caps=22.3984 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=52 layout=365.18,115.2 limit=803,179,381,535 paints=#EEEEF0FF role=text size=32 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      55 TextBlock at=803,301,381,80 box=381,80 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=52 layout=376.4,79.2 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      56 TextBlock at=803,387,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="System: psx" in=52 layout=113.15,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="System: psx"
                      57 TextBlock at=803,420,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="cover: ScreenScraper" in=52 layout=201.76,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="cover: ScreenScraper"
                      58 TextBlock at=803,453,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="screenshot: missing" in=52 layout=188.21,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="screenshot: missing"
                      59 TextBlock at=803,486,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="marquee: missing" in=52 layout=167.42,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="marquee: missing"
                      60 TextBlock at=803,519,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="miximage: the game's package" in=52 layout=287.54,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="miximage: the game's package"
            61 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              62 HintBar at=96,757,1088,29 by=panel heard="Select, Back, System, List, Apply…, Mark" limit=0,0,1280,800 role=group
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
edge 31 down 36
edge 31 left same
edge 31 right 12
edge 36 up 31
edge 36 down 41
edge 36 left same
edge 36 right 12
edge 41 up 36
edge 41 down same
edge 41 left same
edge 41 right 12
press pageup top=ScraperWindow
press pagedown top=ScraperWindow
press first focus="TextBox Search games (Search)" top=ScraperWindow
press last focus="TextBox Search games (Search)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
