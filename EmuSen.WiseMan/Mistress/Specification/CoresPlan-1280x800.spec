# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,44 by=panel focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,50,158,24 box=158,24 caps=13.999 face=Barlow heard="Search cores" in=8 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              10 Border at=64,40,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,44 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,40,66,44 by=panel focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,48,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=64,40,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,40,146,44 by=panel focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,48,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,40,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,40,160,44 by=panel focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,48,128,24 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,40,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,40,79,44 by=panel focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,48,47,24 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=460,40,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,40,127,44 by=panel focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,48,95,24 box=95,24 by=clip caps=13.999 face=Barlow heard=INSTALLED layout=94.8,24 limit=547,40,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,40,127,44 by=panel focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,48,95,24 box=95,24 by=clip caps=13.999 face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,40,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,40,112,44 by=panel focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,48,80,24 box=80,24 by=clip caps=13.999 face=Barlow heard=UPDATES layout=79.8,24 limit=817,40,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,96,1152,471 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,96,1152,471 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                28 Grid at=64,96,1152,471 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                  29 LunaTable at=64,96,1152,471 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 ScrollViewer at=64,107,1152,460 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,107,1152,460
                      31 ListBoxItem at=64,107,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        32 Border at=88,117,1104,33 by=clip in=30 limit=64,107,1152,54 paints=#707070FF role=none
                          33 Grid at=88,117,1104,32 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,107,1152,54 role=none
                            34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,117,108,32 in=30 owner=31 paints=#4C9AE8FF role=text
                            35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,117,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                            36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,117,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=74.02,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="built in"
                      37 ListBoxItem at=64,161,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        38 Border at=88,171,1104,33 by=clip in=30 limit=64,161,1152,54 paints=#707070FF role=none
                          39 Grid at=88,171,1104,32 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,161,1152,54 role=none
                            40 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,171,108,32 in=30 owner=37 paints=#4C9AE8FF role=text
                            41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,171,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=98.67,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=Diamond
                            42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,171,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=74.02,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text="built in"
                      43 ListBoxItem at=64,215,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        44 Border at=88,225,1104,33 by=clip in=30 limit=64,215,1152,54 paints=#707070FF role=none
                          45 Grid at=88,225,1104,32 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,215,1152,54 role=none
                            46 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,225,108,32 in=30 owner=43 paints=#4C9AE8FF role=text
                            47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,225,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=95.08,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                            48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,225,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=74.02,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="built in"
                      49 ListBoxItem at=64,269,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        50 Border at=88,279,1104,33 by=clip in=30 limit=64,269,1152,54 paints=#707070FF role=none
                          51 Grid at=88,279,1104,32 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,269,1152,54 role=none
                            52 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,279,108,32 in=30 owner=49 paints=#4C9AE8FF role=text
                            53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,279,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=162.4,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                            54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,279,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=218.35,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="waiting for approval"
                      55 ListBoxItem at=64,323,1152,54 by=fixed-scroll focus heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,107,1152,460 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                        56 Border at=88,333,1104,34 by=clip in=30 limit=64,323,1152,54 paints=#707070FF role=none
                          57 Grid at=88,333,1104,33 by=clip heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=30 limit=64,323,1152,54 role=none
                            58 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,333,108,33 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ install" in=30 layout=95.95,32.27 owner=55 paints=#4C9AE8FF role=text size=26 text="✓ install"
                            59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,333,462,32 box=462,32 caps=18.1987 ellipsis face=Barlow heard=Core in=30 layout=459.06,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                            60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,333,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=96.59,31.2 owner=55 paints=#EEEEF0FF role=edit size=26 text=available
                      61 ListBoxItem at=64,377,1152,54 by=fixed-scroll focus heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        62 Border at=88,387,1104,34 by=clip in=30 limit=64,377,1152,54 paints=#707070FF role=none
                          63 Grid at=88,387,1104,33 by=clip heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=30 limit=64,377,1152,54 role=none
                            64 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,387,108,33 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ update" in=30 layout=107.34,32.27 owner=61 paints=#4C9AE8FF role=text size=26 text="✓ update"
                            65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,387,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=338.29,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Sony - PlayStation (Beetle PSX)"
                            66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,387,462,33 box=462,33 caps=18.1987 face=Barlow heard=State in=30 layout=103.92,32.27 owner=61 paints=#D4D4D4FF role=edit size=26 text="update ↑"
                      67 ListBoxItem at=64,431,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        68 Border at=88,441,1104,33 by=clip in=30 limit=64,431,1152,54 paints=#707070FF role=none
                          69 Grid at=88,441,1104,32 by=clip heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=30 limit=64,431,1152,54 role=none
                            70 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,441,108,32 in=30 owner=67 paints=#4C9AE8FF role=text
                            71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,441,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=448.47,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Nintendo - SNES / SFC (Snes9x - Current)"
                            72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,441,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=92.95,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text=installed
                      73 ListBoxItem at=64,485,1152,54 by=fixed-scroll focus heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,107,1152,460 op paints=#D4D4D4FF role=listitem
                        74 Border at=88,495,1104,33 by=clip in=30 limit=64,485,1152,54 paints=#707070FF role=none
                          75 Grid at=88,495,1104,32 by=clip heard="Mark: not marked, Core: Only GL Core, State: not supported" in=30 limit=64,485,1152,54 role=none
                            76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,495,108,32 in=30 owner=73 paints=#4C9AE8FF role=text
                            77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,495,462,32 box=462,32 caps=18.1987 face=Barlow heard=Core in=30 layout=137.85,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Only GL Core"
                            78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=706,495,462,32 box=462,32 caps=18.1987 face=Barlow heard=State in=30 layout=154.31,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="not supported"
            79 Border at=64,575,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              80 WrapPanel at=64,575,1152,96 by=panel limit=0,0,1280,800 role=none
                81 ActionButton at=64,575,101,44 by=panel focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  82 TextBlock at=81,581,67,32 box=67,32 by=clip caps=18.1987 face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,575,101,44 owner=81 paints=#D4D4D4FF role=text size=26 text=INSTALL
                83 ActionButton at=177,575,98,44 by=panel focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  84 TextBlock at=194,581,64,32 box=64,32 by=clip caps=18.1987 face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,575,98,44 owner=83 paints=#D4D4D4FF role=text size=26 text=UPDATE
                85 ActionButton at=287,575,101,44 by=panel focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  86 TextBlock at=304,581,67,32 box=67,32 by=clip caps=18.1987 face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,575,101,44 owner=85 paints=#D4D4D4FF role=text size=26 text=REMOVE
                87 ActionButton at=400,575,127,44 by=panel heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  88 TextBlock at=417,581,93,32 box=93,32 by=clip caps=18.1987 face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,575,127,44 off owner=87 paints=#707070FF role=text size=26 text="APPROVE…"
                89 ActionButton at=539,575,155,44 by=panel heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  90 TextBlock at=556,581,121,32 box=121,32 by=clip caps=18.1987 face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,575,155,44 off owner=89 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                91 ActionButton at=706,575,121,44 by=panel heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  92 TextBlock at=723,581,87,32 box=87,32 by=clip caps=18.1987 face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,575,121,44 off owner=91 paints=#707070FF role=text size=26 text="OPTIONS…"
                93 ActionButton at=839,575,92,44 by=panel focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  94 TextBlock at=856,581,58,32 box=58,32 by=clip caps=18.1987 face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,575,92,44 owner=93 paints=#D4D4D4FF role=text size=26 text=VERIFY
                95 ActionButton at=943,575,205,44 by=panel focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  96 TextBlock at=960,581,171,32 box=171,32 by=clip caps=18.1987 face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,575,205,44 owner=95 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                97 ActionButton at=64,627,80,44 by=panel focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  98 TextBlock at=81,633,46,32 box=46,32 by=clip caps=18.1987 face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,627,80,44 owner=97 paints=#D4D4D4FF role=text size=26 text=MARK
            99 Border at=48,679,1184,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              100 Grid at=52,683,1176,59 by=panel limit=0,0,1280,800 role=none
                101 Grid at=52,683,1176,27 by=panel limit=0,0,1280,800 role=none
                  102 TextBlock at=60,683,1168,27 box=1168,27 by=panel caps=15.3989 face=Barlow heard="2 marked: install 1, update 1, about 8 MB to download" layout=479.95,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 marked: install 1, update 1, about 8 MB to download"
                103 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                  104 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                    105 ProgressBar at=813,723,200,14 by=panel heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    106 TextBlock at=1021,718,207,24 box=207,24 by=panel caps=13.999 face=Barlow heard="/Cores 11 MB · 410 GB free" layout=206.6,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 11 MB · 410 GB free"
            107 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              108 HintBar at=96,757,1088,29 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          109 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveCover role=none
            110 Border at=280,184,720,453 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              111 DockPanel at=313,217,654,387 by=panel limit=0,0,1280,800 role=none
                112 TextBlock at=313,217,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                113 StackPanel at=313,492,654,112 by=panel limit=0,0,1280,800 role=none
                  114 Button at=313,492,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    115 TextBlock at=341,503,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,492,654,54 owner=114 paints=#EEEEF0FF role=text size=26 text=No
                  116 Button at=313,550,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    117 TextBlock at=341,561,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,550,654,54 owner=116 paints=#D4D4D4FF role=text size=26 text=Yes
                118 ScrollViewer at=313,268,654,212 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,268,654,212
                  119 TextBlock at=313,268,654,212 box=654,212 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=118 layout=639.98,211.2 limit=313,268,654,212 paints=#D4D4D4FF role=text size=22 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
focus 114
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
edge 73 down 116
edge 73 left same
edge 73 right same
edge 81 up 73
edge 81 down 97
edge 81 left same
edge 81 right 83
edge 83 up 73
edge 83 down 97
edge 83 left 81
edge 83 right 85
edge 85 up 116
edge 85 down 97
edge 85 left 83
edge 85 right 116
edge 93 up 116
edge 93 down 97
edge 93 left 116
edge 93 right 95
edge 95 up 73
edge 95 down 97
edge 95 left 93
edge 95 right same
edge 97 up 81
edge 97 down same
edge 97 left same
edge 97 right 83
edge 114 up 67
edge 114 down 116
edge 114 left 83
edge 114 right 73
edge 116 up 114
edge 116 down 85
edge 116 left 83
edge 116 right 95
press pageup focus="Button  (No)" top=CoresWindow
press pagedown focus="Button  (No)" top=CoresWindow
press first focus="Button  (No)" top=CoresWindow
press last focus="Button  (No)" top=CoresWindow
press back focus="ListBoxItem  (Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available)" top=CoresWindow
