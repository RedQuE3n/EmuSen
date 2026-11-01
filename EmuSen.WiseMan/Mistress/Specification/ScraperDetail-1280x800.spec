# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Scraper
case ScraperDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 ScraperPane at=258,136,765,536 by=panel heard=Scraper limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,129,44 by=panel drawn focus heard="All Games" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=291,155,95,32 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=94.25,31.2 limit=274,149,129,44 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=411,149,139,44 by=panel drawn focus heard=Favourites limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=428,155,105,32 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=104.44,31.2 limit=411,149,139,44 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=558,149,164,44 by=panel drawn focus heard="Missing Media" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=575,155,130,32 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=129.14,31.2 limit=558,149,164,44 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          17 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            18 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              19 ScrollViewer at=282,214,717,263 by=panel fades=b limit=234,124,812.35,548 overflow=34 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,263
                20 StackPanel at=282,214,717,297 by=scroll in=19 limit=282,214,717,263 paints=#9C9CA0FF role=none seen=282,214,717,263
                  21 TextBlock at=282,214,717,58 box=717,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=19 layout=698.02,57.6 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                  22 TextBlock at=282,278,717,58 box=717,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=19 layout=697.18,57.6 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                  23 TextBlock at=282,342,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=19 layout=97.3,28.8 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="System: psx"
                  24 TextBlock at=282,377,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=19 layout=174.65,28.8 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="cover: ScreenScraper"
                  25 TextBlock at=282,412,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=19 layout=163.61,28.8 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="screenshot: missing"
                  26 TextBlock at=282,447,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=19 layout=145.18,28.8 limit=282,214,717,263 paints=#9C9CA0FF role=text size=24 text="marquee: missing"
        27 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          28 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            29 ActionButton at=274,493,118,44 by=panel drawn focus heard="MARK ALL" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Mark All"
              30 TextBlock at=291,499,84,32 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=83.93,31.2 limit=274,493,118,44 owner=29 paints=#9C9CA0FF role=text size=26 text="MARK ALL"
            31 ActionButton at=404,493,93,44 by=panel drawn focus heard=THEME limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Theme
              32 TextBlock at=421,499,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=58.94,31.2 limit=404,493,93,44 owner=31 paints=#9C9CA0FF role=text size=26 text=THEME
            33 ActionButton at=509,493,97,44 by=panel drawn focus heard=VIDEOS limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Videos
              34 TextBlock at=526,499,63,32 box=63,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=62.74,31.2 limit=509,493,97,44 owner=33 paints=#9C9CA0FF role=text size=26 text=VIDEOS
            35 ActionButton at=618,493,136,44 by=panel drawn focus heard=OVERWRITE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Overwrite
              36 TextBlock at=635,499,102,32 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=101.24,31.2 limit=618,493,136,44 owner=35 paints=#9C9CA0FF role=text size=26 text=OVERWRITE
            37 ActionButton at=766,493,78,44 by=panel drawn heard=STOP limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text=Stop
              38 TextBlock at=783,499,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=766,493,78,44 off owner=37 paints=#55555AFF role=text size=26 text=STOP
            39 ActionButton at=856,493,82,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              40 TextBlock at=873,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=856,493,82,44 owner=39 paints=#9C9CA0FF role=text size=26 text=MARK
        41 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          42 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            43 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              44 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=639.6,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
            45 StackPanel at=914,586,105,44 by=panel limit=234,124,812.35,548 role=none
              46 Button at=914,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                47 TextBlock at=931,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=914,586,105,44 off owner=46 paints=#55555AFF role=text size=26 text="APPLY…"
        48 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Mark" limit=234,124,812.35,548 role=group
focus 5
edge 5 up same
edge 5 down 39
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 29
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 31
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 33
edge 14 left 12
edge 14 right 5
edge 29 up 10
edge 29 down same
edge 29 left same
edge 29 right 31
edge 31 up 12
edge 31 down same
edge 31 left 29
edge 31 right 33
edge 33 up 12
edge 33 down same
edge 33 left 31
edge 33 right 35
edge 35 up 14
edge 35 down same
edge 35 left 33
edge 35 right 39
edge 39 up 5
edge 39 down same
edge 39 left 35
edge 39 right same
press pageup focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press first focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press last focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
