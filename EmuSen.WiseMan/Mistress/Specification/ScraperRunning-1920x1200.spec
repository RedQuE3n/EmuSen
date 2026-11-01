# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,162,1219.05,837
title Scraper
case ScraperRunning
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,312,1221,657 by=panel limit=350.48,312,1219.05,657 paints=#9C9CA0FF role=none seen=350.48,312,1219.05,657
    1 ScraperPane at=386,330,1149,639 by=panel heard=Scraper limit=350.48,312,1219.05,657 role=pane
      2 Grid at=386,330,1149,639 by=panel limit=350.48,312,1219.05,657 role=none
        3 DockPanel at=410,348,1101,69 by=panel limit=350.48,312,1219.05,657 role=none
          4 Border at=1241,348,270,69 by=panel limit=350.48,312,1219.05,657 name=AdaptiveSearch role=none
            5 TextBox at=1241,348,270,69 by=panel drawn focus heard=Search limit=350.48,312,1219.05,657 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,358.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,358.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,348,831,69 by=panel limit=350.48,312,1219.05,657 name=AdaptiveNavigation role=none
            9 WholeRow at=410,348,831,69 by=panel limit=350.48,312,1219.05,657 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,349.5,193.5,66 by=panel drawn focus heard="All Games" limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=435.5,358.5,142.5,48 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=94.25,31.2 limit=410,349.5,193.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=615.5,349.5,208.5,66 by=panel drawn focus heard=Favourites limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=641,358.5,157.5,48 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=104.44,31.2 limit=615.5,349.5,208.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=836,349.5,246,66 by=panel drawn focus heard="Missing Media" limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=861.5,358.5,195,48 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=129.14,31.2 limit=836,349.5,246,66 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 role=none
          17 Border at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 name=AdaptiveContent role=none
            18 Grid at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 role=none
              19 LunaTable at=410,435,1101,253.5 by=panel heard=Rows limit=350.48,312,1219.05,657 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=428,444,129,43.5 box=86,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                21 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=548,441,9,48 focus heard="Resize Mark" paints=#9C9CA0FF role=thumb
                22 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=557,441,858,48 drawn focus heard="Game, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  23 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=557,441,75,48 role=none
                    24 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=557,441,75,48 box=50,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=49.89,31.2 owner=22 paints=#9C9CA0FF role=text size=26 text=Game
                25 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1406,441,9,48 focus heard="Resize Game" paints=#9C9CA0FF role=thumb
                26 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1415,441,78,48 drawn focus heard="Media, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  27 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1415,441,78,48 role=none
                    28 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1415,441,78,48 box=52,32 caps=18.1987 drawn face="Barlow Condensed" heard=Media layout=51.27,31.2 owner=26 paints=#9C9CA0FF role=text size=26 text=Media
                29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,499.5,1101,189 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=410,499.5,1101,189
                  30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,499.5,1101,63 drawn focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    31 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,507,1071,46.5 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 role=none
                      32 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,507,129,46.5 box=86,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ Scrape" in=29 layout=85.34,30.84 owner=30 paints=#EEEEF0FF role=text size=24 text="☑ Scrape"
                      33 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=554,508.5,864,43.5 box=576,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=29 layout=570.65,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1418,508.5,78,43.5 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=45.82,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="2 of 4"
                  35 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,562.5,1101,63 drawn focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    36 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,570,1071,46.5 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 role=none
                      37 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,570,129,46.5 box=86,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ Scrape" in=29 layout=85.34,30.84 owner=35 paints=#9C9CA0FF role=text size=24 text="☑ Scrape"
                      38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=554,571.5,864,43.5 box=576,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=143.54,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="Amber Saga (USA)"
                      39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1418,571.5,78,43.5 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=46.13,28.8 owner=35 paints=#9C9CA0FF role=edit size=24 text="4 of 4"
                  40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,625.5,1101,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#333337FF,#9C9CA0FF role=listitem
                    41 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,633,1071,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 role=none
                      42 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,633,129,46.5 box=86,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#9C9CA0FF role=text size=24 text="☐"
                      43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=554,634.5,864,43.5 box=576,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=203.04,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1418,634.5,78,43.5 box=52,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=46.82,28.8 owner=40 paints=#9C9CA0FF role=edit size=24 text="0 of 4"
        45 Border at=410,700.5,1101,66 by=panel limit=350.48,312,1219.05,657 name=AdaptiveActions role=none
          46 WrapPanel at=410,700.5,1101,66 by=panel limit=350.48,312,1219.05,657 role=none
            47 ActionButton at=410,700.5,177,66 by=panel drawn heard="MARK ALL" limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text="Mark All"
              48 TextBlock at=435.5,709.5,126,48 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=83.93,31.2 limit=410,700.5,177,66 off owner=47 paints=#55555AFF role=text size=26 text="MARK ALL"
            49 ActionButton at=605,700.5,139.5,66 by=panel drawn heard=THEME limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text=Theme
              50 TextBlock at=630.5,709.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=58.94,31.2 limit=605,700.5,139.5,66 off owner=49 paints=#55555AFF role=text size=26 text=THEME
            51 ActionButton at=762.5,700.5,145.5,66 by=panel drawn heard=VIDEOS limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text=Videos
              52 TextBlock at=788,709.5,94.5,48 box=63,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=62.74,31.2 limit=762.5,700.5,145.5,66 off owner=51 paints=#55555AFF role=text size=26 text=VIDEOS
            53 ActionButton at=926,700.5,204,66 by=panel drawn heard=OVERWRITE limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text=Overwrite
              54 TextBlock at=951.5,709.5,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=101.24,31.2 limit=926,700.5,204,66 off owner=53 paints=#55555AFF role=text size=26 text=OVERWRITE
            55 ActionButton at=1148,700.5,117,66 by=panel drawn focus heard=STOP limit=350.48,312,1219.05,657 op paints=#55555AFF,#9C9CA0FF role=button text=Stop
              56 TextBlock at=1173.5,709.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=1148,700.5,117,66 owner=55 paints=#9C9CA0FF role=text size=26 text=STOP
            57 ActionButton at=1283,700.5,123,66 by=panel drawn heard=MARK limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text=Mark
              58 TextBlock at=1308.5,709.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1283,700.5,123,66 off owner=57 paints=#55555AFF role=text size=26 text=MARK
        59 Border at=386,778.5,1149,133.5 by=panel limit=350.48,312,1219.05,657 name=AdaptiveStatus role=none
          60 Grid at=392,784.5,1137,121.5 by=panel limit=350.48,312,1219.05,657 role=none
            61 Grid at=392,784.5,1137,43.5 by=panel limit=350.48,312,1219.05,657 role=none
              62 ProgressBar at=392,802.5,180,6 by=panel drawn heard=Progress limit=350.48,312,1219.05,657 paints=#333337FF,#4C9AE8FF role=progressbar
              63 TextBlock at=584,784.5,945,43.5 box=630,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Scraping: 1 of 2 games" layout=182.59,28.8 limit=350.48,312,1219.05,657 paints=#9C9CA0FF role=text size=24 text="Scraping: 1 of 2 games"
            64 StackPanel at=1371.5,840,157.5,66 by=panel limit=350.48,312,1219.05,657 role=none
              65 Button at=1371.5,840,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,312,1219.05,657 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                66 TextBlock at=1397,849,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=1371.5,840,157.5,66 off owner=65 paints=#55555AFF role=text size=26 text="APPLY…"
        67 HintBar at=410,924,1101,33 by=panel heard="Open, Back, Section, Apply…, Mark" limit=350.48,312,1219.05,657 role=group
focus 5
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
edge 21 left 10
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
edge 30 right 5
edge 35 up 30
edge 35 down 40
edge 35 left same
edge 35 right 55
edge 40 up 35
edge 40 down 55
edge 40 left same
edge 40 right 55
edge 55 up 40
edge 55 down same
edge 55 left 14
edge 55 right 26
press pageup focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press first focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press last focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
