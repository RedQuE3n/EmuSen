# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case CoresNoCatalogue
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1146 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 DockPanel at=96,60,1728,66 by=panel limit=0,0,1920,1200 role=none
              6 Border at=1554,60,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                7 TextBox at=1554,60,270,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1570.5,70.5,237,45 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1570.5,75,237,36 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=8 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              10 Border at=96,60,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                11 WholeRow at=96,60,1458,66 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
                  12 ToggleButton at=96,60,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=120,72,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,60,99,66 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=207,60,219,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=231,72,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=207,60,219,66 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=438,60,240,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=462,72,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=438,60,240,66 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=690,60,118.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=714,72,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=690,60,118.5,66 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=820.5,60,190.5,66 by=panel drawn focus heard=Installed limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=844.5,72,142.5,36 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=820.5,60,190.5,66 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=1023,60,190.5,66 by=panel drawn focus heard=Available limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=1047,72,142.5,36 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=1023,60,190.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=1225.5,60,168,66 by=panel drawn focus heard=Updates limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=1249.5,72,120,36 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=1225.5,60,168,66 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=96,144,1728,706.5 by=panel limit=0,0,1920,1200 role=none
              27 Border at=96,144,1728,706.5 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                28 Grid at=96,144,1728,706.5 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                  29 LunaTable at=96,144,1728,706.5 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 ScrollViewer at=96,160.5,1728,690 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,160.5,1728,690
                      31 ListBoxItem at=96,160.5,1728,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=96,160.5,1728,690 op paints=#D4D4D4FF role=listitem
                        32 Border at=132,175.5,1656,49.5 by=clip in=30 limit=96,160.5,1728,81 paints=#707070FF role=none
                          33 Grid at=132,175.5,1656,48 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=30 limit=96,160.5,1728,81 paints=#4C9AE8FF role=none
                            34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,175.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                            35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=978,175.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="built in"
                      36 ListBoxItem at=96,241.5,1728,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=96,160.5,1728,690 op paints=#D4D4D4FF role=listitem
                        37 Border at=132,256.5,1656,49.5 by=clip in=30 limit=96,241.5,1728,81 paints=#707070FF role=none
                          38 Grid at=132,256.5,1656,48 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=30 limit=96,241.5,1728,81 paints=#4C9AE8FF role=none
                            39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,256.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text=Diamond
                            40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=978,256.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=36 paints=#D4D4D4FF role=edit size=26 text="built in"
                      41 ListBoxItem at=96,322.5,1728,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=96,160.5,1728,690 op paints=#D4D4D4FF role=listitem
                        42 Border at=132,337.5,1656,49.5 by=clip in=30 limit=96,322.5,1728,81 paints=#707070FF role=none
                          43 Grid at=132,337.5,1656,48 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=30 limit=96,322.5,1728,81 paints=#4C9AE8FF role=none
                            44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,337.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                            45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=978,337.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=74.02,31.2 owner=41 paints=#D4D4D4FF role=edit size=26 text="built in"
                      46 ListBoxItem at=96,403.5,1728,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=96,160.5,1728,690 op paints=#D4D4D4FF role=listitem
                        47 Border at=132,418.5,1656,49.5 by=clip in=30 limit=96,403.5,1728,81 paints=#707070FF role=none
                          48 Grid at=132,418.5,1656,48 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=30 limit=96,403.5,1728,81 paints=#4C9AE8FF role=none
                            49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,418.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                            50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=978,418.5,774,48 box=516,32 caps=18.1987 drawn face=Barlow heard=State in=30 layout=218.35,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text="waiting for approval"
            51 Border at=96,862.5,1728,144 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              52 WrapPanel at=96,862.5,1728,144 by=panel limit=0,0,1920,1200 role=none
                53 ActionButton at=96,862.5,151.5,66 by=panel drawn focus heard=INSTALL limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  54 TextBlock at=121.5,871.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=96,862.5,151.5,66 owner=53 paints=#D4D4D4FF role=text size=26 text=INSTALL
                55 ActionButton at=265.5,862.5,147,66 by=panel drawn focus heard=UPDATE limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  56 TextBlock at=291,871.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=265.5,862.5,147,66 owner=55 paints=#D4D4D4FF role=text size=26 text=UPDATE
                57 ActionButton at=430.5,862.5,151.5,66 by=panel drawn focus heard=REMOVE limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  58 TextBlock at=456,871.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=430.5,862.5,151.5,66 owner=57 paints=#D4D4D4FF role=text size=26 text=REMOVE
                59 ActionButton at=600,862.5,190.5,66 by=panel drawn heard="APPROVE…" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  60 TextBlock at=625.5,871.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=600,862.5,190.5,66 off owner=59 paints=#707070FF role=text size=26 text="APPROVE…"
                61 ActionButton at=808.5,862.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  62 TextBlock at=834,871.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=808.5,862.5,232.5,66 off owner=61 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                63 ActionButton at=1059,862.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  64 TextBlock at=1084.5,871.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=1059,862.5,181.5,66 off owner=63 paints=#707070FF role=text size=26 text="OPTIONS…"
                65 ActionButton at=1258.5,862.5,138,66 by=panel drawn focus heard=VERIFY limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  66 TextBlock at=1284,871.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=1258.5,862.5,138,66 owner=65 paints=#D4D4D4FF role=text size=26 text=VERIFY
                67 ActionButton at=1414.5,862.5,307.5,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  68 TextBlock at=1440,871.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=1414.5,862.5,307.5,66 owner=67 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                69 ActionButton at=96,940.5,120,66 by=panel drawn focus heard=MARK limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  70 TextBlock at=121.5,949.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=96,940.5,120,66 owner=69 paints=#D4D4D4FF role=text size=26 text=MARK
            71 Border at=72,1018.5,1776,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              72 Grid at=78,1024.5,1764,88.5 by=panel limit=0,0,1920,1200 role=none
                73 Grid at=78,1024.5,1764,40.5 by=panel limit=0,0,1920,1200 role=none
                  74 TextBlock at=90,1024.5,1752,40.5 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=831.07,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
                75 StackPanel at=1243.5,1077,598.5,36 by=panel limit=0,0,1920,1200 role=none
                  76 StackPanel at=1243.5,1077,598.5,36 by=panel limit=0,0,1920,1200 role=none
                    77 ProgressBar at=1243.5,1084.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    78 TextBlock at=1555.5,1077,286.5,36 box=191,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 0 B · 410 GB free" layout=190.96,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 0 B · 410 GB free"
            79 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              80 HintBar at=144,1135.5,1632,43.5 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1920,1200 role=group
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
edge 31 up 12
edge 31 down 36
edge 31 left same
edge 31 right 7
edge 36 up 31
edge 36 down 41
edge 36 left same
edge 36 right 7
edge 41 up 36
edge 41 down 46
edge 41 left same
edge 41 right 7
edge 46 up 41
edge 46 down 53
edge 46 left same
edge 46 right 65
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
