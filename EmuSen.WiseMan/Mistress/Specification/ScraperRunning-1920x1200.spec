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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,312,1221,657 by=panel limit=350.48,312,1219.05,657 paints=#D4D4D4FF role=none seen=350.48,312,1219.05,657
    1 ScraperPane at=386,330,1149,639 by=panel heard=Scraper limit=350.48,312,1219.05,657 role=pane
      2 Grid at=386,330,1149,639 by=panel limit=350.48,312,1219.05,657 role=none
        3 DockPanel at=410,348,1101,69 by=panel limit=350.48,312,1219.05,657 role=none
          4 Border at=1241,348,270,69 by=panel limit=350.48,312,1219.05,657 name=AdaptiveSearch role=none
            5 TextBox at=1241,348,270,69 by=panel drawn focus heard=Search limit=350.48,312,1219.05,657 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,358.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,358.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=410,348,831,69 by=panel limit=350.48,312,1219.05,657 name=AdaptiveNavigation role=none
            9 WholeRow at=410,348,831,69 by=panel limit=350.48,312,1219.05,657 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,349.5,189,66 by=panel drawn focus heard="All Games" limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=435.5,358.5,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=91.44,31.2 limit=410,349.5,189,66 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=611,349.5,199.5,66 by=panel drawn focus heard=Favourites limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=636.5,358.5,148.5,48 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=98.1,31.2 limit=611,349.5,199.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=822.5,349.5,234,66 by=panel drawn focus heard="Missing Media" limit=350.48,312,1219.05,657 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=848,358.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=121.71,31.2 limit=822.5,349.5,234,66 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 role=none
          17 Border at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 name=AdaptiveContent role=none
            18 Grid at=410,435,1101,253.5 by=panel limit=350.48,312,1219.05,657 role=none
              19 LunaTable at=410,435,1101,253.5 by=panel heard=Rows limit=350.48,312,1219.05,657 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=428,444,124.5,43.5 box=83,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                21 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=543.5,441,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                22 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=552.5,441,867,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  23 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=552.5,441,72,48 role=none
                    24 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=552.5,441,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=22 paints=#D4D4D4FF role=text size=26 text=Game
                25 GridSplitter allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1410.5,441,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                26 Button allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1419.5,441,73.5,48 drawn focus heard="Media, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  27 StackPanel allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1419.5,441,73.5,48 role=none
                    28 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1419.5,441,73.5,48 box=49,32 caps=18.1987 drawn face="Barlow Condensed" heard=Media layout=48.72,31.2 owner=26 paints=#D4D4D4FF role=text size=26 text=Media
                29 ScrollViewer allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,499.5,1101,189 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=410,499.5,1101,189
                  30 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,499.5,1101,63 drawn focus heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 op paints=#333337FF,#D4D4D4FF role=listitem
                    31 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,507,1071,46.5 heard="Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4" in=29 role=none
                      32 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,507,124.5,46.5 box=83,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ Scrape" in=29 layout=82.9,30.84 owner=30 paints=#EEEEF0FF role=text size=24 text="☑ Scrape"
                      33 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=549.5,508.5,873,43.5 box=582,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=29 layout=576,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      34 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1422.5,508.5,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=43.39,28.8 owner=30 paints=#EEEEF0FF role=edit size=24 text="2 of 4"
                  35 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,562.5,1101,63 drawn focus heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 op paints=#333337FF,#D4D4D4FF role=listitem
                    36 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,570,1071,46.5 heard="Mark: Scrape, Game: Amber Saga (USA), Media: 4 of 4" in=29 role=none
                      37 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,570,124.5,46.5 box=83,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ Scrape" in=29 layout=82.9,30.84 owner=35 paints=#D4D4D4FF role=text size=24 text="☑ Scrape"
                      38 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=549.5,571.5,873,43.5 box=582,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=139.25,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                      39 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1422.5,571.5,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=43.94,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text="4 of 4"
                  40 ListBoxItem allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=410,625.5,1101,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 op paints=#333337FF,#D4D4D4FF role=listitem
                    41 Grid allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,633,1071,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), Media: 0 of 4" in=29 role=none
                      42 TextBlock allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=425,633,124.5,46.5 box=83,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#D4D4D4FF role=text size=24 text="☐"
                      43 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=549.5,634.5,873,43.5 box=582,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=29 layout=192.29,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      44 TableCell allow="a game's title or missing kinds cut in the Scraper table are whole in the detail while its row is chosen" at=1422.5,634.5,73.5,43.5 box=49,29 caps=16.7988 drawn face="Barlow Condensed" heard=Media in=29 layout=44.57,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text="0 of 4"
        45 Border at=410,700.5,1101,66 by=panel limit=350.48,312,1219.05,657 name=AdaptiveActions role=none
          46 WrapPanel at=410,700.5,1101,66 by=panel limit=350.48,312,1219.05,657 role=none
            47 ActionButton at=410,700.5,175.5,66 by=panel drawn heard="MARK ALL" limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text="Mark All"
              48 TextBlock at=435.5,709.5,124.5,48 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=82.13,31.2 limit=410,700.5,175.5,66 off owner=47 paints=#707070FF role=text size=26 text="MARK ALL"
            49 ActionButton at=603.5,700.5,135,66 by=panel drawn heard=THEME limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text=Theme
              50 TextBlock at=629,709.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=55.04,31.2 limit=603.5,700.5,135,66 off owner=49 paints=#707070FF role=text size=26 text=THEME
            51 ActionButton at=756.5,700.5,139.5,66 by=panel drawn heard=VIDEOS limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text=Videos
              52 TextBlock at=782,709.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=58.6,31.2 limit=756.5,700.5,139.5,66 off owner=51 paints=#707070FF role=text size=26 text=VIDEOS
            53 ActionButton at=914,700.5,195,66 by=panel drawn heard=OVERWRITE limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text=Overwrite
              54 TextBlock at=939.5,709.5,144,48 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=95,31.2 limit=914,700.5,195,66 off owner=53 paints=#707070FF role=text size=26 text=OVERWRITE
            55 ActionButton at=1127,700.5,115.5,66 by=panel drawn focus heard=STOP limit=350.48,312,1219.05,657 op paints=#707070FF,#D4D4D4FF role=button text=Stop
              56 TextBlock at=1152.5,709.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=1127,700.5,115.5,66 owner=55 paints=#D4D4D4FF role=text size=26 text=STOP
            57 ActionButton at=1260.5,700.5,120,66 by=panel drawn heard=MARK limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text=Mark
              58 TextBlock at=1286,709.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1260.5,700.5,120,66 off owner=57 paints=#707070FF role=text size=26 text=MARK
        59 Border at=386,778.5,1149,133.5 by=panel limit=350.48,312,1219.05,657 name=AdaptiveStatus role=none
          60 Grid at=392,784.5,1137,121.5 by=panel limit=350.48,312,1219.05,657 role=none
            61 Grid at=392,784.5,1137,43.5 by=panel limit=350.48,312,1219.05,657 role=none
              62 ProgressBar at=392,802.5,180,6 by=panel drawn heard=Progress limit=350.48,312,1219.05,657 paints=#333337FF,#4C9AE8FF role=progressbar
              63 TextBlock at=584,784.5,945,43.5 box=630,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Scraping: 1 of 2 games" layout=174.98,28.8 limit=350.48,312,1219.05,657 paints=#D4D4D4FF role=text size=24 text="Scraping: 1 of 2 games"
            64 StackPanel at=1376,840,153,66 by=panel limit=350.48,312,1219.05,657 role=none
              65 Button at=1376,840,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,312,1219.05,657 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                66 TextBlock at=1401.5,849,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1376,840,153,66 off owner=65 paints=#707070FF role=text size=26 text="APPLY…"
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
edge 35 right 5
edge 40 up 35
edge 40 down 55
edge 40 left same
edge 40 right 5
edge 55 up 40
edge 55 down same
edge 55 left 14
edge 55 right 26
press pageup focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press first focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press last focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
