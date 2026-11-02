# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 DockPanel at=64,40,1152,46 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,47,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=64,40,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,46 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,41,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,49,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,41,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,41,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,49,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,41,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,41,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,49,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,41,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,41,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,49,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,41,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,41,127,44 by=panel drawn focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=547,41,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,41,127,44 by=panel drawn focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,41,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,41,112,44 by=panel drawn focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,49,80,24 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=817,41,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,98,1152,468 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                28 Grid at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
                  29 LunaTable at=64,98,1152,468 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 ScrollViewer at=64,109,1152,457 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,109,1152,457
                      31 ListBoxItem at=64,109,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        32 Border at=88,119,1104,33 by=clip in=30 limit=64,109,1152,54 paints=#707070FF role=none
                          33 Grid at=88,119,1104,32 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,109,1152,54 role=none
                            34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,119,108,32 drawn in=30 owner=31 paints=#4C9AE8FF role=text
                            35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,119,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                            36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,119,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="built in"
                      37 ListBoxItem at=64,163,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        38 Border at=88,173,1104,33 by=clip in=30 limit=64,163,1152,54 paints=#707070FF role=none
                          39 Grid at=88,173,1104,32 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,163,1152,54 role=none
                            40 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,173,108,32 drawn in=30 owner=37 paints=#4C9AE8FF role=text
                            41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,173,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=Diamond
                            42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,173,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="built in"
                      43 ListBoxItem at=64,217,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        44 Border at=88,227,1104,33 by=clip in=30 limit=64,217,1152,54 paints=#707070FF role=none
                          45 Grid at=88,227,1104,32 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,217,1152,54 role=none
                            46 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,227,108,32 drawn in=30 owner=43 paints=#4C9AE8FF role=text
                            47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,227,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                            48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,227,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="built in"
                      49 ListBoxItem at=64,271,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        50 Border at=88,281,1104,33 by=clip in=30 limit=64,271,1152,54 paints=#707070FF role=none
                          51 Grid at=88,281,1104,32 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,271,1152,54 role=none
                            52 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,281,108,32 drawn in=30 owner=49 paints=#4C9AE8FF role=text
                            53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,281,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                            54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,281,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=218.35,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="waiting for approval"
                      55 ListBoxItem at=64,325,1152,54 by=fixed-scroll drawn focus heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,109,1152,457 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        56 Border at=88,335,1104,34 by=clip in=30 limit=64,325,1152,54 paints=#707070FF role=none
                          57 Grid at=88,335,1104,33 by=clip heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,325,1152,54 role=none
                            58 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,335,108,33 box=108,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ install" in=30 layout=95.95,32.27 owner=55 paints=#4C9AE8FF role=text size=26 text="✓ install"
                            59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,335,486,32 box=486,32 caps=18.1987 drawn ellipsis face=Barlow heard=Core in=30 layout=479.34,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                            60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,335,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=96.59,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text=available
                      61 ListBoxItem at=64,379,1152,54 by=fixed-scroll drawn focus heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        62 Border at=88,389,1104,34 by=clip in=30 limit=64,379,1152,54 paints=#707070FF role=none
                          63 Grid at=88,389,1104,33 by=clip heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,379,1152,54 role=none
                            64 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,389,108,33 box=108,33 caps=18.916 drawn face="Adwaita Sans" heard="✓ update" in=30 layout=107.34,32.27 owner=61 paints=#4C9AE8FF role=text size=26 text="✓ update"
                            65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,389,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=338.29,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Sony - PlayStation (Beetle PSX)"
                            66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,389,486,33 box=486,33 caps=18.1987 drawn face=Barlow heard=State in=30 layout=103.92,32.27 owner=61 paints=#D4D4D4FF role=edit size=26 text="update ↑"
                      67 ListBoxItem at=64,433,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        68 Border at=88,443,1104,33 by=clip in=30 limit=64,433,1152,54 paints=#707070FF role=none
                          69 Grid at=88,443,1104,32 by=clip heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,433,1152,54 role=none
                            70 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,443,108,32 drawn in=30 owner=67 paints=#4C9AE8FF role=text
                            71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,443,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=448.47,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Nintendo - SNES / SFC (Snes9x - Current)"
                            72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,443,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=92.95,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text=installed
                      73 ListBoxItem at=64,487,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        74 Border at=88,497,1104,33 by=clip in=30 limit=64,487,1152,54 paints=#707070FF role=none
                          75 Grid at=88,497,1104,32 by=clip heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,487,1152,54 role=none
                            76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,497,108,32 drawn in=30 owner=73 paints=#4C9AE8FF role=text
                            77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,497,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=137.85,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Only GL Core"
                            78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,497,486,32 box=486,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=154.31,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="not supported"
            79 Border at=64,574,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              80 WrapPanel at=64,574,1152,96 by=panel limit=0,0,1280,800 role=none
                81 ActionButton at=64,574,101,44 by=panel drawn heard=INSTALL limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Install
                  82 TextBlock at=81,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,574,101,44 off owner=81 paints=#707070FF role=text size=26 text=INSTALL
                83 ActionButton at=177,574,98,44 by=panel drawn heard=UPDATE limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Update
                  84 TextBlock at=194,580,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,574,98,44 off owner=83 paints=#707070FF role=text size=26 text=UPDATE
                85 ActionButton at=287,574,101,44 by=panel drawn heard=REMOVE limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Remove
                  86 TextBlock at=304,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,574,101,44 off owner=85 paints=#707070FF role=text size=26 text=REMOVE
                87 ActionButton at=400,574,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  88 TextBlock at=417,580,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,574,127,44 off owner=87 paints=#707070FF role=text size=26 text="APPROVE…"
                89 ActionButton at=539,574,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  90 TextBlock at=556,580,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,574,155,44 off owner=89 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                91 ActionButton at=706,574,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  92 TextBlock at=723,580,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,574,121,44 off owner=91 paints=#707070FF role=text size=26 text="OPTIONS…"
                93 ActionButton at=839,574,92,44 by=panel drawn heard=VERIFY limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Verify
                  94 TextBlock at=856,580,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,574,92,44 off owner=93 paints=#707070FF role=text size=26 text=VERIFY
                95 ActionButton at=943,574,205,44 by=panel drawn heard="CHECK FOR UPDATES" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  96 TextBlock at=960,580,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,574,205,44 off owner=95 paints=#707070FF role=text size=26 text="CHECK FOR UPDATES"
                97 ActionButton at=64,626,80,44 by=panel drawn heard=MARK limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text=Mark
                  98 TextBlock at=81,632,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,626,80,44 off owner=97 paints=#707070FF role=text size=26 text=MARK
            99 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              100 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                101 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  102 ProgressBar at=52,694,120,4 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                  103 TextBlock at=180,682,1048,29 box=1048,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Installing pcsx2: 18 MB of 41 MB" layout=239.74,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="Installing pcsx2: 18 MB of 41 MB"
                104 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                  105 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                    106 ProgressBar at=823,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    107 TextBlock at=1031,719,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            108 HintBar at=64,760,1152,24 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
focus 7
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
edge 31 up 7
edge 31 down 37
edge 31 left same
edge 31 right same
edge 37 up 31
edge 37 down 43
edge 37 left same
edge 37 right same
edge 43 up 37
edge 43 down 49
edge 43 left same
edge 43 right same
edge 49 up 43
edge 49 down 55
edge 49 left same
edge 49 right same
edge 55 up 49
edge 55 down 61
edge 55 left same
edge 55 right same
edge 61 up 55
edge 61 down 67
edge 61 left same
edge 61 right same
edge 67 up 61
edge 67 down 73
edge 67 left same
edge 67 right same
edge 73 up 67
edge 73 down same
edge 73 left same
edge 73 right same
press pageup focus="TextBox Search cores (Search)" top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, State: built in)" top=CoresWindow
