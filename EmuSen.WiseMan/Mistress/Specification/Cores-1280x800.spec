# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Cores-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case Cores
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,50,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=8 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              10 Border at=64,40,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,44 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,40,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,48,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,40,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,40,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,48,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,40,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,40,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,48,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,40,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,40,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,48,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,40,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,40,127,44 by=panel drawn focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,48,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=547,40,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,40,127,44 by=panel drawn focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,48,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,40,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,40,112,44 by=panel drawn focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,48,80,24 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=817,40,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,96,1152,471 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,96,1152,471 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                28 Grid at=64,96,1152,471 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  29 LunaTable at=64,96,1152,471 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 ScrollViewer at=64,107,1152,460 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,107,1152,460
                      31 ListBoxItem at=64,107,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        32 Border at=88,117,1104,33 by=clip in=30 limit=64,107,1152,54 paints=#707070FF role=none
                          33 Grid at=88,117,1104,32 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,107,1152,54 paints=#4C9AE8FF role=none
                            34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,117,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                            35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,117,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="built in"
                      36 ListBoxItem at=64,161,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        37 Border at=88,171,1104,33 by=clip in=30 limit=64,161,1152,54 paints=#707070FF role=none
                          38 Grid at=88,171,1104,32 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,161,1152,54 paints=#4C9AE8FF role=none
                            39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,171,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text=Diamond
                            40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,171,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="built in"
                      41 ListBoxItem at=64,215,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        42 Border at=88,225,1104,33 by=clip in=30 limit=64,215,1152,54 paints=#707070FF role=none
                          43 Grid at=88,225,1104,32 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,215,1152,54 paints=#4C9AE8FF role=none
                            44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,225,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                            45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,225,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="built in"
                      46 ListBoxItem at=64,269,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        47 Border at=88,279,1104,33 by=clip in=30 limit=64,269,1152,54 paints=#707070FF role=none
                          48 Grid at=88,279,1104,32 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,269,1152,54 paints=#4C9AE8FF role=none
                            49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,279,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                            50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,279,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=218.35,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="waiting for approval"
                      51 ListBoxItem at=64,323,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,107,1152,460 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        52 Border at=88,333,1104,33 by=clip in=30 limit=64,323,1152,54 paints=#707070FF role=none
                          53 Grid at=88,333,1104,32 by=clip heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,323,1152,54 paints=#4C9AE8FF role=none
                            54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,333,516,32 box=516,32 caps=18.1987 drawn ellipsis face=Barlow heard=Core in=30 layout=510.3,31.2 owner=51 paints=#EEEEF0FF role=edit size=26 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                            55 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,333,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=96.59,31.2 owner=51 paints=#EEEEF0FF role=edit size=26 text=available
                      56 ListBoxItem at=64,377,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        57 Border at=88,387,1104,34 by=clip in=30 limit=64,377,1152,54 paints=#707070FF role=none
                          58 Grid at=88,387,1104,33 by=clip heard="Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,377,1152,54 paints=#4C9AE8FF role=none
                            59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,387,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=338.29,31.2 owner=56 paints=#D4D4D4FF role=edit size=26 text="Sony - PlayStation (Beetle PSX)"
                            60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,387,516,33 box=516,33 caps=18.1987 drawn face=Barlow heard=State in=30 layout=103.92,32.27 owner=56 paints=#D4D4D4FF role=edit size=26 text="update ↑"
                      61 ListBoxItem at=64,431,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        62 Border at=88,441,1104,33 by=clip in=30 limit=64,431,1152,54 paints=#707070FF role=none
                          63 Grid at=88,441,1104,32 by=clip heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,431,1152,54 paints=#4C9AE8FF role=none
                            64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,441,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=448.47,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Nintendo - SNES / SFC (Snes9x - Current)"
                            65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,441,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=92.95,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text=installed
                      66 ListBoxItem at=64,485,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        67 Border at=88,495,1104,33 by=clip in=30 limit=64,485,1152,54 paints=#707070FF role=none
                          68 Grid at=88,495,1104,32 by=clip heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,485,1152,54 paints=#4C9AE8FF role=none
                            69 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,495,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=137.85,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="Only GL Core"
                            70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,495,516,32 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=154.31,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="not supported"
            71 Border at=64,575,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              72 WrapPanel at=64,575,1152,96 by=panel limit=0,0,1280,800 role=none
                73 ActionButton at=64,575,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  74 TextBlock at=81,581,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,575,101,44 owner=73 paints=#D4D4D4FF role=text size=26 text=INSTALL
                75 ActionButton at=177,575,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  76 TextBlock at=194,581,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,575,98,44 owner=75 paints=#D4D4D4FF role=text size=26 text=UPDATE
                77 ActionButton at=287,575,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  78 TextBlock at=304,581,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,575,101,44 owner=77 paints=#D4D4D4FF role=text size=26 text=REMOVE
                79 ActionButton at=400,575,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  80 TextBlock at=417,581,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,575,127,44 off owner=79 paints=#707070FF role=text size=26 text="APPROVE…"
                81 ActionButton at=539,575,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  82 TextBlock at=556,581,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,575,155,44 off owner=81 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                83 ActionButton at=706,575,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  84 TextBlock at=723,581,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,575,121,44 off owner=83 paints=#707070FF role=text size=26 text="OPTIONS…"
                85 ActionButton at=839,575,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  86 TextBlock at=856,581,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,575,92,44 owner=85 paints=#D4D4D4FF role=text size=26 text=VERIFY
                87 ActionButton at=943,575,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  88 TextBlock at=960,581,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,575,205,44 owner=87 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                89 ActionButton at=64,627,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  90 TextBlock at=81,633,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,627,80,44 owner=89 paints=#D4D4D4FF role=text size=26 text=MARK
            91 Border at=48,679,1184,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              92 Grid at=52,683,1176,59 by=panel limit=0,0,1280,800 role=none
                93 Grid at=52,683,1176,27 by=panel limit=0,0,1280,800 role=none
                  94 TextBlock at=60,683,1168,27 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                95 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                  96 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                    97 ProgressBar at=813,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    98 TextBlock at=1021,718,207,24 box=207,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 11 MB · 410 GB free" layout=206.6,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 11 MB · 410 GB free"
            99 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              100 HintBar at=96,757,1088,29 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
focus 51
edge 7 up same
edge 7 down 31
edge 7 left 24
edge 7 right same
edge 12 up same
edge 12 down 31
edge 12 left same
edge 12 right 14
edge 14 up same
edge 14 down 31
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 31
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 31
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 31
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 31
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 31
edge 24 left 22
edge 24 right 7
edge 31 up 12
edge 31 down 36
edge 31 left same
edge 31 right same
edge 36 up 31
edge 36 down 41
edge 36 left same
edge 36 right same
edge 41 up 36
edge 41 down 46
edge 41 left same
edge 41 right same
edge 46 up 41
edge 46 down 51
edge 46 left same
edge 46 right same
edge 51 up 46
edge 51 down 56
edge 51 left same
edge 51 right same
edge 56 up 51
edge 56 down 61
edge 56 left same
edge 56 right same
edge 61 up 56
edge 61 down 66
edge 61 left same
edge 61 right same
edge 66 up 61
edge 66 down 73
edge 66 left same
edge 66 right same
edge 73 up 66
edge 73 down 89
edge 73 left same
edge 73 right 75
edge 75 up 66
edge 75 down 89
edge 75 left 73
edge 75 right 77
edge 77 up 66
edge 77 down 89
edge 77 left 75
edge 77 right 85
edge 85 up 66
edge 85 down 89
edge 85 left 77
edge 85 right 87
edge 87 up 66
edge 87 down 89
edge 87 left 85
edge 87 right same
edge 89 up 73
edge 89 down same
edge 89 left same
edge 89 right 75
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, State: built in)" top=CoresWindow
