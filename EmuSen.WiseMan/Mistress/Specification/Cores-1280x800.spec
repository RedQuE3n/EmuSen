# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Cores-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case Cores
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,738 by=panel heard=Cores limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 DockPanel at=64,40,1152,46 by=panel limit=0,0,1280,774 role=none
              6 Border at=1036,40,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,47,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=64,40,972,46 by=panel limit=0,0,1280,774 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,46 by=panel limit=0,0,1280,774 paints=#FFFFFFFF role=none
                  12 ToggleButton at=64,41,65,44 by=panel drawn focus heard=All limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=All
                    13 TextBlock at=81,47,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=64,41,65,44 owner=12 paints=#EEEEF0FF role=text size=26 text=ALL
                  14 ToggleButton at=137,41,141,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=PlayStation
                    15 TextBlock at=154,47,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=137,41,141,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  16 ToggleButton at=286,41,156,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="PlayStation 2"
                    17 TextBlock at=303,47,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=286,41,156,44 owner=16 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
                  18 ToggleButton at=450,41,77,44 by=panel drawn focus heard=SNES limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=SNES
                    19 TextBlock at=467,47,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=450,41,77,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
                  20 ToggleButton at=535,41,122,44 by=panel drawn focus heard=Installed limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Installed
                    21 TextBlock at=552,47,88,32 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALLED layout=87.88,31.2 limit=535,41,122,44 owner=20 paints=#FFFFFFFF role=text size=26 text=INSTALLED
                  22 ToggleButton at=665,41,122,44 by=panel drawn focus heard=Available limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Available
                    23 TextBlock at=682,47,88,32 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=AVAILABLE layout=87.59,31.2 limit=665,41,122,44 owner=22 paints=#FFFFFFFF role=text size=26 text=AVAILABLE
                  24 ToggleButton at=795,41,108,44 by=panel drawn focus heard=Updates limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Updates
                    25 TextBlock at=812,47,74,32 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATES layout=73.87,31.2 limit=795,41,108,44 owner=24 paints=#FFFFFFFF role=text size=26 text=UPDATES
            26 Grid at=64,98,1152,429 by=panel limit=0,0,1280,774 role=none
              27 Border at=64,98,1152,429 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
                28 Grid at=64,98,1152,429 by=panel limit=0,0,1280,774 role=none
                  29 LunaTable at=64,98,1152,429 by=panel heard=Rows limit=0,0,1280,774 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                    30 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=76,104,38,29 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                    31 GridSplitter at=108,102,6,32 by=clip focus heard="Resize Mark" limit=64,98,1152,43 paints=#D4D4D4FF role=thumb
                    32 Button at=114,102,545,32 by=clip drawn focus heard="Core, not sorted" limit=64,98,1152,43 op paints=#707070FF,#D4D4D4FF role=button
                      33 StackPanel at=114,102,39,32 by=clip limit=114,102,545,32 role=none
                        34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=114,102,39,32 box=39,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=38.3,31.2 owner=32 paints=#D4D4D4FF role=text size=26 text=Core
                    35 GridSplitter at=653,102,6,32 by=clip focus heard="Resize Core" limit=64,98,1152,43 paints=#D4D4D4FF role=thumb
                    36 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,104,545,29 box=545,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                    37 ScrollViewer at=64,141,1152,386 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,141,1152,386
                      38 ListBoxItem at=64,141,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        39 Grid at=74,146,1132,31 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=37 limit=64,141,1152,42 role=none
                          40 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,146,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=38 paints=#D4D4D4FF role=text size=24 text="☐"
                          41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,147,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=69.38,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text=Sapphire
                          42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,147,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=38 paints=#D4D4D4FF role=edit size=24 text="built in"
                      43 ListBoxItem at=64,183,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        44 Grid at=74,188,1132,31 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=37 limit=64,183,1152,42 role=none
                          45 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,188,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=43 paints=#D4D4D4FF role=text size=24 text="☐"
                          46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,189,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=68.5,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text=Diamond
                          47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,189,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=43 paints=#D4D4D4FF role=edit size=24 text="built in"
                      48 ListBoxItem at=64,225,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        49 Grid at=74,230,1132,31 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=37 limit=64,225,1152,42 role=none
                          50 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,230,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=48 paints=#D4D4D4FF role=text size=24 text="☐"
                          51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,231,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=65.62,28.8 owner=48 paints=#D4D4D4FF role=edit size=24 text=VenusRT
                          52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,231,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=52.08,28.8 owner=48 paints=#D4D4D4FF role=edit size=24 text="built in"
                      53 ListBoxItem at=64,267,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        54 Grid at=74,272,1132,31 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=37 limit=64,267,1152,42 role=none
                          55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,272,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=53 paints=#D4D4D4FF role=text size=24 text="☐"
                          56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,273,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=113.69,28.8 owner=53 paints=#D4D4D4FF role=edit size=24 text="Extra PS2 Core"
                          57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,273,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=155.38,28.8 owner=53 paints=#D4D4D4FF role=edit size=24 text="waiting for approval"
                      58 ListBoxItem at=64,309,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        59 Grid at=74,314,1132,31 by=clip heard="Mark: not marked, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=37 limit=64,309,1152,42 role=none
                          60 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,314,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=58 paints=#EEEEF0FF role=text size=24 text="☐"
                          61 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,315,547,29 box=547,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Core in=37 layout=535.97,28.8 owner=58 paints=#EEEEF0FF role=edit size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                          62 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,315,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=68.26,28.8 owner=58 paints=#EEEEF0FF role=edit size=24 text=available
                      63 ListBoxItem at=64,351,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        64 Grid at=74,356,1132,31 by=clip heard="Mark: not marked, Core: Sony - PlayStation (Beetle PSX), State: update ↑" in=37 limit=64,351,1152,42 role=none
                          65 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,356,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=63 paints=#D4D4D4FF role=text size=24 text="☐"
                          66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,357,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=237.58,28.8 owner=63 paints=#D4D4D4FF role=edit size=24 text="Sony - PlayStation (Beetle PSX)"
                          67 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,356,547,31 box=547,31 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=73.97,30.84 owner=63 paints=#D4D4D4FF role=edit size=24 text="update ↑"
                      68 ListBoxItem at=64,393,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        69 Grid at=74,398,1132,31 by=clip heard="Mark: not marked, Core: Nintendo - SNES / SFC (Snes9x - Current), State: installed" in=37 limit=64,393,1152,42 role=none
                          70 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,398,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=68 paints=#D4D4D4FF role=text size=24 text="☐"
                          71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,399,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=314.95,28.8 owner=68 paints=#D4D4D4FF role=edit size=24 text="Nintendo - SNES / SFC (Snes9x - Current)"
                          72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,399,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=65.16,28.8 owner=68 paints=#D4D4D4FF role=edit size=24 text=installed
                      73 ListBoxItem at=64,435,1152,42 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Only GL Core, State: not supported" in=37 limit=64,141,1152,386 op paints=#333337FF,#D4D4D4FF role=listitem
                        74 Grid at=74,440,1132,31 by=clip heard="Mark: not marked, Core: Only GL Core, State: not supported" in=37 limit=64,435,1152,42 role=none
                          75 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=74,440,38,31 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=37 layout=14.4,30.84 owner=73 paints=#D4D4D4FF role=text size=24 text="☐"
                          76 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,441,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=37 layout=97.63,28.8 owner=73 paints=#D4D4D4FF role=edit size=24 text="Only GL Core"
                          77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=659,441,547,29 box=547,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=37 layout=109.94,28.8 owner=73 paints=#D4D4D4FF role=edit size=24 text="not supported"
            78 Border at=64,535,1152,96 by=panel limit=0,0,1280,774 name=AdaptiveActions role=none
              79 WrapPanel at=64,535,1152,96 by=panel limit=0,0,1280,774 role=none
                80 ActionButton at=64,535,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  81 TextBlock at=81,541,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,535,101,44 owner=80 paints=#D4D4D4FF role=text size=26 text=INSTALL
                82 ActionButton at=177,535,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  83 TextBlock at=194,541,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,535,98,44 owner=82 paints=#D4D4D4FF role=text size=26 text=UPDATE
                84 ActionButton at=287,535,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  85 TextBlock at=304,541,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,535,101,44 owner=84 paints=#D4D4D4FF role=text size=26 text=REMOVE
                86 ActionButton at=400,535,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  87 TextBlock at=417,541,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,535,127,44 off owner=86 paints=#707070FF role=text size=26 text="APPROVE…"
                88 ActionButton at=539,535,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  89 TextBlock at=556,541,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,535,155,44 off owner=88 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                90 ActionButton at=706,535,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  91 TextBlock at=723,541,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,535,121,44 off owner=90 paints=#707070FF role=text size=26 text="OPTIONS…"
                92 ActionButton at=839,535,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  93 TextBlock at=856,541,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,535,92,44 owner=92 paints=#D4D4D4FF role=text size=26 text=VERIFY
                94 ActionButton at=943,535,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  95 TextBlock at=960,541,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,535,205,44 owner=94 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                96 ActionButton at=64,587,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  97 TextBlock at=81,593,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,587,80,44 owner=96 paints=#D4D4D4FF role=text size=26 text=MARK
            98 Border at=48,639,1184,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
              99 Grid at=52,643,1176,81 by=panel limit=0,0,1280,774 role=none
                100 Grid at=52,643,1176,29 by=panel limit=0,0,1280,774 role=none
                  101 TextBlock at=60,643,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                102 StackPanel at=709,680,519,44 by=panel limit=0,0,1280,774 role=none
                  103 Button at=709,680,102,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    104 TextBlock at=726,686,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=709,680,102,44 off owner=103 paints=#707070FF role=text size=26 text="APPLY…"
                  105 StackPanel at=823,680,405,44 by=panel limit=0,0,1280,774 role=none
                    106 ProgressBar at=823,695,200,14 by=panel drawn heard=Space limit=0,0,1280,774 paints=#333337FF,#4C9AE8FF role=progressbar
                    107 TextBlock at=1031,688,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            108 HintBar at=64,736,1152,22 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,774 role=group
focus 31
edge 7 up same
edge 7 down 38
edge 7 left 24
edge 7 right same
edge 12 up same
edge 12 down 31
edge 12 left same
edge 12 right 14
edge 14 up same
edge 14 down 32
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
edge 22 down 38
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 38
edge 24 left 22
edge 24 right 7
edge 31 up 12
edge 31 down 38
edge 31 left same
edge 31 right 32
edge 32 up 14
edge 32 down 38
edge 32 left 31
edge 32 right 22
edge 38 up 31
edge 38 down 43
edge 38 left same
edge 38 right same
edge 43 up 38
edge 43 down 48
edge 43 left same
edge 43 right same
edge 48 up 43
edge 48 down 53
edge 48 left same
edge 48 right same
edge 53 up 48
edge 53 down 58
edge 53 left same
edge 53 right same
edge 58 up 53
edge 58 down 63
edge 58 left same
edge 58 right same
edge 63 up 58
edge 63 down 68
edge 63 left same
edge 63 right same
edge 68 up 63
edge 68 down 73
edge 68 left same
edge 68 right same
edge 73 up 68
edge 73 down 80
edge 73 left same
edge 73 right same
edge 80 up 73
edge 80 down 96
edge 80 left same
edge 80 right 82
edge 82 up 73
edge 82 down 96
edge 82 left 80
edge 82 right 84
edge 84 up 73
edge 84 down 96
edge 84 left 82
edge 84 right 92
edge 92 up 73
edge 92 down 96
edge 92 left 84
edge 92 right 94
edge 94 up 73
edge 94 down 96
edge 94 left 92
edge 94 right same
edge 96 up 80
edge 96 down same
edge 96 left same
edge 96 right 82
press pageup focus="GridSplitter  (Resize Mark)" top=CoresWindow
press pagedown focus="GridSplitter  (Resize Mark)" top=CoresWindow
press first focus="GridSplitter  (Resize Mark)" top=CoresWindow
press last focus="GridSplitter  (Resize Mark)" top=CoresWindow
press back top=none
