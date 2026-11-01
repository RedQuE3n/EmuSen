# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case ScraperPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 ScraperPane at=48,28,1184,738 by=panel heard=Scraper limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 Border at=48,28,1184,51 by=panel heard=SCRAPER limit=0,0,1280,774 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,774 role=none
                7 StackPanel at=64,40,1152,39 by=panel limit=0,0,1280,774 role=none
                  8 TextBlock at=64,40,1152,39 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,774 role=none
              10 Border at=1036,91,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                11 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=12 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              14 Border at=64,91,972,46 by=panel limit=0,0,1280,774 name=AdaptiveNavigation role=none
                15 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,774 role=none
                  16 ToggleButton at=64,92,65,44 by=panel drawn focus heard=All limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=All
                    17 TextBlock at=81,98,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=64,92,65,44 owner=16 paints=#EEEEF0FF role=text size=26 text=ALL
                  18 ToggleButton at=137,92,67,44 by=panel drawn focus heard=nes limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=nes
                    19 TextBlock at=154,98,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=137,92,67,44 owner=18 paints=#FFFFFFFF role=text size=26 text=NES
                  20 ToggleButton at=212,92,66,44 by=panel drawn focus heard=psx limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=psx
                    21 TextBlock at=229,98,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PSX layout=31.04,31.2 limit=212,92,66,44 owner=20 paints=#FFFFFFFF role=text size=26 text=PSX
                  22 ToggleButton at=286,92,77,44 by=panel drawn focus heard=snes limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=snes
                    23 TextBlock at=303,98,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=286,92,77,44 owner=22 paints=#FFFFFFFF role=text size=26 text=SNES
            24 Grid at=64,149,1152,519 by=panel limit=0,0,1280,774 role=none
              25 Grid at=64,149,691,519 by=panel limit=0,0,1280,774 role=none
                26 Border at=64,149,691,430 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
                  27 Grid at=64,149,691,430 by=panel limit=0,0,1280,774 role=none
                    28 Border at=68,149,684,430 by=panel limit=0,0,1280,774 paints=#1B1B1EFF,#707070FF role=none
                      29 DockPanel at=85,166,650,396 by=panel limit=0,0,1280,774 role=none
                        30 TextBlock at=85,166,650,39 box=650,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        31 StackPanel at=85,470,650,92 by=panel limit=0,0,1280,774 role=none
                          32 Button at=85,470,650,44 by=panel drawn focus heard=No limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=No
                            33 TextBlock at=102,476,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=85,470,650,44 owner=32 paints=#EEEEF0FF role=text size=26 text=NO
                          34 Button at=85,518,650,44 by=panel drawn focus heard=Yes limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            35 TextBlock at=102,524,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=85,518,650,44 owner=34 paints=#D4D4D4FF role=text size=26 text=YES
                        36 StackPanel at=85,354,650,104 by=panel limit=0,0,1280,774 role=none
                          37 ToggleSwitch at=85,354,650,32 by=panel drawn focus heard=Theme limit=0,0,1280,774 op paints=#8D8D91FF,#D4D4D4FF role=button
                            38 TextBlock at=137,356,23,29 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=85,354,650,32 owner=37 paints=#D4D4D4FF role=text size=24 text=Off
                          39 ToggleSwitch at=85,390,650,32 by=panel drawn focus heard=Videos limit=0,0,1280,774 op paints=#8D8D91FF,#D4D4D4FF role=button
                            40 TextBlock at=137,392,23,29 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=85,390,650,32 owner=39 paints=#D4D4D4FF role=text size=24 text=Off
                          41 ToggleSwitch at=85,426,650,32 by=panel drawn focus heard=Overwrite limit=0,0,1280,774 op paints=#8D8D91FF,#D4D4D4FF role=button
                            42 TextBlock at=137,428,23,29 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=85,426,650,32 owner=41 paints=#D4D4D4FF role=text size=24 text=Off
                        43 ScrollViewer at=85,217,650,137 by=panel fades=b limit=0,0,1280,774 overflow=7 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=85,217,650,137
                          44 TextBlock at=85,217,650,144 box=650,144 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=43 layout=644.9,144 limit=85,217,650,137 paints=#D4D4D4FF role=text seen=85,217,650,137 size=24 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
                45 Border at=64,579,691,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
                  46 Grid at=68,583,683,81 by=panel limit=0,0,1280,774 role=none
                    47 Grid at=68,583,683,29 by=panel limit=0,0,1280,774 role=none
                      48 TextBlock at=76,583,675,29 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 games marked to scrape" layout=208.61,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="2 games marked to scrape"
                    49 StackPanel at=649,620,102,44 by=panel limit=0,0,1280,774 role=none
                      50 Button at=649,620,102,44 by=panel drawn focus heard="Apply the marked changes" limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                        51 TextBlock at=666,626,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=649,620,102,44 owner=50 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              52 Border at=771,149,445,519 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
                53 Border at=771,149,445,519 by=panel limit=0,0,1280,774 role=none
                  54 ScrollViewer at=779,157,429,503 by=panel limit=0,0,1280,774 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=779,157,429,503
                    55 StackPanel at=779,157,429,503 by=fixed-scroll in=54 limit=779,157,429,503 role=none
                      56 TextBlock at=779,157,429,58 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=54 layout=361.46,57.6 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      57 TextBlock at=779,221,429,58 box=429,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=54 layout=415.39,57.6 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      58 TextBlock at=779,285,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=54 layout=93.91,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      59 TextBlock at=779,320,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=54 layout=166.8,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                      60 TextBlock at=779,355,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=54 layout=155.74,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                      61 TextBlock at=779,390,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=54 layout=139.46,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                      62 TextBlock at=779,425,429,29 box=429,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=54 layout=241.15,28.8 limit=779,157,429,503 paints=#D4D4D4FF role=text size=24 text="miximage: the game's package"
            63 Border at=64,676,1152,44 by=panel limit=0,0,1280,774 name=AdaptiveActions role=none
              64 WrapPanel at=64,676,1152,44 by=panel limit=0,0,1280,774 role=none
                65 ActionButton at=64,676,77,44 by=panel drawn heard=STOP limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  66 TextBlock at=81,682,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=64,676,77,44 off owner=65 paints=#707070FF role=text size=26 text=STOP
            67 HintBar at=64,736,1152,22 by=panel heard="Select, Back, Section, List, Apply…, Mark" limit=0,0,1280,774 role=group
focus 32
edge 11 up same
edge 11 down 37
edge 11 left 22
edge 11 right same
edge 16 up same
edge 16 down 37
edge 16 left same
edge 16 right 18
edge 18 up same
edge 18 down 37
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 37
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 37
edge 22 left 20
edge 22 right 11
edge 32 up 41
edge 32 down 34
edge 32 left same
edge 32 right 11
edge 34 up 32
edge 34 down 50
edge 34 left same
edge 34 right 11
edge 37 up 16
edge 37 down 39
edge 37 left same
edge 37 right 11
edge 39 up 37
edge 39 down 41
edge 39 left same
edge 39 right 11
edge 41 up 39
edge 41 down 32
edge 41 left same
edge 41 right 11
edge 50 up 34
edge 50 down same
edge 50 left 18
edge 50 right 11
press pageup focus="Button 'No' (No)" top=ScraperWindow
press pagedown focus="Button 'No' (No)" top=ScraperWindow
press first focus="Button 'No' (No)" top=ScraperWindow
press last focus="Button 'No' (No)" top=ScraperWindow
press back focus="GridSplitter  (Resize Mark)" top=ScraperWindow
