# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Packages
case Packages
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 PackagesPane at=386,205,1149,804 by=panel heard=Packages limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,99,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=410,224.5,99,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=521,224.5,103.5,66 by=panel drawn focus heard=NES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=546.5,233.5,52.5,48 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=34.79,31.2 limit=521,224.5,103.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=636.5,224.5,223.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=662,233.5,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=636.5,224.5,223.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=872,224.5,160.5,66 by=panel drawn focus heard="Sega CD" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=897.5,233.5,109.5,48 box=73,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=72.96,31.2 limit=872,224.5,160.5,66 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=1044.5,224.5,120,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=1070,233.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=1044.5,224.5,120,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
          21 Border at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            22 Grid at=410,310,1101,418.5 by=panel limit=350.48,187,1219.05,822 role=none
              23 LunaTable at=410,310,1101,418.5 by=panel heard=Rows limit=350.48,187,1219.05,822 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                24 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=428,319,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                25 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=479,316,9,48 focus heard="Resize Mark" paints=#9C9CA0FF role=thumb
                26 Button allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=488,316,502.5,48 drawn focus heard="Game, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  27 StackPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=488,316,75,48 role=none
                    28 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=488,316,75,48 box=50,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=49.89,31.2 owner=26 paints=#9C9CA0FF role=text size=26 text=Game
                29 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=981.5,316,9,48 focus heard="Resize Game" paints=#9C9CA0FF role=thumb
                30 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,319,502.5,43.5 box=335,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=42.48,28.8 paints=#9C9CA0FF role=text size=24 text=State
                31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,374.5,1101,354 fades=b name=PART_ScrollViewer overflow=24 paints=#55555AFF,#FFFFFFFF part role=scrollviewer viewport=410,374.5,1101,354
                  32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,374.5,1101,63 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 op paints=#333337FF,#9C9CA0FF role=listitem
                    33 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,382,1071,46.5 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 role=none
                      34 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,382,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=32 paints=#9C9CA0FF role=text size=24 text="☐"
                      35 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,383.5,505.5,43.5 box=337,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=327.86,28.8 owner=32 paints=#9C9CA0FF role=edit size=24 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                      36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,383.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=21.79,28.8 owner=32 paints=#9C9CA0FF role=edit size=24 text=OK
                  37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,437.5,1101,63 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 op paints=#333337FF,#9C9CA0FF role=listitem
                    38 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,445,1071,46.5 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 role=none
                      39 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,445,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=37 paints=#9C9CA0FF role=text size=24 text="☐"
                      40 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,446.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=203.04,28.8 owner=37 paints=#9C9CA0FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,446.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=21.79,28.8 owner=37 paints=#9C9CA0FF role=edit size=24 text=OK
                  42 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,500.5,1101,63 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 op paints=#333337FF,#9C9CA0FF role=listitem
                    43 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,508,1071,46.5 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 role=none
                      44 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,508,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=42 paints=#9C9CA0FF role=text size=24 text="☐"
                      45 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,509.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=199.15,28.8 owner=42 paints=#9C9CA0FF role=edit size=24 text="Ruby Lantern (Synthetic)"
                      46 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,509.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=70.8,28.8 owner=42 paints=#9C9CA0FF role=edit size=24 text=Changed
                  47 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,563.5,1101,63 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#333337FF,#9C9CA0FF role=listitem
                    48 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,571,1071,46.5 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                      49 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,571,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=47 paints=#EEEEF0FF role=text size=24 text="☐"
                      50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,572.5,505.5,43.5 box=337,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=328.58,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      51 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,572.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=112.3,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Not packaged"
                  52 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,626.5,1101,63 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#333337FF,#9C9CA0FF role=listitem
                    53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,634,1071,46.5 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                      54 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,634,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=52 paints=#9C9CA0FF role=text size=24 text="☐"
                      55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,635.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=147.12,28.8 owner=52 paints=#9C9CA0FF role=edit size=24 text="Ember Choir (USA)"
                      56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,635.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=112.3,28.8 owner=52 paints=#9C9CA0FF role=edit size=24 text="Not packaged"
                  57 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=410,689.5,1101,63 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 op paints=#333337FF,#9C9CA0FF role=listitem seen=410,689.5,1101,39
                    58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,697,1071,46.5 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 role=none seen=425,697,1071,31.5
                      59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=425,697,60,46.5 box=40,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=57 paints=#9C9CA0FF role=text seen=425,697,60,31.5 size=24 text="☐"
                      60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=485,698.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=143.54,28.8 owner=57 paints=#9C9CA0FF role=edit seen=485,698.5,505.5,30 size=24 text="Amber Saga (USA)"
                      61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=990.5,698.5,505.5,43.5 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=112.3,28.8 owner=57 paints=#9C9CA0FF role=edit seen=990.5,698.5,505.5,30 size=24 text="Not packaged"
        62 Border at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          63 WrapPanel at=410,740.5,1101,66 by=panel limit=350.48,187,1219.05,822 role=none
            64 ActionButton at=410,740.5,123,66 by=panel drawn focus heard=MAKE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Make
              65 TextBlock at=435.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=47.11,31.2 limit=410,740.5,123,66 owner=64 paints=#9C9CA0FF role=text size=26 text=MAKE
            66 ActionButton at=551,740.5,139.5,66 by=panel drawn focus heard=CHECK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Check
              67 TextBlock at=576.5,749.5,88.5,48 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=58.19,31.2 limit=551,740.5,139.5,66 owner=66 paints=#9C9CA0FF role=text size=26 text=CHECK
            68 ActionButton at=708.5,740.5,93,66 by=panel drawn focus heard=ZIP limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
              69 TextBlock at=734,749.5,42,48 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=27.35,31.2 limit=708.5,740.5,93,66 owner=68 paints=#9C9CA0FF role=text size=26 text=ZIP
            70 ActionButton at=819.5,740.5,276,66 by=panel drawn focus heard="CONVERT TO CHD" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
              71 TextBlock at=845,749.5,225,48 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=149.58,31.2 limit=819.5,740.5,276,66 owner=70 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
            72 ActionButton at=1113.5,740.5,169.5,66 by=panel drawn focus heard=REFRESH limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
              73 TextBlock at=1139,749.5,118.5,48 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=1113.5,740.5,169.5,66 owner=72 paints=#9C9CA0FF role=text size=26 text=REFRESH
            74 ActionButton at=1301,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              75 TextBlock at=1326.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1301,740.5,123,66 owner=74 paints=#9C9CA0FF role=text size=26 text=MARK
        76 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          77 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            78 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              79 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            80 StackPanel at=648.5,880,880.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              81 Button at=648.5,880,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                82 TextBlock at=674,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=648.5,880,157.5,66 off owner=81 paints=#55555AFF role=text size=26 text="APPLY…"
              83 StackPanel at=824,880,705,66 by=panel limit=350.48,187,1219.05,822 role=none
                84 ProgressBar at=824,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                85 TextBlock at=1136,892,393,43.5 box=262,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=261.82,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="263 KB in packages · 412 GB free"
        86 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=350.48,187,1219.05,822 role=group
focus 25
edge 5 up same
edge 5 down 32
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 25
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 26
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 26
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 26
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 32
edge 18 left 16
edge 18 right 5
edge 25 up 10
edge 25 down 32
edge 25 left 10
edge 25 right 26
edge 26 up 12
edge 26 down 32
edge 26 left 25
edge 26 right 18
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
edge 42 right 74
edge 47 up 42
edge 47 down 52
edge 47 left same
edge 47 right 74
edge 52 up 47
edge 52 down 57
edge 52 left same
edge 52 right 74
edge 57 up 52
edge 57 down 64
edge 57 left same
edge 57 right 74
edge 64 up 57
edge 64 down same
edge 64 left same
edge 64 right 66
edge 66 up 57
edge 66 down same
edge 66 left 64
edge 66 right 68
edge 68 up 57
edge 68 down same
edge 68 left 66
edge 68 right 70
edge 70 up 57
edge 70 down same
edge 70 left 68
edge 70 right 72
edge 72 up 57
edge 72 down same
edge 72 left 70
edge 72 right 74
edge 74 up 57
edge 74 down same
edge 74 left 72
edge 74 right 57
press pageup focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press pagedown focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press first focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press last focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press back top=none
