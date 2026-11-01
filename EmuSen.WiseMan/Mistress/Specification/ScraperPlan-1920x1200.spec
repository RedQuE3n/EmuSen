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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,244,1221,765 by=panel limit=350.48,244,1219.05,765 paints=#D4D4D4FF role=none seen=350.48,244,1219.05,765
    1 ScraperPane at=386,262,1149,747 by=panel heard=Scraper limit=350.48,244,1219.05,765 role=pane
      2 Grid at=386,262,1149,747 by=panel limit=350.48,244,1219.05,765 role=none
        3 DockPanel at=410,280,1101,69 by=panel limit=350.48,244,1219.05,765 role=none
          4 Border at=1241,280,270,69 by=panel limit=350.48,244,1219.05,765 name=AdaptiveSearch role=none
            5 TextBox at=1241,280,270,69 by=panel drawn focus heard=Search limit=350.48,244,1219.05,765 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,290.5,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,290.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=410,280,831,69 by=panel limit=350.48,244,1219.05,765 name=AdaptiveNavigation role=none
            9 WholeRow at=410,280,831,69 by=panel limit=350.48,244,1219.05,765 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,281.5,189,66 by=panel drawn focus heard="All Games" limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=435.5,290.5,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=91.44,31.2 limit=410,281.5,189,66 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=611,281.5,199.5,66 by=panel drawn focus heard=Favourites limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=636.5,290.5,148.5,48 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=98.1,31.2 limit=611,281.5,199.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=822.5,281.5,234,66 by=panel drawn focus heard="Missing Media" limit=350.48,244,1219.05,765 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=848,290.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=121.71,31.2 limit=822.5,281.5,234,66 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 role=none
          17 Border at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 name=AdaptiveContent role=none
            18 Grid at=410,367,1101,361.5 by=panel limit=350.48,244,1219.05,765 role=none
              19 Border at=447.5,367,1026,361.5 by=panel limit=350.48,244,1219.05,765 paints=#1B1B1EFF,#707070FF role=none
                20 DockPanel at=473,392.5,975,310.5 by=panel limit=350.48,244,1219.05,765 role=none
                  21 TextBlock at=473,392.5,975,43.5 box=650,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=350.48,244,1219.05,765 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  22 StackPanel at=1160,637,288,66 by=panel limit=350.48,244,1219.05,765 role=none
                    23 Button at=1160,637,148.5,66 by=panel drawn focus heard=Cancel limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      24 TextBlock at=1185.5,646,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1160,637,148.5,66 owner=23 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    25 Button at=1320.5,637,127.5,66 by=panel drawn focus heard=Apply limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      26 TextBlock at=1346,646,76.5,48 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=1320.5,637,127.5,66 owner=25 paints=#D4D4D4FF role=text size=26 text=APPLY
                  27 ScrollViewer at=473,454,975,165 by=panel fades=b limit=350.48,244,1219.05,765 overflow=51 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=473,454,975,165
                    28 TextBlock at=473,454,975,216 box=650,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=27 layout=644.9,144 limit=473,454,975,165 paints=#D4D4D4FF role=text seen=473,454,975,165 size=24 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
        29 Border at=410,740.5,1101,66 by=panel limit=350.48,244,1219.05,765 name=AdaptiveActions role=none
          30 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,244,1219.05,765 role=none
            31 ActionButton at=410,740.5,175.5,66 by=panel drawn focus heard="MARK ALL" limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text="Mark All"
              32 TextBlock at=435.5,749.5,124.5,48 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=82.13,31.2 limit=410,740.5,175.5,66 owner=31 paints=#D4D4D4FF role=text size=26 text="MARK ALL"
            33 ActionButton at=603.5,740.5,135,66 by=panel drawn focus heard=THEME limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Theme
              34 TextBlock at=629,749.5,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=55.04,31.2 limit=603.5,740.5,135,66 owner=33 paints=#D4D4D4FF role=text size=26 text=THEME
            35 ActionButton at=756.5,740.5,139.5,66 by=panel drawn focus heard=VIDEOS limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Videos
              36 TextBlock at=782,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=58.6,31.2 limit=756.5,740.5,139.5,66 owner=35 paints=#D4D4D4FF role=text size=26 text=VIDEOS
            37 ActionButton at=914,740.5,195,66 by=panel drawn focus heard=OVERWRITE limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Overwrite
              38 TextBlock at=939.5,749.5,144,48 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=95,31.2 limit=914,740.5,195,66 owner=37 paints=#D4D4D4FF role=text size=26 text=OVERWRITE
            39 ActionButton at=1127,740.5,115.5,66 by=panel drawn heard=STOP limit=350.48,244,1219.05,765 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              40 TextBlock at=1152.5,749.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=1127,740.5,115.5,66 off owner=39 paints=#707070FF role=text size=26 text=STOP
            41 ActionButton at=1260.5,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              42 TextBlock at=1286,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1260.5,740.5,120,66 owner=41 paints=#D4D4D4FF role=text size=26 text=MARK
        43 Border at=386,818.5,1149,133.5 by=panel limit=350.48,244,1219.05,765 name=AdaptiveStatus role=none
          44 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,244,1219.05,765 role=none
            45 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,244,1219.05,765 role=none
              46 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 games marked to scrape" layout=208.61,28.8 limit=350.48,244,1219.05,765 paints=#D4D4D4FF role=text size=24 text="2 games marked to scrape"
            47 StackPanel at=1376,880,153,66 by=panel limit=350.48,244,1219.05,765 role=none
              48 Button at=1376,880,153,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,244,1219.05,765 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                49 TextBlock at=1401.5,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1376,880,153,66 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
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
edge 48 up 25
edge 48 down same
edge 48 left 23
edge 48 right same
press pageup focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press first focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press last focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
