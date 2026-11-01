# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case Packages
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 PackagesPane at=72,42,1776,1107 by=panel heard=Packages limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=PACKAGES limit=0,0,1920,1161 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                7 TextBlock at=1486.5,72,337.5,36 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=96,60,1372.5,58.5 by=panel limit=0,0,1920,1161 role=none
                  9 TextBlock at=96,60,1372.5,58.5 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1161 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 role=none
                  17 ToggleButton at=96,138,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    18 TextBlock at=121.5,147,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,138,97.5,66 owner=17 paints=#EEEEF0FF role=text size=26 text=ALL
                  19 ToggleButton at=205.5,138,100.5,66 by=panel drawn focus heard=NES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=NES
                    20 TextBlock at=231,147,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=205.5,138,100.5,66 owner=19 paints=#FFFFFFFF role=text size=26 text=NES
                  21 ToggleButton at=318,138,211.5,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=PlayStation
                    22 TextBlock at=343.5,147,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=318,138,211.5,66 owner=21 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  23 ToggleButton at=541.5,138,156,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="Sega CD"
                    24 TextBlock at=567,147,105,48 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=541.5,138,156,66 owner=23 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
                  25 ToggleButton at=709.5,138,115.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=SNES
                    26 TextBlock at=735,147,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=709.5,138,115.5,66 owner=25 paints=#FFFFFFFF role=text size=26 text=SNES
            27 Grid at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 role=none
              28 Border at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                29 Grid at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 role=none
                  30 LunaTable at=96,223.5,1728,735 by=panel heard=Rows limit=0,0,1920,1161 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    31 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=114,232.5,57,43.5 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                    32 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=162,229.5,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                    33 Button allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=171,229.5,817.5,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                      34 StackPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=171,229.5,72,48 role=none
                        35 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=171,229.5,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=33 paints=#D4D4D4FF role=text size=26 text=Game
                    36 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=979.5,229.5,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                    37 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,232.5,817.5,43.5 box=545,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                    38 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,288,1728,670.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,288,1728,670.5
                      39 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,288,1728,63 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        40 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,295.5,1698,46.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=38 role=none
                          41 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,295.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=39 paints=#D4D4D4FF role=text size=24 text="☐"
                          42 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,297,820.5,43.5 box=547,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=38 layout=538.49,28.8 owner=39 paints=#D4D4D4FF role=edit size=24 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                          43 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,297,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=20.71,28.8 owner=39 paints=#D4D4D4FF role=edit size=24 text=OK
                      44 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,351,1728,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        45 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,358.5,1698,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=38 role=none
                          46 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,358.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=44 paints=#D4D4D4FF role=text size=24 text="☐"
                          47 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,360,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=192.29,28.8 owner=44 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                          48 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,360,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=20.71,28.8 owner=44 paints=#D4D4D4FF role=edit size=24 text=OK
                      49 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,414,1728,63 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        50 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,421.5,1698,46.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=38 role=none
                          51 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,421.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=49 paints=#D4D4D4FF role=text size=24 text="☐"
                          52 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,423,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=189.96,28.8 owner=49 paints=#D4D4D4FF role=edit size=24 text="Ruby Lantern (Synthetic)"
                          53 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,423,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=68.06,28.8 owner=49 paints=#D4D4D4FF role=edit size=24 text=Changed
                      54 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,477,1728,63 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        55 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,484.5,1698,46.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=38 role=none
                          56 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,484.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=54 paints=#EEEEF0FF role=text size=24 text="☐"
                          57 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,486,820.5,43.5 box=547,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=38 layout=546.77,28.8 owner=54 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                          58 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,486,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=107.38,28.8 owner=54 paints=#EEEEF0FF role=edit size=24 text="Not packaged"
                      59 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,540,1728,63 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        60 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,547.5,1698,46.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=38 role=none
                          61 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,547.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=59 paints=#D4D4D4FF role=text size=24 text="☐"
                          62 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,549,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=140.47,28.8 owner=59 paints=#D4D4D4FF role=edit size=24 text="Ember Choir (USA)"
                          63 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,549,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=107.38,28.8 owner=59 paints=#D4D4D4FF role=edit size=24 text="Not packaged"
                      64 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=96,603,1728,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=38 op paints=#333337FF,#D4D4D4FF role=listitem
                        65 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,610.5,1698,46.5 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=38 role=none
                          66 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=111,610.5,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=38 layout=14.4,30.84 owner=64 paints=#D4D4D4FF role=text size=24 text="☐"
                          67 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=168,612,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=38 layout=139.25,28.8 owner=64 paints=#D4D4D4FF role=edit size=24 text="Amber Saga (USA)"
                          68 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=988.5,612,820.5,43.5 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=38 layout=107.38,28.8 owner=64 paints=#D4D4D4FF role=edit size=24 text="Not packaged"
            69 Border at=72,958.5,1776,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
              70 Grid at=78,964.5,1764,121.5 by=panel limit=0,0,1920,1161 role=none
                71 Grid at=78,964.5,1764,43.5 by=panel limit=0,0,1920,1161 role=none
                  72 TextBlock at=90,964.5,1752,43.5 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                73 StackPanel at=979.5,1020,862.5,66 by=panel limit=0,0,1920,1161 role=none
                  74 Button at=979.5,1020,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    75 TextBlock at=1005,1029,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=979.5,1020,153,66 off owner=74 paints=#707070FF role=text size=26 text="APPLY…"
                  76 StackPanel at=1150.5,1020,691.5,66 by=panel limit=0,0,1920,1161 role=none
                    77 ProgressBar at=1150.5,1042.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                    78 TextBlock at=1462.5,1032,379.5,43.5 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            79 HintBar at=96,1104,1728,33 by=panel heard="Open, Back, Section, List, Apply…, Mark" limit=0,0,1920,1161 role=group
focus 32
edge 12 up same
edge 12 down 39
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 32
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 33
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 33
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 33
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 33
edge 25 left 23
edge 25 right 12
edge 32 up 17
edge 32 down 39
edge 32 left 17
edge 32 right 33
edge 33 up 19
edge 33 down 39
edge 33 left 32
edge 33 right 12
edge 39 up 32
edge 39 down 44
edge 39 left same
edge 39 right 12
edge 44 up 39
edge 44 down 49
edge 44 left same
edge 44 right 12
edge 49 up 44
edge 49 down 54
edge 49 left same
edge 49 right 12
edge 54 up 49
edge 54 down 59
edge 54 left same
edge 54 right 12
edge 59 up 54
edge 59 down 64
edge 59 left same
edge 59 right 12
edge 64 up 59
edge 64 down same
edge 64 left same
edge 64 right 12
press pageup focus="GridSplitter  (Resize Mark)" top=PackagesWindow
press pagedown focus="GridSplitter  (Resize Mark)" top=PackagesWindow
press first focus="GridSplitter  (Resize Mark)" top=PackagesWindow
press last focus="GridSplitter  (Resize Mark)" top=PackagesWindow
press back top=none
