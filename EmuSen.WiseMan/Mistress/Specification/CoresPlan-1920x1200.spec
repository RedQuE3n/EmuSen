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
                7 TextBlock at=868.5,70.5,955.5,37.5 box=637,25 by=panel caps=13.999 face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.52,24.82 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=96,60,754.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,754.5,58.5 box=503,39 by=panel caps=22.3984 face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=96,136.5,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,66 by=panel focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,151.5,237,36 box=158,24 caps=13.999 face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,136.5,99,66 by=panel focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,148.5,51,36 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=96,136.5,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,136.5,219,66 by=panel focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    20 TextBlock at=231,148.5,171,36 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=207,136.5,219,66 owner=19 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  21 ToggleButton at=438,136.5,240,66 by=panel focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    22 TextBlock at=462,148.5,192,36 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=438,136.5,240,66 owner=21 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  23 ToggleButton at=690,136.5,118.5,66 by=panel focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=714,148.5,70.5,36 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=690,136.5,118.5,66 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=96,220.5,1728,898.5 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,220.5,1036.5,898.5 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,220.5,1036.5,798 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,220.5,1036.5,798 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=96,237,1036.5,781.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,237,1036.5,781.5
                        31 ListBoxItem at=96,237,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          32 Border at=132,252,964.5,49.5 by=clip in=30 limit=96,237,1036.5,81 paints=#707070FF role=none
                            33 Grid at=132,252,964.5,48 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,237,1036.5,81 role=none
                              34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,252,162,48 in=30 owner=31 paints=#4C9AE8FF role=text
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,252,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,252,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,252,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=75.79,31.2 owner=31 paints=#8D8D91FF role=edit size=26 text="Built in"
                        38 ListBoxItem at=96,318,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          39 Border at=132,333,964.5,51 by=clip in=30 limit=96,318,1036.5,81 paints=#707070FF role=none
                            40 Grid at=132,333,964.5,49.5 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,318,1036.5,81 role=none
                              41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,333,162,48 in=30 owner=38 paints=#4C9AE8FF role=text
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,333,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=98.67,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,333,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,333,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=38 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        45 ListBoxItem at=96,399,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          46 Border at=132,414,964.5,51 by=clip in=30 limit=96,399,1036.5,81 paints=#707070FF role=none
                            47 Grid at=132,414,964.5,49.5 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=96,399,1036.5,81 role=none
                              48 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,414,162,48 in=30 owner=45 paints=#4C9AE8FF role=text
                              49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,414,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=95.08,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,414,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=SNES
                              51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,414,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=45 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        52 ListBoxItem at=96,480,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          53 Border at=132,495,964.5,51 by=clip in=30 limit=96,480,1036.5,81 paints=#707070FF role=none
                            54 Grid at=132,495,964.5,49.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,480,1036.5,81 role=none
                              55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,495,162,48 in=30 owner=52 paints=#4C9AE8FF role=text
                              56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,495,205.5,48 box=137,32 caps=18.1987 ellipsis face=Barlow heard=Core in=30 layout=132.18,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                              57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,495,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,495,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=156.22,32.27 owner=52 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        59 ListBoxItem at=96,561,1036.5,81 by=fixed-scroll focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,237,1036.5,781.5 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          60 Border at=132,576,964.5,51 by=clip in=30 limit=96,561,1036.5,81 paints=#707070FF role=none
                            61 Grid at=132,576,964.5,49.5 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,561,1036.5,81 role=none
                              62 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,576,162,49.5 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ install" in=30 layout=95.95,32.27 owner=59 paints=#4C9AE8FF role=text size=26 text="✓ install"
                              63 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,576,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=72.54,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,576,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,576,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=98.31,31.2 owner=59 paints=#8D8D91FF role=edit size=26 text=Available
                        66 ListBoxItem at=96,642,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          67 Border at=132,657,964.5,51 by=clip in=30 limit=96,642,1036.5,81 paints=#707070FF role=none
                            68 Grid at=132,657,964.5,49.5 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,642,1036.5,81 role=none
                              69 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,657,162,48 in=30 owner=66 paints=#4C9AE8FF role=text
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,657,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=73.45,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,657,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,657,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=66 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        73 ListBoxItem at=96,723,1036.5,81 by=fixed-scroll focus heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          74 Border at=132,738,964.5,51 by=clip in=30 limit=96,723,1036.5,81 paints=#707070FF role=none
                            75 Grid at=132,738,964.5,49.5 by=clip heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,723,1036.5,81 role=none
                              76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,738,162,49.5 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ update" in=30 layout=107.34,32.27 owner=73 paints=#4C9AE8FF role=text size=26 text="✓ update"
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,738,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=118.22,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,738,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              79 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,738,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=108.74,32.27 owner=73 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        80 ListBoxItem at=96,804,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          81 Border at=132,819,964.5,51 by=clip in=30 limit=96,804,1036.5,81 paints=#707070FF role=none
                            82 Grid at=132,819,964.5,49.5 by=clip heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=96,804,1036.5,81 role=none
                              83 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,819,162,48 in=30 owner=80 paints=#4C9AE8FF role=text
                              84 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,819,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=79.27,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                              85 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,819,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=SNES
                              86 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,819,237,49.5 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=123.07,32.27 owner=80 paints=#35A064FF role=edit size=26 text="✓ Installed"
                        87 ListBoxItem at=96,885,1036.5,81 by=fixed-scroll focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,237,1036.5,781.5 op paints=#D4D4D4FF role=listitem
                          88 Border at=132,900,964.5,49.5 by=clip in=30 limit=96,885,1036.5,81 paints=#707070FF role=none
                            89 Grid at=132,900,964.5,48 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,885,1036.5,81 role=none
                              90 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,900,162,48 in=30 owner=87 paints=#4C9AE8FF role=text
                              91 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=330,900,205.5,48 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=81.77,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                              92 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,900,216,48 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              93 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=823.5,900,237,48 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=157.48,31.2 owner=87 paints=#8D8D91FF role=edit size=26 text="Not supported"
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
edge 31 down 38
edge 31 left same
edge 31 right 12
edge 38 up 31
edge 38 down 45
edge 38 left same
edge 38 right 12
edge 45 up 38
edge 45 down 52
edge 45 left same
edge 45 right 12
edge 52 up 45
edge 52 down 59
edge 52 left same
edge 52 right 12
edge 59 up 52
edge 59 down 66
edge 59 left same
edge 59 right 12
edge 66 up 59
edge 66 down 73
edge 66 left same
edge 66 right 120
edge 73 up 66
edge 73 down 80
edge 73 left same
edge 73 right 120
edge 80 up 73
edge 80 down 87
edge 80 left same
edge 80 right 122
edge 87 up 80
edge 87 down same
edge 87 left same
edge 87 right 12
edge 120 up 66
edge 120 down 122
edge 120 left 66
edge 120 right 12
edge 122 up 120
edge 122 down 87
edge 122 left 80
edge 122 right 12
press pageup focus="Button  (No)" top=CoresWindow
press pagedown focus="Button  (No)" top=CoresWindow
press first focus="Button  (No)" top=CoresWindow
press last focus="Button  (No)" top=CoresWindow
press back focus="ListBoxItem  (Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available)" top=CoresWindow
