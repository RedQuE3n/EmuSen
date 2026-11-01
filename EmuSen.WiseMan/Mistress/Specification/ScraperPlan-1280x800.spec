# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,63,812.7,629
title Scraper
case ScraperPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,163,813,509 by=panel limit=234,163,812.35,509 paints=#D4D4D4FF role=none seen=234,163,812.35,509
    1 ScraperPane at=258,175,765,497 by=panel heard=Scraper limit=234,163,812.35,509 role=pane
      2 Grid at=258,175,765,497 by=panel limit=234,163,812.35,509 role=none
        3 DockPanel at=274,187,733,46 by=panel limit=234,163,812.35,509 role=none
          4 Border at=827,187,180,46 by=panel limit=234,163,812.35,509 name=AdaptiveSearch role=none
            5 TextBox at=827,187,180,46 by=panel drawn focus heard=Search limit=234,163,812.35,509 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,194,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,194,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=274,187,553,46 by=panel limit=234,163,812.35,509 name=AdaptiveNavigation role=none
            9 WholeRow at=274,187,553,46 by=panel limit=234,163,812.35,509 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,188,126,44 by=panel drawn focus heard="All Games" limit=234,163,812.35,509 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=291,194,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=91.44,31.2 limit=274,188,126,44 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=408,188,133,44 by=panel drawn focus heard=Favourites limit=234,163,812.35,509 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=425,194,99,32 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=98.1,31.2 limit=408,188,133,44 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=549,188,156,44 by=panel drawn focus heard="Missing Media" limit=234,163,812.35,509 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=566,194,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=121.71,31.2 limit=549,188,156,44 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=274,245,733,240 by=panel limit=234,163,812.35,509 role=none
          17 Border at=274,245,733,240 by=panel limit=234,163,812.35,509 name=AdaptiveContent role=none
            18 Grid at=274,245,733,240 by=panel limit=234,163,812.35,509 role=none
              19 Border at=298,245,684,240 by=panel limit=234,163,812.35,509 paints=#1B1B1EFF,#707070FF role=none
                20 DockPanel at=315,262,650,206 by=panel limit=234,163,812.35,509 role=none
                  21 TextBlock at=315,262,650,29 box=650,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=234,163,812.35,509 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  22 StackPanel at=773,424,192,44 by=panel limit=234,163,812.35,509 role=none
                    23 Button at=773,424,99,44 by=panel drawn focus heard=Cancel limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      24 TextBlock at=790,430,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=773,424,99,44 owner=23 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    25 Button at=880,424,85,44 by=panel drawn focus heard=Apply limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      26 TextBlock at=897,430,51,32 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=880,424,85,44 owner=25 paints=#D4D4D4FF role=text size=26 text=APPLY
                  27 ScrollViewer at=315,303,650,109 by=panel fades=b limit=234,163,812.35,509 overflow=35 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=315,303,650,109
                    28 TextBlock at=315,303,650,144 box=650,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=27 layout=644.9,144 limit=315,303,650,109 paints=#D4D4D4FF role=text seen=315,303,650,109 size=24 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
        29 Border at=274,493,733,44 by=panel limit=234,163,812.35,509 name=AdaptiveActions role=none
          30 WrapPanel at=274,493,733,44 by=panel limit=234,163,812.35,509 role=none
            31 ActionButton at=274,493,117,44 by=panel drawn focus heard="MARK ALL" limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text="Mark All"
              32 TextBlock at=291,499,83,32 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=82.13,31.2 limit=274,493,117,44 owner=31 paints=#D4D4D4FF role=text size=26 text="MARK ALL"
            33 ActionButton at=403,493,90,44 by=panel drawn focus heard=THEME limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Theme
              34 TextBlock at=420,499,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=55.04,31.2 limit=403,493,90,44 owner=33 paints=#D4D4D4FF role=text size=26 text=THEME
            35 ActionButton at=505,493,93,44 by=panel drawn focus heard=VIDEOS limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Videos
              36 TextBlock at=522,499,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=58.6,31.2 limit=505,493,93,44 owner=35 paints=#D4D4D4FF role=text size=26 text=VIDEOS
            37 ActionButton at=610,493,130,44 by=panel drawn focus heard=OVERWRITE limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Overwrite
              38 TextBlock at=627,499,96,32 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=95,31.2 limit=610,493,130,44 owner=37 paints=#D4D4D4FF role=text size=26 text=OVERWRITE
            39 ActionButton at=752,493,77,44 by=panel drawn heard=STOP limit=234,163,812.35,509 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              40 TextBlock at=769,499,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=752,493,77,44 off owner=39 paints=#707070FF role=text size=26 text=STOP
            41 ActionButton at=841,493,80,44 by=panel drawn focus heard=MARK limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              42 TextBlock at=858,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=841,493,80,44 owner=41 paints=#D4D4D4FF role=text size=26 text=MARK
        43 Border at=258,545,765,89 by=panel limit=234,163,812.35,509 name=AdaptiveStatus role=none
          44 Grid at=262,549,757,81 by=panel limit=234,163,812.35,509 role=none
            45 Grid at=262,549,757,29 by=panel limit=234,163,812.35,509 role=none
              46 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 games marked to scrape" layout=208.61,28.8 limit=234,163,812.35,509 paints=#D4D4D4FF role=text size=24 text="2 games marked to scrape"
            47 StackPanel at=917,586,102,44 by=panel limit=234,163,812.35,509 role=none
              48 Button at=917,586,102,44 by=panel drawn focus heard="Apply the marked changes" limit=234,163,812.35,509 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                49 TextBlock at=934,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=917,586,102,44 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
        50 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Mark" limit=234,163,812.35,509 role=group
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
edge 41 right same
edge 48 up 25
edge 48 down same
edge 48 left 23
edge 48 right same
press pageup focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press first focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press last focus="Button 'Cancel' (Cancel)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
