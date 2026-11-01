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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 ScraperPane at=258,136,765,536 by=panel heard=Scraper limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,126,44 by=panel drawn focus heard="All Games" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="All Games"
                11 TextBlock at=291,155,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ALL GAMES" layout=91.44,31.2 limit=274,149,126,44 owner=10 paints=#EEEEF0FF role=text size=26 text="ALL GAMES"
              12 ToggleButton at=408,149,133,44 by=panel drawn focus heard=Favourites limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=Favourites
                13 TextBlock at=425,155,99,32 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=FAVOURITES layout=98.1,31.2 limit=408,149,133,44 owner=12 paints=#FFFFFFFF role=text size=26 text=FAVOURITES
              14 ToggleButton at=549,149,156,44 by=panel drawn focus heard="Missing Media" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Missing Media"
                15 TextBlock at=566,155,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MISSING MEDIA" layout=121.71,31.2 limit=549,149,156,44 owner=14 paints=#FFFFFFFF role=text size=26 text="MISSING MEDIA"
        16 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          17 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            18 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              19 ScrollViewer at=282,214,717,263 by=panel fades=b limit=234,124,812.35,548 overflow=5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,263
                20 StackPanel at=282,214,717,268 by=scroll in=19 limit=282,214,717,263 role=none seen=282,214,717,263
                  21 TextBlock at=282,214,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=19 layout=706.97,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                  22 TextBlock at=282,249,717,58 box=717,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=19 layout=707.83,57.6 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                  23 TextBlock at=282,313,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=19 layout=93.91,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="System: psx"
                  24 TextBlock at=282,348,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=19 layout=166.8,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                  25 TextBlock at=282,383,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=19 layout=155.74,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                  26 TextBlock at=282,418,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=19 layout=139.46,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                  27 TextBlock at=282,453,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=19 layout=241.15,28.8 limit=282,214,717,263 paints=#D4D4D4FF role=text seen=282,453,717,24 size=24 text="miximage: the game's package"
        28 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          29 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            30 ActionButton at=274,493,117,44 by=panel drawn focus heard="MARK ALL" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Mark All"
              31 TextBlock at=291,499,83,32 box=83,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MARK ALL" layout=82.13,31.2 limit=274,493,117,44 owner=30 paints=#D4D4D4FF role=text size=26 text="MARK ALL"
            32 ActionButton at=403,493,90,44 by=panel drawn focus heard=THEME limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Theme
              33 TextBlock at=420,499,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=THEME layout=55.04,31.2 limit=403,493,90,44 owner=32 paints=#D4D4D4FF role=text size=26 text=THEME
            34 ActionButton at=505,493,93,44 by=panel drawn focus heard=VIDEOS limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Videos
              35 TextBlock at=522,499,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VIDEOS layout=58.6,31.2 limit=505,493,93,44 owner=34 paints=#D4D4D4FF role=text size=26 text=VIDEOS
            36 ActionButton at=610,493,130,44 by=panel drawn focus heard=OVERWRITE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Overwrite
              37 TextBlock at=627,499,96,32 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=OVERWRITE layout=95,31.2 limit=610,493,130,44 owner=36 paints=#D4D4D4FF role=text size=26 text=OVERWRITE
            38 ActionButton at=752,493,77,44 by=panel drawn heard=STOP limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              39 TextBlock at=769,499,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=752,493,77,44 off owner=38 paints=#707070FF role=text size=26 text=STOP
            40 ActionButton at=841,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              41 TextBlock at=858,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=841,493,80,44 owner=40 paints=#D4D4D4FF role=text size=26 text=MARK
        42 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          43 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            44 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              45 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it." layout=613.94,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="1 game(s) marked \"exclude from multi-scraper\" left out; name one to include it."
            46 StackPanel at=917,586,102,44 by=panel limit=234,124,812.35,548 role=none
              47 Button at=917,586,102,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                48 TextBlock at=934,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=917,586,102,44 off owner=47 paints=#707070FF role=text size=26 text="APPLY…"
        49 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Mark" limit=234,124,812.35,548 role=group
focus 5
edge 5 up same
edge 5 down 40
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 30
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 32
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 34
edge 14 left 12
edge 14 right 5
edge 30 up 10
edge 30 down same
edge 30 left same
edge 30 right 32
edge 32 up 12
edge 32 down same
edge 32 left 30
edge 32 right 34
edge 34 up 12
edge 34 down same
edge 34 left 32
edge 34 right 36
edge 36 up 14
edge 36 down same
edge 36 left 34
edge 36 right 40
edge 40 up 5
edge 40 down same
edge 40 left 36
edge 40 right same
press pageup focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press pagedown focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press first focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press last focus="TextBox Search games (Search)" title=Scraper top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" title=Scraper top=ScraperWindow
