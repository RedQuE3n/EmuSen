# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresNoCatalogue
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
                          33 Grid at=88,119,1104,32 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=64,109,1152,54 paints=#4C9AE8FF role=none
                            34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,119,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                            35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,119,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="built in"
                      36 ListBoxItem at=64,163,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        37 Border at=88,173,1104,33 by=clip in=30 limit=64,163,1152,54 paints=#707070FF role=none
                          38 Grid at=88,173,1104,32 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=64,163,1152,54 paints=#4C9AE8FF role=none
                            39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,173,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text=Diamond
                            40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,173,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="built in"
                      41 ListBoxItem at=64,217,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        42 Border at=88,227,1104,33 by=clip in=30 limit=64,217,1152,54 paints=#707070FF role=none
                          43 Grid at=88,227,1104,32 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=64,217,1152,54 paints=#4C9AE8FF role=none
                            44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,227,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                            45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,227,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="built in"
                      46 ListBoxItem at=64,271,1152,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,109,1152,457 op paints=#D4D4D4FF role=listitem
                        47 Border at=88,281,1104,33 by=clip in=30 limit=64,271,1152,54 paints=#707070FF role=none
                          48 Grid at=88,281,1104,32 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=64,271,1152,54 paints=#4C9AE8FF role=none
                            49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,281,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                            50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=652,281,540,32 box=540,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=218.35,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="waiting for approval"
            51 Border at=64,574,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              52 WrapPanel at=64,574,1152,96 by=panel limit=0,0,1280,800 role=none
                53 ActionButton at=64,574,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  54 TextBlock at=81,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,574,101,44 owner=53 paints=#D4D4D4FF role=text size=26 text=INSTALL
                55 ActionButton at=177,574,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  56 TextBlock at=194,580,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,574,98,44 owner=55 paints=#D4D4D4FF role=text size=26 text=UPDATE
                57 ActionButton at=287,574,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  58 TextBlock at=304,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,574,101,44 owner=57 paints=#D4D4D4FF role=text size=26 text=REMOVE
                59 ActionButton at=400,574,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  60 TextBlock at=417,580,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,574,127,44 off owner=59 paints=#707070FF role=text size=26 text="APPROVE…"
                61 ActionButton at=539,574,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  62 TextBlock at=556,580,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,574,155,44 off owner=61 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                63 ActionButton at=706,574,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  64 TextBlock at=723,580,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,574,121,44 off owner=63 paints=#707070FF role=text size=26 text="OPTIONS…"
                65 ActionButton at=839,574,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  66 TextBlock at=856,580,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,574,92,44 owner=65 paints=#D4D4D4FF role=text size=26 text=VERIFY
                67 ActionButton at=943,574,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  68 TextBlock at=960,580,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,574,205,44 owner=67 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                69 ActionButton at=64,626,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  70 TextBlock at=81,632,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,626,80,44 owner=69 paints=#D4D4D4FF role=text size=26 text=MARK
            71 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              72 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                73 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  74 TextBlock at=60,682,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=697.66,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
                75 StackPanel at=837,719,391,29 by=panel limit=0,0,1280,800 role=none
                  76 StackPanel at=837,719,391,29 by=panel limit=0,0,1280,800 role=none
                    77 ProgressBar at=837,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    78 TextBlock at=1045,719,183,29 box=183,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 0 B · 410 GB free" layout=182.88,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="/Cores 0 B · 410 GB free"
            79 HintBar at=64,760,1152,24 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
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
edge 46 down 53
edge 46 left same
edge 46 right same
edge 53 up 46
edge 53 down 69
edge 53 left same
edge 53 right 55
edge 55 up 46
edge 55 down 69
edge 55 left 53
edge 55 right 57
edge 57 up 46
edge 57 down 69
edge 57 left 55
edge 57 right 59
edge 59 up 46
edge 59 down 69
edge 59 left 57
edge 59 right 61
edge 61 up 46
edge 61 down 69
edge 61 left 59
edge 61 right 63
edge 63 up 46
edge 63 down 69
edge 63 left 61
edge 63 right 65
edge 65 up 46
edge 65 down 69
edge 65 left 63
edge 65 right 67
edge 67 up 46
edge 67 down 69
edge 67 left 65
edge 67 right same
edge 69 up 53
edge 69 down same
edge 69 left same
edge 69 right 55
press pageup focus="TextBox Search cores (Search)" top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, State: built in)" top=CoresWindow
