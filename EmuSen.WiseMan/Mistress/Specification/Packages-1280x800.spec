# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Packages-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Packages
case Packages
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,65,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,149,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,149,67,44 by=panel drawn focus heard=NES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=364,155,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=347,149,67,44 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=422,149,141,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=439,155,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=422,149,141,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=571,149,104,44 by=panel drawn focus heard="Sega CD" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=588,155,70,32 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=571,149,104,44 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=683,149,77,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=700,155,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=683,149,77,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          21 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            22 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              23 LunaTable at=274,206,733,279 by=panel heard=Rows limit=234,124,812.35,548 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                24 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=286,212,38,29 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                25 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=318,210,6,32 focus heard="Resize Mark" paints=#D4D4D4FF role=thumb
                26 Button allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=324,210,336,32 drawn focus heard="Game, not sorted" op paints=#707070FF,#D4D4D4FF role=button
                  27 StackPanel allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=324,210,48,32 role=none
                    28 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=324,210,48,32 box=48,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=47.89,31.2 owner=26 paints=#D4D4D4FF role=text size=26 text=Game
                29 GridSplitter allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=654,210,6,32 focus heard="Resize Game" paints=#D4D4D4FF role=thumb
                30 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,212,335,29 box=335,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                31 ScrollViewer allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,249,733,236 fades=b name=PART_ScrollViewer overflow=16 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=274,249,733,236
                  32 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,249,733,42 drawn focus heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    33 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,254,713,31 heard="Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: OK" in=31 role=none
                      34 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,254,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=32 paints=#D4D4D4FF role=text size=24 text="☐"
                      35 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,255,338,29 box=338,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=336.58,28.8 owner=32 paints=#D4D4D4FF role=edit size=24 text="Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic)"
                      36 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,255,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=20.71,28.8 owner=32 paints=#D4D4D4FF role=edit size=24 text=OK
                  37 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,291,733,42 drawn focus heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    38 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,296,713,31 heard="Mark: not marked, Game: Cobalt Harbor (Synthetic), State: OK" in=31 role=none
                      39 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,296,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=37 paints=#D4D4D4FF role=text size=24 text="☐"
                      40 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,297,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=192.29,28.8 owner=37 paints=#D4D4D4FF role=edit size=24 text="Cobalt Harbor (Synthetic)"
                      41 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,297,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=20.71,28.8 owner=37 paints=#D4D4D4FF role=edit size=24 text=OK
                  42 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,333,733,42 drawn focus heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    43 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,338,713,31 heard="Mark: not marked, Game: Ruby Lantern (Synthetic), State: Changed" in=31 role=none
                      44 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,338,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=42 paints=#D4D4D4FF role=text size=24 text="☐"
                      45 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,339,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=189.96,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text="Ruby Lantern (Synthetic)"
                      46 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,339,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=68.06,28.8 owner=42 paints=#D4D4D4FF role=edit size=24 text=Changed
                  47 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,375,733,42 drawn focus heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    48 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,380,713,31 heard="Mark: not marked, Game: Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3), State: Not packaged" in=31 role=none
                      49 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,380,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=47 paints=#EEEEF0FF role=text size=24 text="☐"
                      50 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,381,338,29 box=338,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Game in=31 layout=337.22,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                      51 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,381,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=107.38,28.8 owner=47 paints=#EEEEF0FF role=edit size=24 text="Not packaged"
                  52 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,417,733,42 drawn focus heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem
                    53 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,422,713,31 heard="Mark: not marked, Game: Ember Choir (USA), State: Not packaged" in=31 role=none
                      54 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,422,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=52 paints=#D4D4D4FF role=text size=24 text="☐"
                      55 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,423,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=140.47,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Ember Choir (USA)"
                      56 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,423,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=107.38,28.8 owner=52 paints=#D4D4D4FF role=edit size=24 text="Not packaged"
                  57 ListBoxItem allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=274,459,733,42 drawn focus heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 op paints=#333337FF,#D4D4D4FF role=listitem seen=274,459,733,26
                    58 Grid allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,464,713,31 heard="Mark: not marked, Game: Amber Saga (USA), State: Not packaged" in=31 role=none seen=284,464,713,21
                      59 TextBlock allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=284,464,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=31 layout=14.4,30.84 owner=57 paints=#D4D4D4FF role=text seen=284,464,38,21 size=24 text="☐"
                      60 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=322,465,338,29 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Game in=31 layout=139.25,28.8 owner=57 paints=#D4D4D4FF role=edit seen=322,465,338,20 size=24 text="Amber Saga (USA)"
                      61 TableCell allow="a game's title or state cut in the Packages table is whole in the detail while its row is chosen" at=660,465,337,29 box=337,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=31 layout=107.38,28.8 owner=57 paints=#D4D4D4FF role=edit seen=660,465,337,20 size=24 text="Not packaged"
        62 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          63 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            64 ActionButton at=274,493,80,44 by=panel drawn focus heard=MAKE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Make
              65 TextBlock at=291,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=45.21,31.2 limit=274,493,80,44 owner=64 paints=#D4D4D4FF role=text size=26 text=MAKE
            66 ActionButton at=366,493,89,44 by=panel drawn focus heard=CHECK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Check
              67 TextBlock at=383,499,55,32 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=54.55,31.2 limit=366,493,89,44 owner=66 paints=#D4D4D4FF role=text size=26 text=CHECK
            68 ActionButton at=467,493,60,44 by=panel drawn focus heard=ZIP limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Zip
              69 TextBlock at=484,499,26,32 box=26,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=25.45,31.2 limit=467,493,60,44 owner=68 paints=#D4D4D4FF role=text size=26 text=ZIP
            70 ActionButton at=539,493,176,44 by=panel drawn focus heard="CONVERT TO CHD" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Convert to CHD"
              71 TextBlock at=556,499,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=141.08,31.2 limit=539,493,176,44 owner=70 paints=#D4D4D4FF role=text size=26 text="CONVERT TO CHD"
            72 ActionButton at=727,493,109,44 by=panel drawn focus heard=REFRESH limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Refresh
              73 TextBlock at=744,499,75,32 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=727,493,109,44 owner=72 paints=#D4D4D4FF role=text size=26 text=REFRESH
            74 ActionButton at=848,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              75 TextBlock at=865,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=848,493,80,44 owner=74 paints=#D4D4D4FF role=text size=26 text=MARK
        76 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          77 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            78 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              79 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            80 StackPanel at=444,586,575,44 by=panel limit=234,124,812.35,548 role=none
              81 Button at=444,586,102,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                82 TextBlock at=461,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=444,586,102,44 off owner=81 paints=#707070FF role=text size=26 text="APPLY…"
              83 StackPanel at=558,586,461,44 by=panel limit=234,124,812.35,548 role=none
                84 ProgressBar at=558,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                85 TextBlock at=766,594,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
        86 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Refresh, Mark" limit=234,124,812.35,548 role=group
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
edge 25 left same
edge 25 right 26
edge 26 up 12
edge 26 down 32
edge 26 left 25
edge 26 right 18
edge 32 up 25
edge 32 down 37
edge 32 left same
edge 32 right same
edge 37 up 32
edge 37 down 42
edge 37 left same
edge 37 right same
edge 42 up 37
edge 42 down 47
edge 42 left same
edge 42 right same
edge 47 up 42
edge 47 down 52
edge 47 left same
edge 47 right same
edge 52 up 47
edge 52 down 57
edge 52 left same
edge 52 right same
edge 57 up 52
edge 57 down 64
edge 57 left same
edge 57 right same
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
edge 74 right same
press pageup focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press pagedown focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press first focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press last focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
press back top=none
