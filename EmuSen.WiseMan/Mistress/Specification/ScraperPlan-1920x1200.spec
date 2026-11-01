# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,94,1219.05,945
title Scraper
case ScraperPlan
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,244,1221,765 by=panel limit=350.48,244,1219.05,765 paints=#9C9CA0FF role=none seen=350.48,244,1219.05,765
    1 ScraperPane at=386,262,1149,747 by=panel heard=Scraper limit=350.48,244,1219.05,765 role=pane
      2 Grid at=386,262,1149,747 by=panel limit=350.48,244,1219.05,765 role=none
        3 DockPanel at=410,280,1101,69 by=panel limit=350.48,244,1219.05,765 role=none
          4 Border at=1241,280,270,69 by=panel limit=350.48,244,1219.05,765 name=AdaptiveSearch role=none
            5 TextBox at=1241,280,270,69 by=panel drawn focus heard=Search limit=350.48,244,1219.05,765 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,290.5,237,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,290.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,280,831,69 by=panel limit=350.48,244,1219.05,765 name=AdaptiveNavigation role=none
            9 WholeRow at=410,280,831,69 by=panel limit=350.48,244,1219.05,765 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,281.5,193.5,66 by=panel drawn focus heard="All Games" limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=435.5,290.5,142.5,48 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=94.25,31.2 limit=410,281.5,193.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=615.5,281.5,208.5,66 by=panel drawn focus heard=Favourites limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=641,290.5,157.5,48 box=105,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=104.44,31.2 limit=615.5,281.5,208.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=836,281.5,246,66 by=panel drawn focus heard="Missing Media" limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=861.5,290.5,195,48 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=129.14,31.2 limit=836,281.5,246,66 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 role=none
          17 Border at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 name=AdaptiveContent role=none
            18 Grid at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 role=none
              19 Border at=425,367,1069.5,361.5 by=panel limit=350.48,244,1219.05,765 paints=#1B1B1EFF,#55555AFF role=none
                20 DockPanel at=450.5,392.5,1018.5,310.5 by=panel limit=350.48,244,1219.05,765 role=none
                  21 TextBlock at=450.5,392.5,1018.5,43.5 box=679,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=175.61,28.8 limit=350.48,244,1219.05,765 paints=#9C9CA0FF role=text size=24 text="Apply these changes?"
                  22 StackPanel at=1169,637,300,66 by=panel limit=350.48,244,1219.05,765 role=none
                    23 Button at=1169,637,154.5,66 by=panel drawn focus heard=Cancel limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
                      24 TextBlock at=1194.5,646,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1169,637,154.5,66 owner=23 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    25 Button at=1335.5,637,133.5,66 by=panel drawn focus heard=Apply limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Apply
                      26 TextBlock at=1361,646,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=54.29,31.2 limit=1335.5,637,133.5,66 owner=25 paints=#9C9CA0FF role=text size=26 text=APPLY
                  27 ScrollViewer at=450.5,454,1018.5,165 by=panel fades=b limit=350.48,244,1219.05,765 overflow=51 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=450.5,454,1018.5,165
                    28 TextBlock at=450.5,454,1018.5,216 box=679,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=27 layout=673.56,144 limit=450.5,454,1018.5,165 paints=#9C9CA0FF role=text seen=450.5,454,1018.5,165 size=24 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
        29 Border at=410,740.5,1101,66 by=panel limit=350.48,244,1219.05,765 name=AdaptiveActions role=none
          30 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,244,1219.05,765 role=none
            31 ActionButton at=410,740.5,177,66 by=panel drawn focus heard="MARK ALL" limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text="Mark All"
              32 TextBlock at=435.5,749.5,126,48 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=83.93,31.2 limit=410,740.5,177,66 owner=31 paints=#9C9CA0FF role=text size=26 text="MARK ALL"
            33 ActionButton at=605,740.5,139.5,66 by=panel drawn focus heard=THEME limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Theme
              34 TextBlock at=630.5,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=58.94,31.2 limit=605,740.5,139.5,66 owner=33 paints=#9C9CA0FF role=text size=26 text=THEME
            35 ActionButton at=762.5,740.5,145.5,66 by=panel drawn focus heard=VIDEOS limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Videos
              36 TextBlock at=788,749.5,94.5,48 box=63,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=62.74,31.2 limit=762.5,740.5,145.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text=VIDEOS
            37 ActionButton at=926,740.5,204,66 by=panel drawn focus heard=OVERWRITE limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Overwrite
              38 TextBlock at=951.5,749.5,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=101.24,31.2 limit=926,740.5,204,66 owner=37 paints=#9C9CA0FF role=text size=26 text=OVERWRITE
            39 ActionButton at=1148,740.5,117,66 by=panel drawn heard=STOP limit=350.48,244,1219.05,765 off paints=#55555AFF,#9C9CA0FF role=button text=Stop
              40 TextBlock at=1173.5,749.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=1148,740.5,117,66 off owner=39 paints=#55555AFF role=text size=26 text=STOP
            41 ActionButton at=1283,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              42 TextBlock at=1308.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1283,740.5,123,66 owner=41 paints=#9C9CA0FF role=text size=26 text=MARK
        43 Border at=386,818.5,1149,133.5 by=panel limit=350.48,244,1219.05,765 name=AdaptiveStatus role=none
          44 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,244,1219.05,765 role=none
            45 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,244,1219.05,765 role=none
              46 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 games marked to scrape" layout=216.7,28.8 limit=350.48,244,1219.05,765 paints=#9C9CA0FF role=text size=24 text="2 games marked to scrape"
            47 StackPanel at=1371.5,880,157.5,66 by=panel limit=350.48,244,1219.05,765 role=none
              48 Button at=1371.5,880,157.5,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,244,1219.05,765 op paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                49 TextBlock at=1397,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=1371.5,880,157.5,66 owner=48 paints=#9C9CA0FF role=text size=26 text="APPLY…"
        50 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Mark" limit=350.48,244,1219.05,765 role=group
focus 23
edge 5 up same
edge 5 down 23
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 31
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 33
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 35
edge 14 left 12
edge 14 right 5
edge 23 up 5
edge 23 down 41
edge 23 left 37
edge 23 right 25
edge 25 up 5
edge 25 down 41
edge 25 left 23
edge 25 right same
edge 31 up 10
edge 31 down 48
edge 31 left same
edge 31 right 33
edge 33 up 12
edge 33 down 48
edge 33 left 31
edge 33 right 35
edge 35 up 12
edge 35 down 48
edge 35 left 33
edge 35 right 37
edge 37 up 14
edge 37 down 48
edge 37 left 35
edge 37 right 41
edge 41 up 23
edge 41 down 48
edge 41 left 37
edge 41 right 48
edge 48 up 41
edge 48 down same
edge 48 left 23
edge 48 right same
press pageup focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press first focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press last focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
