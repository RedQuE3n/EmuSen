# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1146 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=868.5,69,955.5,39 box=637,26 by=panel caps=13.999 face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.08,25.7 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=96,60,754.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,754.5,58.5 box=503,39 by=panel caps=22.3984 face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel focus heard=Search limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel focus heard=All limit=0,0,1920,1200 paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,240,66 by=panel focus heard="PlayStation 2" limit=0,0,1920,1200 paints=#D4D4D4FF role=button text="PlayStation 2"
                    20 TextBlock at=231,148.5,192,36 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=207,136.5,240,66 owner=19 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  21 ToggleButton at=459,136.5,219,66 by=panel focus heard=PlayStation limit=0,0,1920,1200 paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=483,148.5,171,36 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=459,136.5,219,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=690,136.5,118.5,66 by=panel focus heard=SNES limit=0,0,1920,1200 paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=714,148.5,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=690,136.5,118.5,66 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,220.5,1036.5,798 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=96,237,1036.5,781.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,781.5
                        31 ListBoxItem at=96,237,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          32 Border at=132,252,964.5,52.5 by=clip in=30 limit=96,237,1036.5,81 paints=#707070FF role=none
                            33 Grid at=132,252,964.5,51 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,237,1036.5,81 role=none
                              34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,253.5,163.5,48 in=30 owner=31 paints=#4C9AE8FF role=text
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,253.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=73.45,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,253.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,252,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        38 ListBoxItem at=96,318,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          39 Border at=132,333,964.5,52.5 by=clip in=30 limit=96,318,1036.5,81 paints=#707070FF role=none
                            40 Grid at=132,333,964.5,51 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,318,1036.5,81 role=none
                              41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,334.5,163.5,48 in=30 owner=38 paints=#4C9AE8FF role=text
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,334.5,204,48 box=136,32 caps=18.1987 ellipsis face=Barlow heard=Core in=30 layout=132.18,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                              43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,334.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,333,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=38 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        45 ListBoxItem at=96,399,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          46 Border at=132,414,964.5,49.5 by=clip in=30 limit=96,399,1036.5,81 paints=#707070FF role=none
                            47 Grid at=132,414,964.5,48 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,399,1036.5,81 role=none
                              48 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,414,163.5,48 in=30 owner=45 paints=#4C9AE8FF role=text
                              49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,414,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=99.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,414,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,414,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=75.79,31.2 owner=45 paints=#8D8D91FF role=edit size=26 text="Built in"
                        52 ListBoxItem at=96,480,1036.5,81 by=fixed-scroll focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,237,1036.5,781.5 paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          53 Border at=132,495,964.5,52.5 by=clip in=30 limit=96,480,1036.5,81 paints=#707070FF role=none
                            54 Grid at=132,495,964.5,51 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,480,1036.5,81 role=none
                              55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,495,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ install" in=30 layout=96.85,33.41 owner=52 paints=#4C9AE8FF role=text size=26 text="✓ install"
                              56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,496.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=72.54,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                              57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,496.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,496.5,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=98.31,31.2 owner=52 paints=#8D8D91FF role=edit size=26 text=Available
                        59 ListBoxItem at=96,561,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          60 Border at=132,576,964.5,52.5 by=clip in=30 limit=96,561,1036.5,81 paints=#707070FF role=none
                            61 Grid at=132,576,964.5,51 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,561,1036.5,81 role=none
                              62 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,577.5,163.5,48 in=30 owner=59 paints=#4C9AE8FF role=text
                              63 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,577.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=98.67,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,577.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,576,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=59 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        66 ListBoxItem at=96,642,1036.5,81 by=fixed-scroll focus heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          67 Border at=132,657,964.5,52.5 by=clip in=30 limit=96,642,1036.5,81 paints=#707070FF role=none
                            68 Grid at=132,657,964.5,51 by=clip heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,642,1036.5,81 role=none
                              69 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,657,163.5,51 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ update" in=30 layout=108.24,33.41 owner=66 paints=#4C9AE8FF role=text size=26 text="✓ update"
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,658.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=118.22,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,658.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,657,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=110.94,33.41 owner=66 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        73 ListBoxItem at=96,723,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          74 Border at=132,738,964.5,49.5 by=clip in=30 limit=96,723,1036.5,81 paints=#707070FF role=none
                            75 Grid at=132,738,964.5,48 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,723,1036.5,81 role=none
                              76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,738,163.5,48 in=30 owner=73 paints=#4C9AE8FF role=text
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,738,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=81.77,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,738,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              79 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,738,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=157.48,31.2 owner=73 paints=#8D8D91FF role=edit size=26 text="Not supported"
                        80 ListBoxItem at=96,804,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          81 Border at=132,819,964.5,52.5 by=clip in=30 limit=96,804,1036.5,81 paints=#707070FF role=none
                            82 Grid at=132,819,964.5,51 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,804,1036.5,81 role=none
                              83 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,820.5,163.5,48 in=30 owner=80 paints=#4C9AE8FF role=text
                              84 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,820.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=95.08,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              85 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,820.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=SNES
                              86 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,819,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=80 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        87 ListBoxItem at=96,885,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,237,1036.5,781.5 paints=#D4D4D4FF role=listitem
                          88 Border at=132,900,964.5,52.5 by=clip in=30 limit=96,885,1036.5,81 paints=#707070FF role=none
                            89 Grid at=132,900,964.5,51 by=clip heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,885,1036.5,81 role=none
                              90 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,901.5,163.5,48 in=30 owner=87 paints=#4C9AE8FF role=text
                              91 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=331.5,901.5,204,48 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=79.27,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                              92 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,901.5,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=SNES
                              93 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,900,237,51 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=123.97,33.41 owner=87 paints=#35A064FF role=edit size=26 text="✓ Installed"
                94 Border at=96,1018.5,1036.5,100.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  95 Grid at=102,1024.5,1024.5,88.5 by=panel limit=0,0,1920,1200 role=none
                    96 Grid at=102,1024.5,1024.5,40.5 by=panel limit=0,0,1920,1200 role=none
                      97 TextBlock at=114,1024.5,1012.5,40.5 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 marked: install 1, update 1, about 8 MB to download" layout=479.95,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="2 marked: install 1, update 1, about 8 MB to download"
                    98 StackPanel at=498,1077,628.5,36 by=panel limit=0,0,1920,1200 role=none
                      99 StackPanel at=498,1077,628.5,36 by=panel limit=0,0,1920,1200 role=none
                        100 ProgressBar at=498,1084.5,300,21 by=panel heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        101 TextBlock at=810,1077,316.5,36 box=211,24 by=panel caps=13.999 face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              102 Border at=1156.5,220.5,667.5,898.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                103 Border at=1192.5,256.5,595.5,826.5 by=panel limit=0,0,1920,1200 role=none
                  104 ScrollViewer at=1204.5,268.5,571.5,802.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1204.5,268.5,571.5,802.5
                    105 StackPanel at=1204.5,268.5,571.5,802.5 by=fixed-scroll in=104 limit=1204.5,268.5,571.5,802.5 name=CoresDetail role=none
                      106 TextBlock at=1204.5,268.5,571.5,58.5 box=381,39 by=fixed-scroll caps=22.3984 face="Barlow Condensed" heard=LRPS2 in=104 layout=64.35,38.4 limit=1204.5,268.5,571.5,802.5 paints=#EEEEF0FF role=text size=32 text=LRPS2
                      107 TextBlock at=1204.5,339,571.5,159 box=381,106 by=fixed-scroll caps=15.3989 face=Barlow heard="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”" in=104 layout=361.53,105.6 limit=1204.5,268.5,571.5,802.5 paints=#8D8D91FF role=text size=22 text="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”"
                      108 TextBlock at=1204.5,510,571.5,120 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own." in=104 layout=355.48,79.2 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own."
                      109 TextBlock at=1204.5,642,571.5,79.5 box=381,53 by=fixed-scroll caps=15.3989 face=Barlow heard="By PCSX2 Team, libretro, under the GPLv3. Build 2.3.1 2026-10-31." in=104 layout=319.66,52.8 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="By PCSX2 Team, libretro, under the GPLv3. Build 2.3.1 2026-10-31."
                      110 TextBlock at=1204.5,733.5,571.5,79.5 box=381,53 by=fixed-scroll caps=15.3989 face=Barlow heard="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads." in=104 layout=364.5,52.8 limit=1204.5,268.5,571.5,802.5 paints=#D4D4D4FF role=text size=22 text="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads."
                      111 TextBlock at=1204.5,825,571.5,120 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=104 layout=358.86,79.2 limit=1204.5,268.5,571.5,802.5 paints=#8D8D91FF role=text size=22 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      112 TextBlock at=1204.5,957,571.5,40.5 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="Not tested by EmuSen." in=104 layout=210.43,26.4 limit=1204.5,268.5,571.5,802.5 paints=#8D8D91FF role=text size=22 text="Not tested by EmuSen."
            113 Border at=72,1119,1776,69 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              114 HintBar at=144,1135.5,1632,43.5 by=panel heard="Choose, No" limit=0,0,1920,1200 role=group
          115 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 name=AdaptiveCover paints=#000000B8 role=none
            116 Border at=420,315,1080,600 by=panel limit=0,0,1920,1200 paints=#26262BFF,#707070FF role=none
              117 DockPanel at=469.5,364.5,981,501 by=panel limit=0,0,1920,1200 role=none
                118 TextBlock at=469.5,364.5,981,58.5 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply 2 changes?" layout=181.06,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply 2 changes?"
                119 StackPanel at=469.5,697.5,981,168 by=panel limit=0,0,1920,1200 role=none
                  120 Button at=469.5,697.5,981,81 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    121 TextBlock at=511.5,714,46.5,48 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=469.5,697.5,981,81 owner=120 paints=#EEEEF0FF role=text size=26 text=No
                  122 Button at=469.5,784.5,981,81 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#D4D4D4FF role=button
                    123 TextBlock at=511.5,801,60,48 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=469.5,784.5,981,81 owner=122 paints=#D4D4D4FF role=text size=26 text=Yes
                124 ScrollViewer at=469.5,441,981,238.5 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=469.5,441,981,238.5
                  125 TextBlock at=469.5,441,981,238.5 box=654,159 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Download and install 1 core from buildbot.libretro.com: LRPS2.\n\nUpdate Beetle PSX; the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=124 layout=626.89,158.4 limit=469.5,441,981,238.5 paints=#D4D4D4FF role=text size=22 text="Download and install 1 core from buildbot.libretro.com: LRPS2.\n\nUpdate Beetle PSX; the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
focus 120
edge 120 up same
edge 120 down 122
edge 120 left same
edge 120 right same
edge 122 up 120
edge 122 down same
edge 122 left same
edge 122 right same
press pageup focus="Button  (No)" top=CoresWindow
press pagedown focus="Button  (No)" top=CoresWindow
press first focus="Button  (No)" top=CoresWindow
press last focus="Button  (No)" top=CoresWindow
press back focus="ListBoxItem  (Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available)" top=CoresWindow
