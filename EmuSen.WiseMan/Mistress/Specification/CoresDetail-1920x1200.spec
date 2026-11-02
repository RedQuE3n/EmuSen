# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case CoresDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1146 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=868.5,69,955.5,39 box=637,26 by=panel caps=13.999 drawn face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.08,25.7 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=96,60,754.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,754.5,58.5 box=503,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,240,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    20 TextBlock at=231,148.5,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=207,136.5,240,66 owner=19 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  21 ToggleButton at=459,136.5,219,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=483,148.5,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=459,136.5,219,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=690,136.5,118.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=714,148.5,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=690,136.5,118.5,66 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,220.5,1036.5,798 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=96,237,1036.5,781.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,781.5
                        31 ListBoxItem at=96,237,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          32 Border at=132,252,964.5,52.5 by=clip in=30 limit=96,237,1036.5,81 paints=#707070FF role=none
                            33 Grid at=132,252,964.5,51 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,237,1036.5,81 paints=#4C9AE8FF role=none
                              34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,253.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=73.45,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,253.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,252,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        37 ListBoxItem at=96,318,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,237,1036.5,781.5 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          38 Border at=132,333,964.5,52.5 by=clip in=30 limit=96,318,1036.5,81 paints=#707070FF role=none
                            39 Grid at=132,333,964.5,51 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,318,1036.5,81 paints=#4C9AE8FF role=none
                              40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,334.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=37 paints=#EEEEF0FF role=edit size=26 text="Extra PS2 Core"
                              41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,334.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=37 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,333,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=37 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        43 ListBoxItem at=96,399,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          44 Border at=132,414,964.5,49.5 by=clip in=30 limit=96,399,1036.5,81 paints=#707070FF role=none
                            45 Grid at=132,414,964.5,48 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,399,1036.5,81 paints=#4C9AE8FF role=none
                              46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,414,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,414,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,414,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=43 paints=#8D8D91FF role=edit size=26 text="Built in"
                        49 ListBoxItem at=96,480,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          50 Border at=132,495,964.5,49.5 by=clip in=30 limit=96,480,1036.5,81 paints=#707070FF role=none
                            51 Grid at=132,495,964.5,48 by=clip heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,480,1036.5,81 paints=#4C9AE8FF role=none
                              52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,495,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=72.54,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text=LRPS2
                              53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,495,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=49 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,495,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=98.31,31.2 owner=49 paints=#8D8D91FF role=edit size=26 text=Available
                        55 ListBoxItem at=96,561,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          56 Border at=132,576,964.5,52.5 by=clip in=30 limit=96,561,1036.5,81 paints=#707070FF role=none
                            57 Grid at=132,576,964.5,51 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,561,1036.5,81 paints=#4C9AE8FF role=none
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,577.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=55 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,577.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=55 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,576,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=55 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        61 ListBoxItem at=96,642,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          62 Border at=132,657,964.5,52.5 by=clip in=30 limit=96,642,1036.5,81 paints=#707070FF role=none
                            63 Grid at=132,657,964.5,51 by=clip heard="Mark: not marked, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,642,1036.5,81 paints=#4C9AE8FF role=none
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,658.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=118.22,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,658.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,657,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=110.94,33.41 owner=61 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        67 ListBoxItem at=96,723,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          68 Border at=132,738,964.5,49.5 by=clip in=30 limit=96,723,1036.5,81 paints=#707070FF role=none
                            69 Grid at=132,738,964.5,48 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,723,1036.5,81 paints=#4C9AE8FF role=none
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,738,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=81.77,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,738,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,738,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=157.48,31.2 owner=67 paints=#8D8D91FF role=edit size=26 text="Not supported"
                        73 ListBoxItem at=96,804,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          74 Border at=132,819,964.5,52.5 by=clip in=30 limit=96,804,1036.5,81 paints=#707070FF role=none
                            75 Grid at=132,819,964.5,51 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,804,1036.5,81 paints=#4C9AE8FF role=none
                              76 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,820.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,820.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=60.79,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=SNES
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,819,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=73 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        79 ListBoxItem at=96,885,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          80 Border at=132,900,964.5,52.5 by=clip in=30 limit=96,885,1036.5,81 paints=#707070FF role=none
                            81 Grid at=132,900,964.5,51 by=clip heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,885,1036.5,81 paints=#4C9AE8FF role=none
                              82 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=168,901.5,367.5,48 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=79.27,31.2 owner=79 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                              83 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,901.5,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=60.79,31.2 owner=79 paints=#D4D4D4FF role=edit size=26 text=SNES
                              84 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,900,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=123.97,33.41 owner=79 paints=#35A064FF role=edit size=26 text="✓ Installed"
                85 Border at=96,1018.5,1036.5,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  86 Grid at=102,1024.5,1024.5,88.5 by=panel limit=0,0,1920,1200 role=none
                    87 Grid at=102,1024.5,1024.5,40.5 by=panel limit=0,0,1920,1200 role=none
                      88 TextBlock at=114,1024.5,1012.5,40.5 box=675,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    89 StackPanel at=498,1077,628.5,36 by=panel limit=0,0,1920,1200 role=none
                      90 StackPanel at=498,1077,628.5,36 by=panel limit=0,0,1920,1200 role=none
                        91 ProgressBar at=498,1084.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        92 TextBlock at=810,1077,316.5,36 box=211,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              93 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                94 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  95 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    96 StackPanel at=1204.5,268.5,571.5,802.5 by=fixed-scroll in=95 limit=1204.5,268.5,571.5,802.5 name=CoresDetail role=none
                      97 TextBlock at=1204.5,268.5,571.5,58.5 box=381,39 by=fixed-scroll caps=22.3984 drawn face="Barlow Condensed" heard="Extra PS2 Core" in=95 layout=151.58,38.4 limit=1204.5,268.5,571.5,802.5 paints=#EEEEF0FF role=text size=32 text="Extra PS2 Core"
                      98 TextBlock at=1204.5,339,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Added by you · PlayStation 2" in=95 layout=259.03,26.4 limit=1204.5,268.5,571.5,802.5 paints=#8D8D91FF role=text size=22 text="Added by you · PlayStation 2"
                      99 TextBlock at=1204.5,391.5,571.5,79.5 box=381,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Runs PlayStation 2 games, from a file of your own." in=95 layout=363.86,52.8 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games, from a file of your own."
                      100 TextBlock at=1204.5,483,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="By Someone Else, under the MIT." in=95 layout=298.98,26.4 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="By Someone Else, under the MIT."
                      101 TextBlock at=1204.5,535.5,571.5,82.5 box=381,55 by=fixed-scroll caps=16.1724 drawn face="Adwaita Mono" heard="⚠ Waiting for your approval. Nothing downloaded runs until you approve it." in=95 layout=346.9,54.67 limit=1204.5,268.5,571.5,802.5 paints=#DAA520FF role=text size=22 text="⚠ Waiting for your approval. Nothing downloaded runs until you approve it."
            102 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              103 HintBar at=144,1135.5,1632,43.5 by=panel heard="Approve…, Back, Console, List, Apply…, More" limit=0,0,1920,1200 role=group
focus 37
edge 12 up same
edge 12 down 31
edge 12 left 23
edge 12 right same
edge 17 up same
edge 17 down 31
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 31
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 31
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 31
edge 23 left 21
edge 23 right 12
edge 31 up 17
edge 31 down 37
edge 31 left same
edge 31 right 12
edge 37 up 31
edge 37 down 43
edge 37 left same
edge 37 right 12
edge 43 up 37
edge 43 down 49
edge 43 left same
edge 43 right 12
edge 49 up 43
edge 49 down 55
edge 49 left same
edge 49 right 12
edge 55 up 49
edge 55 down 61
edge 55 left same
edge 55 right 12
edge 61 up 55
edge 61 down 67
edge 61 left same
edge 61 right 12
edge 67 up 61
edge 67 down 73
edge 67 left same
edge 67 right 12
edge 73 up 67
edge 73 down 79
edge 73 left same
edge 73 right 12
edge 79 up 73
edge 79 down same
edge 79 left same
edge 79 right 12
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default)" top=CoresWindow
