# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScraperPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case ScraperPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 ScraperPane at=72,42,1776,1146 by=panel heard=Scraper limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=SCRAPER limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                  8 TextBlock at=96,60,1728,58.5 box=1152,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=SCRAPER layout=92.8,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=SCRAPER
            9 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1200 role=none
              10 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                11 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  12 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    13 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=12 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              14 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                15 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 role=none
                  16 ToggleButton at=96,138,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    17 TextBlock at=120,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,138,99,66 owner=16 paints=#EEEEF0FF role=text size=20 text=ALL
                  18 ToggleButton at=207,138,102,66 by=panel drawn focus heard=nes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=nes
                    19 TextBlock at=231,150,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=207,138,102,66 owner=18 paints=#D4D4D4FF role=text size=20 text=NES
                  20 ToggleButton at=321,138,99,66 by=panel drawn focus heard=psx limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=psx
                    21 TextBlock at=345,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=PSX layout=33.78,24 limit=321,138,99,66 owner=20 paints=#D4D4D4FF role=text size=20 text=PSX
                  22 ToggleButton at=432,138,118.5,66 by=panel drawn focus heard=snes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=snes
                    23 TextBlock at=456,150,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=432,138,118.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text=SNES
            24 Grid at=96,223.5,1728,814.5 by=panel limit=0,0,1920,1200 role=none
              25 Grid at=96,223.5,1036.5,814.5 by=panel limit=0,0,1920,1200 role=none
                26 Border at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  27 Grid at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 role=none
                    28 Border at=102,268.5,1026,655.5 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF,#707070FF role=none
                      29 DockPanel at=127.5,294,975,604.5 by=panel limit=0,0,1920,1200 role=none
                        30 TextBlock at=127.5,294,975,58.5 box=650,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                        31 StackPanel at=127.5,760.5,975,138 by=panel limit=0,0,1920,1200 role=none
                          32 Button at=127.5,760.5,975,66 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=No
                            33 TextBlock at=153,769.5,36,48 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=127.5,760.5,975,66 owner=32 paints=#EEEEF0FF role=text size=26 text=NO
                          34 Button at=127.5,832.5,975,66 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                            35 TextBlock at=153,841.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=127.5,832.5,975,66 owner=34 paints=#D4D4D4FF role=text size=26 text=YES
                        36 StackPanel at=127.5,586.5,975,156 by=panel limit=0,0,1920,1200 role=none
                          37 ToggleSwitch at=127.5,586.5,975,48 by=panel drawn focus heard=Theme limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                            38 TextBlock at=205.5,589.5,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=127.5,586.5,975,48 owner=37 paints=#D4D4D4FF role=text size=24 text=Off
                          39 ToggleSwitch at=127.5,640.5,975,48 by=panel drawn focus heard=Videos limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                            40 TextBlock at=205.5,643.5,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=127.5,640.5,975,48 owner=39 paints=#D4D4D4FF role=text size=24 text=Off
                          41 ToggleSwitch at=127.5,694.5,975,48 by=panel drawn focus heard=Overwrite limit=0,0,1920,1200 op paints=#8D8D91FF,#D4D4D4FF role=button
                            42 TextBlock at=205.5,697.5,34.5,43.5 box=23,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Off layout=22.1,28.8 limit=127.5,694.5,975,48 owner=41 paints=#D4D4D4FF role=text size=24 text=Off
                        43 ScrollViewer at=127.5,370.5,975,216 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=127.5,370.5,975,216
                          44 TextBlock at=127.5,370.5,975,216 box=650,144 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply." in=43 layout=644.9,144 limit=127.5,370.5,975,216 paints=#D4D4D4FF role=text size=24 text="2 game(s), about 7 request(s) (at most 10), 0 already complete; 19880 request(s) left today; about 1 min.\nKinds: cover, screenshot, marquee, miximage.\nWhat is there is kept; only what is missing is fetched.\nNothing is sent to ScreenScraper until Apply."
                45 Border at=96,970.5,1036.5,67.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  46 Grid at=102,976.5,1024.5,55.5 by=panel limit=0,0,1920,1200 role=none
                    47 Grid at=102,976.5,1024.5,43.5 by=panel limit=0,0,1920,1200 role=none
                      48 TextBlock at=114,976.5,1012.5,43.5 box=675,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 games marked to scrape" layout=208.61,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="2 games marked to scrape"
              49 Border at=1156.5,223.5,667.5,814.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                50 Border at=1192.5,259.5,595.5,742.5 by=panel limit=0,0,1920,1200 role=none
                  51 ScrollViewer at=1204.5,271.5,571.5,718.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,271.5,571.5,718.5
                    52 StackPanel at=1204.5,271.5,571.5,718.5 by=fixed-scroll in=51 limit=1204.5,271.5,571.5,718.5 role=none
                      53 TextBlock at=1204.5,271.5,571.5,87 box=381,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=51 layout=361.46,57.6 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      54 TextBlock at=1204.5,367.5,571.5,130.5 box=381,87 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=51 layout=344.14,86.4 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="/r/psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                      55 TextBlock at=1204.5,507,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=51 layout=93.91,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                      56 TextBlock at=1204.5,559.5,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="cover: ScreenScraper" in=51 layout=166.8,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="cover: ScreenScraper"
                      57 TextBlock at=1204.5,612,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="screenshot: missing" in=51 layout=155.74,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="screenshot: missing"
                      58 TextBlock at=1204.5,664.5,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="marquee: missing" in=51 layout=139.46,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="marquee: missing"
                      59 TextBlock at=1204.5,717,571.5,43.5 box=381,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="miximage: the game's package" in=51 layout=241.15,28.8 limit=1204.5,271.5,571.5,718.5 paints=#D4D4D4FF role=text size=24 text="miximage: the game's package"
            60 Border at=96,1050,1728,66 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              61 WrapPanel at=96,1050,1728,66 by=panel limit=0,0,1920,1200 role=none
                62 ActionButton at=96,1050,115.5,66 by=panel drawn heard=STOP limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text=Stop
                  63 TextBlock at=121.5,1059,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1050,115.5,66 off owner=62 paints=#707070FF role=text size=26 text=STOP
            64 HintBar at=96,1140,1728,36 by=panel heard="Select, Back, System, List, Apply…, Mark" limit=0,0,1920,1200 role=group
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
edge 34 down same
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
press pageup focus="Button 'No' (No)" top=ScraperWindow
press pagedown focus="Button 'No' (No)" top=ScraperWindow
press first focus="Button 'No' (No)" top=ScraperWindow
press last focus="Button 'No' (No)" top=ScraperWindow
press back focus="ListBoxItem  (Mark: Scrape, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), Media: 2 of 4)" top=ScraperWindow
