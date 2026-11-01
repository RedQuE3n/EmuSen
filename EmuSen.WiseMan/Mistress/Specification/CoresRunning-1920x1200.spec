# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case CoresRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1107 by=panel heard=Cores limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 DockPanel at=96,60,1728,69 by=panel limit=0,0,1920,1161 role=none
              6 Border at=1554,60,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                7 TextBox at=1554,60,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1570.5,70.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1570.5,70.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=96,60,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                11 WholeRow at=96,60,1458,69 by=panel limit=0,0,1920,1161 paints=#FFFFFFFF role=none
                  12 ToggleButton at=96,61.5,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    13 TextBlock at=121.5,70.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,61.5,97.5,66 owner=12 paints=#EEEEF0FF role=text size=26 text=ALL
                  14 ToggleButton at=205.5,61.5,211.5,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=PlayStation
                    15 TextBlock at=231,70.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=205.5,61.5,211.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  16 ToggleButton at=429,61.5,234,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="PlayStation 2"
                    17 TextBlock at=454.5,70.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=429,61.5,234,66 owner=16 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
                  18 ToggleButton at=675,61.5,115.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=SNES
                    19 TextBlock at=700.5,70.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=675,61.5,115.5,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
                  20 ToggleButton at=802.5,61.5,183,66 by=panel drawn focus heard=Installed limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Installed
                    21 TextBlock at=828,70.5,132,48 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALLED layout=87.88,31.2 limit=802.5,61.5,183,66 owner=20 paints=#FFFFFFFF role=text size=26 text=INSTALLED
                  22 ToggleButton at=997.5,61.5,183,66 by=panel drawn focus heard=Available limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Available
                    23 TextBlock at=1023,70.5,132,48 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=AVAILABLE layout=87.59,31.2 limit=997.5,61.5,183,66 owner=22 paints=#FFFFFFFF role=text size=26 text=AVAILABLE
                  24 ToggleButton at=1192.5,61.5,162,66 by=panel drawn focus heard=Updates limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Updates
                    25 TextBlock at=1218,70.5,111,48 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATES layout=73.87,31.2 limit=1192.5,61.5,162,66 owner=24 paints=#FFFFFFFF role=text size=26 text=UPDATES
            26 Grid at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 role=none
              27 Border at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                28 Grid at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 role=none
                  29 LunaTable at=96,147,1728,643.5 by=panel heard=Rows limit=0,0,1920,1161 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=114,156,126,43.5 box=84,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                    31 GridSplitter at=231,153,9,48 by=clip focus heard="Resize Mark" limit=96,147,1728,64.5 paints=#D4D4D4FF role=thumb
                    32 Button at=240,153,783,48 by=clip drawn focus heard="Core, not sorted" limit=96,147,1728,64.5 op paints=#707070FF,#D4D4D4FF role=button
                      33 StackPanel at=240,153,58.5,48 by=clip limit=240,153,783,48 role=none
                        34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=240,153,58.5,48 box=39,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=38.3,31.2 owner=32 paints=#D4D4D4FF role=text size=26 text=Core
                    35 GridSplitter at=1014,153,9,48 by=clip focus heard="Resize Core" limit=96,147,1728,64.5 paints=#D4D4D4FF role=thumb
                    36 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,156,783,43.5 box=522,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                    37 ScrollViewer at=96,211.5,1728,579 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,211.5,1728,579
                      38 ListBoxItem at=96,211.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        39 Grid at=111,219,1698,46.5 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=37 limit=96,211.5,1728,63 role=none
                          40 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,219,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=38 paints=#D4D4D4FF role=text size=24 text="☐"
                          41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,220.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=69.38,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text=Sapphire
                          42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,220.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text="built in"
                      43 ListBoxItem at=96,274.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        44 Grid at=111,282,1698,46.5 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=37 limit=96,274.5,1728,63 role=none
                          45 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,282,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=43 paints=#D4D4D4FF role=text size=24 text="☐"
                          46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,283.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=68.5,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text=Diamond
                          47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,283.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text="built in"
                      48 ListBoxItem at=96,337.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        49 Grid at=111,345,1698,46.5 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=37 limit=96,337.5,1728,63 role=none
                          50 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,345,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=48 paints=#D4D4D4FF role=text size=24 text="☐"
                          51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,346.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=65.62,28.8 owner=48 paints=#D4D4D4FF role=edit size=24 text=VenusRT
                          52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,346.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=48 paints=#D4D4D4FF role=edit size=24 text="built in"
                      53 ListBoxItem at=96,400.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        54 Grid at=111,408,1698,46.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=37 limit=96,400.5,1728,63 role=none
                          55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,408,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=53 paints=#D4D4D4FF role=text size=24 text="☐"
                          56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,409.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=113.69,28.8 owner=53 paints=#D4D4D4FF role=edit size=24 text="Extra PS2 Core"
                          57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,409.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=155.38,28.8 owner=53 paints=#D4D4D4FF role=edit size=24 text="waiting for approval"
                      58 ListBoxItem at=96,463.5,1728,63 by=fixed-scroll drawn focus heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        59 Grid at=111,471,1698,46.5 by=clip heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=37 limit=96,463.5,1728,63 role=none
                          60 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,471,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ install" in=37 layout=74.52,30.84 owner=58 paints=#EEEEF0FF role=text size=24 text="☑ install"
                          61 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,472.5,786,43.5 box=524,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Core in=37 layout=516.67,28.8 owner=58 paints=#EEEEF0FF role=edit size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                          62 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,472.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=68.26,28.8 owner=58 paints=#EEEEF0FF role=edit size=24 text=available
                      63 ListBoxItem at=96,526.5,1728,63 by=fixed-scroll drawn focus heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        64 Grid at=111,534,1698,46.5 by=clip heard="Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=37 limit=96,526.5,1728,63 role=none
                          65 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,534,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ update" in=37 layout=83.57,30.84 owner=63 paints=#D4D4D4FF role=text size=24 text="☑ update"
                          66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,535.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=237.58,28.8 owner=63 paints=#D4D4D4FF role=edit size=24 text="Sony - PlayStation (Beetle PSX)"
                          67 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,534,786,46.5 box=524,31 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=73.97,30.84 owner=63 paints=#D4D4D4FF role=edit size=24 text="update ↑"
                      68 ListBoxItem at=96,589.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        69 Grid at=111,597,1698,46.5 by=clip heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=37 limit=96,589.5,1728,63 role=none
                          70 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,597,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=68 paints=#D4D4D4FF role=text size=24 text="☐"
                          71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,598.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=314.95,28.8 owner=68 paints=#D4D4D4FF role=edit size=24 text="Nintendo - SNES / SFC (Snes9x - Current)"
                          72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,598.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=65.16,28.8 owner=68 paints=#D4D4D4FF role=edit size=24 text=installed
                      73 ListBoxItem at=96,652.5,1728,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Only GL Core, State: not supported" in=37 limit=96,211.5,1728,579 op paints=#333337FF,#D4D4D4FF role=listitem
                        74 Grid at=111,660,1698,46.5 by=clip heard="Mark: not marked, Core: Only GL Core, State: not supported" in=37 limit=96,652.5,1728,63 role=none
                          75 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=111,660,126,46.5 box=84,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=73 paints=#D4D4D4FF role=text size=24 text="☐"
                          76 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=237,661.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=97.63,28.8 owner=73 paints=#D4D4D4FF role=edit size=24 text="Only GL Core"
                          77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1023,661.5,786,43.5 box=524,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=109.94,28.8 owner=73 paints=#D4D4D4FF role=edit size=24 text="not supported"
            78 Border at=96,802.5,1728,144 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              79 WrapPanel at=96,802.5,1728,144 by=panel limit=0,0,1920,1161 role=none
                80 ActionButton at=96,802.5,151.5,66 by=panel drawn heard=INSTALL limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Install
                  81 TextBlock at=121.5,811.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=96,802.5,151.5,66 off owner=80 paints=#707070FF role=text size=26 text=INSTALL
                82 ActionButton at=265.5,802.5,147,66 by=panel drawn heard=UPDATE limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Update
                  83 TextBlock at=291,811.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=265.5,802.5,147,66 off owner=82 paints=#707070FF role=text size=26 text=UPDATE
                84 ActionButton at=430.5,802.5,151.5,66 by=panel drawn heard=REMOVE limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Remove
                  85 TextBlock at=456,811.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=430.5,802.5,151.5,66 off owner=84 paints=#707070FF role=text size=26 text=REMOVE
                86 ActionButton at=600,802.5,190.5,66 by=panel drawn heard="APPROVE…" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  87 TextBlock at=625.5,811.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=600,802.5,190.5,66 off owner=86 paints=#707070FF role=text size=26 text="APPROVE…"
                88 ActionButton at=808.5,802.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  89 TextBlock at=834,811.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=808.5,802.5,232.5,66 off owner=88 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                90 ActionButton at=1059,802.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  91 TextBlock at=1084.5,811.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=1059,802.5,181.5,66 off owner=90 paints=#707070FF role=text size=26 text="OPTIONS…"
                92 ActionButton at=1258.5,802.5,138,66 by=panel drawn heard=VERIFY limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Verify
                  93 TextBlock at=1284,811.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=1258.5,802.5,138,66 off owner=92 paints=#707070FF role=text size=26 text=VERIFY
                94 ActionButton at=1414.5,802.5,307.5,66 by=panel drawn heard="CHECK FOR UPDATES" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  95 TextBlock at=1440,811.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=1414.5,802.5,307.5,66 off owner=94 paints=#707070FF role=text size=26 text="CHECK FOR UPDATES"
                96 ActionButton at=96,880.5,120,66 by=panel drawn heard=MARK limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text=Mark
                  97 TextBlock at=121.5,889.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=96,880.5,120,66 off owner=96 paints=#707070FF role=text size=26 text=MARK
            98 Border at=72,958.5,1776,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
              99 Grid at=78,964.5,1764,121.5 by=panel limit=0,0,1920,1161 role=none
                100 Grid at=78,964.5,1764,43.5 by=panel limit=0,0,1920,1161 role=none
                  101 ProgressBar at=78,982.5,180,6 by=panel drawn heard=Progress limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                  102 TextBlock at=270,964.5,1572,43.5 box=1048,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Installing pcsx2: 18 MB of 41 MB" layout=239.74,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="Installing pcsx2: 18 MB of 41 MB"
                103 StackPanel at=1063.5,1020,778.5,66 by=panel limit=0,0,1920,1161 role=none
                  104 Button at=1063.5,1020,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    105 TextBlock at=1089,1029,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1063.5,1020,153,66 off owner=104 paints=#707070FF role=text size=26 text="APPLY…"
                  106 StackPanel at=1234.5,1020,607.5,66 by=panel limit=0,0,1920,1161 role=none
                    107 ProgressBar at=1234.5,1042.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                    108 TextBlock at=1546.5,1032,295.5,43.5 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            109 HintBar at=96,1104,1728,33 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1920,1161 role=group
focus 7
edge 7 up same
edge 7 down 38
edge 7 left 24
edge 7 right same
edge 12 up same
edge 12 down 38
edge 12 left same
edge 12 right 14
edge 14 up same
edge 14 down 31
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 32
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 32
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 32
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 35
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 38
edge 24 left 22
edge 24 right 7
edge 31 up 14
edge 31 down 38
edge 31 left 12
edge 31 right 32
edge 32 up 14
edge 32 down 38
edge 32 left 31
edge 32 right 24
edge 35 up 22
edge 35 down 38
edge 35 left 31
edge 35 right 24
edge 38 up 31
edge 38 down 43
edge 38 left same
edge 38 right 7
edge 43 up 38
edge 43 down 48
edge 43 left same
edge 43 right 7
edge 48 up 43
edge 48 down 53
edge 48 left same
edge 48 right 7
edge 53 up 48
edge 53 down 58
edge 53 left same
edge 53 right 7
edge 58 up 53
edge 58 down 63
edge 58 left same
edge 58 right 7
edge 63 up 58
edge 63 down 68
edge 63 left same
edge 63 right 7
edge 68 up 63
edge 68 down 73
edge 68 left same
edge 68 right 7
edge 73 up 68
edge 73 down same
edge 73 left same
edge 73 right 7
press pageup focus="TextBox Search cores (Search)" top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" top=CoresWindow
