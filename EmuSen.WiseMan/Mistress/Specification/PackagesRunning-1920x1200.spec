# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Packages
case PackagesRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 PackagesPane at=386,205,1149,804 by=panel heard=Packages limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,97.5,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=410,224.5,97.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=519.5,224.5,100.5,66 by=panel drawn focus heard=NES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=545,233.5,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=519.5,224.5,100.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=632,224.5,211.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=657.5,233.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=632,224.5,211.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=855.5,224.5,156,66 by=panel drawn focus heard="Sega CD" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=881,233.5,105,48 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=855.5,224.5,156,66 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=1023.5,224.5,115.5,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=1049,233.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=1023.5,224.5,115.5,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
          21 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            22 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              23 LunaTable at=410,310,1101,418.5 by=panel heard=Rows limit=350.48,187,1219.05,822 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                24 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=428,319,214.5,43.5 box=143,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                25 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=633.5,316,9,48 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                26 Button allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=642.5,316,426,48 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  27 StackPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=642.5,316,72,48 role=none
                    28 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=642.5,316,72,48 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=26 paints=#D4D4D4FF role=text size=26 text=Game
                29 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1059.5,316,9,48 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                30 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,319,424.5,43.5 box=283,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,374.5,1101,354 fades=b name=PART_ScrollViewer overflow=24 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=410,374.5,1101,354
                  32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,374.5,1101,63 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    33 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,382,1071,46.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 role=none
                      34 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,382,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=32 paints=#D4D4D4FF role=text size=24 text="☐"
                      35 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,383.5,429,43.5 box=286,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=283.61,28.8 owner=32 paints=#D4D4D4FF role=edit size=24 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                      36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,383.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=20.71,28.8 owner=32 paints=#D4D4D4FF role=edit size=24 text=OK
                  37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,437.5,1101,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    38 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,445,1071,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 role=none
                      39 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,445,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=37 paints=#D4D4D4FF role=text size=24 text="☐"
                      40 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,446.5,429,43.5 box=286,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=192.29,28.8 owner=37 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,446.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=20.71,28.8 owner=37 paints=#D4D4D4FF role=edit size=24 text=OK
                  42 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,500.5,1101,63 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    43 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,508,1071,46.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 role=none
                      44 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,508,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=42 paints=#D4D4D4FF role=text size=24 text="☐"
                      45 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,509.5,429,43.5 box=286,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=189.96,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text="Ruby Lantern (Synthetic)"
                      46 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,509.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=68.06,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text=Changed
                  47 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,563.5,1101,63 drawn focus heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    48 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,571,1071,46.5 heard="Mark: convert to CHD, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                      49 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,571,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ convert to CHD" in=31 layout=142.97,30.84 owner=47 paints=#EEEEF0FF role=text size=24 text="☑ convert to CHD"
                      50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,572.5,429,43.5 box=286,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=282.62,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      51 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,572.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=107.38,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Not packaged"
                  52 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,626.5,1101,63 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,634,1071,46.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                      54 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,634,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=52 paints=#D4D4D4FF role=text size=24 text="☐"
                      55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,635.5,429,43.5 box=286,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=140.47,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Ember Choir (USA)"
                      56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,635.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=107.38,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Not packaged"
                  57 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,689.5,1101,63 drawn focus heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 op paints=#333337FF,#D4D4D4FF role=listitem seen=410,689.5,1101,39
                    58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,697,1071,46.5 heard="Mark: make, linked, Game: Amber Saga (USA), State: Made, linked to the game where it is" in=31 role=none seen=425,697,1071,31.5
                      59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,697,214.5,46.5 box=143,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ make, linked" in=31 layout=126.65,30.84 owner=57 paints=#D4D4D4FF role=text seen=425,697,214.5,31.5 size=24 text="☑ make, linked"
                      60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=639.5,698.5,429,43.5 box=286,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=139.25,28.8 owner=57 paints=#D4D4D4FF role=edit seen=639.5,698.5,429,30 size=24 text="Amber Saga (USA)"
                      61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=1068.5,698.5,427.5,43.5 box=285,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=278.52,28.8 owner=57 paints=#D4D4D4FF role=edit seen=1068.5,698.5,427.5,30 size=24 text="Made, linked to the game where it is"
        62 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          63 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            64 ActionButton at=410,740.5,120,66 by=panel drawn heard=MAKE limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Make
              65 TextBlock at=435.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=45.21,31.2 limit=410,740.5,120,66 off owner=64 paints=#707070FF role=text size=26 text=MAKE
            66 ActionButton at=548,740.5,133.5,66 by=panel drawn heard=CHECK limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Check
              67 TextBlock at=573.5,749.5,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=54.55,31.2 limit=548,740.5,133.5,66 off owner=66 paints=#707070FF role=text size=26 text=CHECK
            68 ActionButton at=699.5,740.5,90,66 by=panel drawn heard=ZIP limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Zip
              69 TextBlock at=725,749.5,39,48 box=26,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=25.45,31.2 limit=699.5,740.5,90,66 off owner=68 paints=#707070FF role=text size=26 text=ZIP
            70 ActionButton at=807.5,740.5,264,66 by=panel drawn heard="CONVERT TO CHD" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Convert to CHD"
              71 TextBlock at=833,749.5,213,48 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=141.08,31.2 limit=807.5,740.5,264,66 off owner=70 paints=#707070FF role=text size=26 text="CONVERT TO CHD"
            72 ActionButton at=1089.5,740.5,163.5,66 by=panel drawn heard=REFRESH limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Refresh
              73 TextBlock at=1115,749.5,112.5,48 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=1089.5,740.5,163.5,66 off owner=72 paints=#707070FF role=text size=26 text=REFRESH
            74 ActionButton at=1271,740.5,120,66 by=panel drawn heard=MARK limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Mark
              75 TextBlock at=1296.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1271,740.5,120,66 off owner=74 paints=#707070FF role=text size=26 text=MARK
        76 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          77 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            78 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              79 ProgressBar at=392,842.5,180,6 by=panel drawn heard=Progress limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
              80 TextBlock at=584,824.5,945,43.5 box=630,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Making packages: 1 of 2, Amber Saga (USA)" layout=331.06,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Making packages: 1 of 2, Amber Saga (USA)"
            81 StackPanel at=666.5,880,862.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              82 Button at=666.5,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                83 TextBlock at=692,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=666.5,880,153,66 off owner=82 paints=#707070FF role=text size=26 text="APPLY…"
              84 StackPanel at=837.5,880,691.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                85 ProgressBar at=837.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                86 TextBlock at=1149.5,892,379.5,43.5 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
        87 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 5
edge 5 up same
edge 5 down 32
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 32
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 32
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 25
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 26
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 26
edge 18 left 16
edge 18 right 5
edge 25 up 14
edge 25 down 32
edge 25 left 12
edge 25 right 26
edge 26 up 14
edge 26 down 32
edge 26 left 25
edge 26 right 5
edge 32 up 25
edge 32 down 37
edge 32 left same
edge 32 right 5
edge 37 up 32
edge 37 down 42
edge 37 left same
edge 37 right 5
edge 42 up 37
edge 42 down 47
edge 42 left same
edge 42 right 5
edge 47 up 42
edge 47 down 52
edge 47 left same
edge 47 right 5
edge 52 up 47
edge 52 down 57
edge 52 left same
edge 52 right 5
edge 57 up 52
edge 57 down same
edge 57 left same
edge 57 right 5
press pageup focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
