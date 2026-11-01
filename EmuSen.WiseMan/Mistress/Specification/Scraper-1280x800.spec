# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Scraper-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,108,812.7,558
title Scraper
case Scraper
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,208,813,438 by=panel limit=234,208,812.35,438 paints=#9C9CA0FF role=none seen=234,208,812.35,438
    1 ScraperPane at=258,220,765,426 by=panel heard=Scraper limit=234,208,812.35,438 role=pane
      2 Grid at=258,220,765,426 by=panel limit=234,208,812.35,438 role=none
        3 DockPanel at=274,232,733,46 by=panel limit=234,208,812.35,438 role=none
          4 Border at=827,232,180,46 by=panel limit=234,208,812.35,438 name=AdaptiveSearch role=none
            5 TextBox at=827,232,180,46 by=panel drawn focus heard=Search limit=234,208,812.35,438 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,239,158,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,239,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=274,232,553,46 by=panel limit=234,208,812.35,438 name=AdaptiveNavigation role=none
            9 WholeRow at=274,232,553,46 by=panel limit=234,208,812.35,438 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,233,129,44 by=panel drawn focus heard="All Games" limit=234,208,812.35,438 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=291,239,95,32 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=94.25,31.2 limit=274,233,129,44 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=411,233,139,44 by=panel drawn focus heard=Favourites limit=234,208,812.35,438 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=428,239,105,32 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=104.44,31.2 limit=411,233,139,44 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=558,233,164,44 by=panel drawn focus heard="Missing Media" limit=234,208,812.35,438 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=575,239,130,32 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=129.14,31.2 limit=558,233,164,44 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=274,290,733,169 by=panel limit=234,208,812.35,438 role=none
          17 Border at=274,290,733,169 by=panel limit=234,208,812.35,438 name=AdaptiveContent role=none
            18 Grid at=274,290,733,169 by=panel limit=234,208,812.35,438 role=none
              19 LunaTable at=274,290,733,169 by=panel heard=Rows limit=234,208,812.35,438 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=286,296,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                21 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=320,294,6,32 focus heard="Resize Mark" paints=#9C9CA0FF role=thumb
                22 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=326,294,617,32 drawn focus heard="Game, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  23 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=326,294,50,32 role=none
                    24 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=326,294,50,32 box=50,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=49.89,31.2 owner=22 paints=#9C9CA0FF role=text size=26 text=Game
                25 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=937,294,6,32 focus heard="Resize Game" paints=#9C9CA0FF role=thumb
                26 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=943,294,52,32 drawn focus heard="Media, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  27 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=943,294,52,32 role=none
                    28 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=943,294,52,32 box=52,32 caps=18.1987 drawn face="Barlow Condensed" heard=Media layout=51.27,31.2 owner=26 paints=#9C9CA0FF role=text size=26 text=Media
                29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=274,333,733,126 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=274,333,733,126
                  30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=274,333,733,42 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,338,713,31 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 role=none
                      32 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,338,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=30 paints=#EEEEF0FF role=text size=24 text="☐"
                      33 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=324,339,621,29 box=621,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=29 layout=618.34,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=945,339,52,29 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=45.82,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="2 of 4"
                  35 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=274,375,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    36 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,380,713,31 heard="Mark: not marked, Game: Amber Saga (USA), Media: 4 of 4" in=29 role=none
                      37 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,380,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=35 paints=#9C9CA0FF role=text size=24 text="☐"
                      38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=324,381,621,29 box=621,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=143.54,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="Amber Saga (USA)"
                      39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=945,381,52,29 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=46.13,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="4 of 4"
                  40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=274,417,733,42 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    41 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,422,713,31 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 role=none
                      42 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=284,422,40,31 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#9C9CA0FF role=text size=24 text="☐"
                      43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=324,423,621,29 box=621,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=203.04,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=945,423,52,29 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=46.82,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="0 of 4"
        45 Border at=274,467,733,44 by=panel limit=234,208,812.35,438 name=AdaptiveActions role=none
          46 WrapPanel at=274,467,733,44 by=panel limit=234,208,812.35,438 role=none
            47 ActionButton at=274,467,118,44 by=panel drawn focus heard="MARK ALL" limit=234,208,812.35,438 op paints=#55555AFF,#9C9CA0FF role=button text="Mark All"
              48 TextBlock at=291,473,84,32 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=83.93,31.2 limit=274,467,118,44 owner=47 paints=#9C9CA0FF role=text size=26 text="MARK ALL"
            49 ActionButton at=404,467,93,44 by=panel drawn focus heard=THEME limit=234,208,812.35,438 op paints=#55555AFF,#9C9CA0FF role=button text=Theme
              50 TextBlock at=421,473,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=58.94,31.2 limit=404,467,93,44 owner=49 paints=#9C9CA0FF role=text size=26 text=THEME
            51 ActionButton at=509,467,97,44 by=panel drawn focus heard=VIDEOS limit=234,208,812.35,438 op paints=#55555AFF,#9C9CA0FF role=button text=Videos
              52 TextBlock at=526,473,63,32 box=63,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=62.74,31.2 limit=509,467,97,44 owner=51 paints=#9C9CA0FF role=text size=26 text=VIDEOS
            53 ActionButton at=618,467,136,44 by=panel drawn focus heard=OVERWRITE limit=234,208,812.35,438 op paints=#55555AFF,#9C9CA0FF role=button text=Overwrite
              54 TextBlock at=635,473,102,32 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=101.24,31.2 limit=618,467,136,44 owner=53 paints=#9C9CA0FF role=text size=26 text=OVERWRITE
            55 ActionButton at=766,467,78,44 by=panel drawn heard=STOP limit=234,208,812.35,438 off paints=#55555AFF,#9C9CA0FF role=button text=Stop
              56 TextBlock at=783,473,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=766,467,78,44 off owner=55 paints=#55555AFF role=text size=26 text=STOP
            57 ActionButton at=856,467,82,44 by=panel drawn focus heard=MARK limit=234,208,812.35,438 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              58 TextBlock at=873,473,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=856,467,82,44 owner=57 paints=#9C9CA0FF role=text size=26 text=MARK
        59 Border at=258,519,765,89 by=panel limit=234,208,812.35,438 name=AdaptiveStatus role=none
          60 Grid at=262,523,757,81 by=panel limit=234,208,812.35,438 role=none
            61 Grid at=262,523,757,29 by=panel limit=234,208,812.35,438 role=none
              62 TextBlock at=270,523,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=639.6,28.8 limit=234,208,812.35,438 paints=#9C9CA0FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
            63 StackPanel at=914,560,105,44 by=panel limit=234,208,812.35,438 role=none
              64 Button at=914,560,105,44 by=panel drawn heard="Apply the marked changes" limit=234,208,812.35,438 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                65 TextBlock at=931,566,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=914,560,105,44 off owner=64 paints=#55555AFF role=text size=26 text="APPLY…"
        66 HintBar at=274,616,733,22 by=panel heard="Open, Back, Section, Apply…, Mark" limit=234,208,812.35,438 role=group
focus 21
edge 5 up same
edge 5 down 22
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 21
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 22
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 22
edge 14 left 12
edge 14 right 5
edge 21 up 10
edge 21 down 30
edge 21 left same
edge 21 right 22
edge 22 up 5
edge 22 down 30
edge 22 left 21
edge 22 right 26
edge 26 up 5
edge 26 down 30
edge 26 left 22
edge 26 right same
edge 30 up 21
edge 30 down 35
edge 30 left same
edge 30 right same
edge 35 up 30
edge 35 down 40
edge 35 left same
edge 35 right same
edge 40 up 35
edge 40 down 47
edge 40 left same
edge 40 right same
edge 47 up 40
edge 47 down same
edge 47 left same
edge 47 right 49
edge 49 up 40
edge 49 down same
edge 49 left 47
edge 49 right 51
edge 51 up 40
edge 51 down same
edge 51 left 49
edge 51 right 53
edge 53 up 40
edge 53 down same
edge 53 left 51
edge 53 right 57
edge 57 up 40
edge 57 down same
edge 57 left 53
edge 57 right 25
press pageup focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
press pagedown focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
press first focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
press last focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
press back top=none
