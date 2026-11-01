# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Scraper
case ScraperDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 ScraperPane at=386,205,1149,804 by=panel heard=Scraper limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,193.5,66 by=panel drawn focus heard="All Games" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=435.5,233.5,142.5,48 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=94.25,31.2 limit=410,224.5,193.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=615.5,224.5,208.5,66 by=panel drawn focus heard=Favourites limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=641,233.5,157.5,48 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=104.44,31.2 limit=615.5,224.5,208.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=836,224.5,246,66 by=panel drawn focus heard="Missing Media" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=861.5,233.5,195,48 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=129.14,31.2 limit=836,224.5,246,66 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
          17 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            18 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              19 ScrollViewer at=422,322,1077,394.5 by=panel fades=b limit=350.48,187,1219.05,822 overflow=51 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,394.5
                20 StackPanel at=422,322,1077,445.5 by=scroll in=19 limit=422,322,1077,394.5 paints=#9C9CA0FF role=none seen=422,322,1077,394.5
                  21 TextBlock at=422,322,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=19 layout=698.02,57.6 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                  22 TextBlock at=422,418,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=19 layout=697.18,57.6 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                  23 TextBlock at=422,514,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=19 layout=97.3,28.8 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="System: psx"
                  24 TextBlock at=422,566.5,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=19 layout=174.65,28.8 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="cover: ScreenScraper"
                  25 TextBlock at=422,619,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=19 layout=163.61,28.8 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="screenshot: missing"
                  26 TextBlock at=422,671.5,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=19 layout=145.18,28.8 limit=422,322,1077,394.5 paints=#9C9CA0FF role=text size=24 text="marquee: missing"
        27 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          28 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            29 ActionButton at=410,740.5,177,66 by=panel drawn focus heard="MARK ALL" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Mark All"
              30 TextBlock at=435.5,749.5,126,48 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=83.93,31.2 limit=410,740.5,177,66 owner=29 paints=#9C9CA0FF role=text size=26 text="MARK ALL"
            31 ActionButton at=605,740.5,139.5,66 by=panel drawn focus heard=THEME limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Theme
              32 TextBlock at=630.5,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=58.94,31.2 limit=605,740.5,139.5,66 owner=31 paints=#9C9CA0FF role=text size=26 text=THEME
            33 ActionButton at=762.5,740.5,145.5,66 by=panel drawn focus heard=VIDEOS limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Videos
              34 TextBlock at=788,749.5,94.5,48 box=63,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=62.74,31.2 limit=762.5,740.5,145.5,66 owner=33 paints=#9C9CA0FF role=text size=26 text=VIDEOS
            35 ActionButton at=926,740.5,204,66 by=panel drawn focus heard=OVERWRITE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Overwrite
              36 TextBlock at=951.5,749.5,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=101.24,31.2 limit=926,740.5,204,66 owner=35 paints=#9C9CA0FF role=text size=26 text=OVERWRITE
            37 ActionButton at=1148,740.5,117,66 by=panel drawn heard=STOP limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text=Stop
              38 TextBlock at=1173.5,749.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=1148,740.5,117,66 off owner=37 paints=#55555AFF role=text size=26 text=STOP
            39 ActionButton at=1283,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              40 TextBlock at=1308.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1283,740.5,123,66 owner=39 paints=#9C9CA0FF role=text size=26 text=MARK
        41 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          42 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            43 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              44 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=639.6,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
            45 StackPanel at=1371.5,880,157.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              46 Button at=1371.5,880,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                47 TextBlock at=1397,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=1371.5,880,157.5,66 off owner=46 paints=#55555AFF role=text size=26 text="APPLY…"
        48 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Mark" limit=350.48,187,1219.05,822 role=group
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
