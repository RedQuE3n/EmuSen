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
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=579,46,637,26 box=637,26 by=panel caps=13.999 face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.08,25.7 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=64,40,503,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,503,39 box=503,39 by=panel caps=22.3984 face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel focus heard=All limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,160,44 by=panel focus heard="PlayStation 2" limit=0,0,1280,800 paints=#D4D4D4FF role=button text="PlayStation 2"
                    20 TextBlock at=154,99,128,24 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=138,91,160,44 owner=19 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  21 ToggleButton at=306,91,146,44 by=panel focus heard=PlayStation limit=0,0,1280,800 paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=322,99,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=306,91,146,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=460,91,79,44 by=panel focus heard=SNES limit=0,0,1280,800 paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=476,99,47,24 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=460,91,79,44 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,147,691,532 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,147,691,532 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,147,691,532 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,158,691,521 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,521
                        31 ListBoxItem at=64,158,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          32 Border at=88,168,643,35 by=clip in=30 limit=64,158,691,54 paints=#707070FF role=none
                            33 Grid at=88,168,643,34 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,158,691,54 role=none
                              34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,169,109,32 in=30 owner=31 paints=#4C9AE8FF role=text
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,169,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=73.45,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,169,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,168,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        38 ListBoxItem at=64,212,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          39 Border at=88,222,643,35 by=clip in=30 limit=64,212,691,54 paints=#707070FF role=none
                            40 Grid at=88,222,643,34 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,212,691,54 role=none
                              41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,223,109,32 in=30 owner=38 paints=#4C9AE8FF role=text
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,223,136,32 box=136,32 caps=18.1987 ellipsis face=Barlow heard=Core in=30 layout=132.18,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                              43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,223,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,222,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=38 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        45 ListBoxItem at=64,266,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          46 Border at=88,276,643,33 by=clip in=30 limit=64,266,691,54 paints=#707070FF role=none
                            47 Grid at=88,276,643,32 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,266,691,54 role=none
                              48 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,276,109,32 in=30 owner=45 paints=#4C9AE8FF role=text
                              49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,276,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=99.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,276,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,276,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=75.79,31.2 owner=45 paints=#8D8D91FF role=edit size=26 text="Built in"
                        52 ListBoxItem at=64,320,691,54 by=fixed-scroll focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,158,691,521 paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          53 Border at=88,330,643,35 by=clip in=30 limit=64,320,691,54 paints=#707070FF role=none
                            54 Grid at=88,330,643,34 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,320,691,54 role=none
                              55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,330,109,34 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ install" in=30 layout=96.85,33.41 owner=52 paints=#4C9AE8FF role=text size=26 text="✓ install"
                              56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,331,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=72.54,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                              57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,331,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,331,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=98.31,31.2 owner=52 paints=#8D8D91FF role=edit size=26 text=Available
                        59 ListBoxItem at=64,374,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          60 Border at=88,384,643,35 by=clip in=30 limit=64,374,691,54 paints=#707070FF role=none
                            61 Grid at=88,384,643,34 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,374,691,54 role=none
                              62 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,385,109,32 in=30 owner=59 paints=#4C9AE8FF role=text
                              63 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,385,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=98.67,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,385,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,384,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=59 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        66 ListBoxItem at=64,428,691,54 by=fixed-scroll focus heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          67 Border at=88,438,643,35 by=clip in=30 limit=64,428,691,54 paints=#707070FF role=none
                            68 Grid at=88,438,643,34 by=clip heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=64,428,691,54 role=none
                              69 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,438,109,34 box=109,34 caps=19.1128 face="Adwaita Mono" heard="✓ update" in=30 layout=108.24,33.41 owner=66 paints=#4C9AE8FF role=text size=26 text="✓ update"
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,439,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=118.22,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,439,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,438,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=110.94,33.41 owner=66 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        73 ListBoxItem at=64,482,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          74 Border at=88,492,643,33 by=clip in=30 limit=64,482,691,54 paints=#707070FF role=none
                            75 Grid at=88,492,643,32 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=64,482,691,54 role=none
                              76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,492,109,32 in=30 owner=73 paints=#4C9AE8FF role=text
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,492,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=81.77,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,492,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              79 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,492,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=157.48,31.2 owner=73 paints=#8D8D91FF role=edit size=26 text="Not supported"
                        80 ListBoxItem at=64,536,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          81 Border at=88,546,643,35 by=clip in=30 limit=64,536,691,54 paints=#707070FF role=none
                            82 Grid at=88,546,643,34 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,536,691,54 role=none
                              83 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,547,109,32 in=30 owner=80 paints=#4C9AE8FF role=text
                              84 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,547,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=95.08,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              85 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,547,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=SNES
                              86 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,546,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=80 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        87 ListBoxItem at=64,590,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,158,691,521 paints=#D4D4D4FF role=listitem
                          88 Border at=88,600,643,35 by=clip in=30 limit=64,590,691,54 paints=#707070FF role=none
                            89 Grid at=88,600,643,34 by=clip heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,590,691,54 role=none
                              90 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,601,109,32 in=30 owner=87 paints=#4C9AE8FF role=text
                              91 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=221,601,136,32 box=136,32 caps=18.1987 face=Barlow heard=Core in=30 layout=79.27,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                              92 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,601,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=SNES
                              93 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,600,158,34 box=158,34 caps=19.1128 face="Adwaita Mono" heard=Status in=30 layout=123.97,33.41 owner=87 paints=#35A064FF role=edit size=26 text="✓ Installed"
                94 Border at=64,679,691,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  95 Grid at=68,683,683,59 by=panel limit=0,0,1280,800 role=none
                    96 Grid at=68,683,683,27 by=panel limit=0,0,1280,800 role=none
                      97 TextBlock at=76,683,675,27 box=675,27 by=panel caps=15.3989 face=Barlow heard="2 marked: install 1, update 1, about 8 MB to download" layout=479.95,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 marked: install 1, update 1, about 8 MB to download"
                    98 StackPanel at=332,718,419,24 by=panel limit=0,0,1280,800 role=none
                      99 StackPanel at=332,718,419,24 by=panel limit=0,0,1280,800 role=none
                        100 ProgressBar at=332,723,200,14 by=panel heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        101 TextBlock at=540,718,211,24 box=211,24 by=panel caps=13.999 face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              102 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                103 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  104 ScrollViewer at=803,179,381,535 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=803,179,381,535
                    105 StackPanel at=803,179,381,535 by=fixed-scroll in=104 limit=803,179,381,535 name=CoresDetail role=none
                      106 TextBlock at=803,179,381,39 box=381,39 by=fixed-scroll caps=22.3984 face="Barlow Condensed" heard=LRPS2 in=104 layout=64.35,38.4 limit=803,179,381,535 paints=#EEEEF0FF role=text size=32 text=LRPS2
                      107 TextBlock at=803,226,381,106 box=381,106 by=fixed-scroll caps=15.3989 face=Barlow heard="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”" in=104 layout=361.53,105.6 limit=803,179,381,535 paints=#8D8D91FF role=text size=22 text="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”"
                      108 TextBlock at=803,340,381,80 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own." in=104 layout=355.48,79.2 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own."
                      109 TextBlock at=803,428,381,53 box=381,53 by=fixed-scroll caps=15.3989 face=Barlow heard="By PCSX2 Team, libretro, under the GPLv3. Build 2.3.1 2026-10-31." in=104 layout=319.66,52.8 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="By PCSX2 Team, libretro, under the GPLv3. Build 2.3.1 2026-10-31."
                      110 TextBlock at=803,489,381,53 box=381,53 by=fixed-scroll caps=15.3989 face=Barlow heard="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads." in=104 layout=364.5,52.8 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads."
                      111 TextBlock at=803,550,381,80 box=381,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=104 layout=358.86,79.2 limit=803,179,381,535 paints=#8D8D91FF role=text size=22 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      112 TextBlock at=803,638,381,27 box=381,27 by=fixed-scroll caps=15.3989 face=Barlow heard="Not tested by EmuSen." in=104 layout=210.43,26.4 limit=803,179,381,535 paints=#8D8D91FF role=text size=22 text="Not tested by EmuSen."
            113 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              114 HintBar at=96,757,1088,29 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          115 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            116 Border at=280,210,720,400 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              117 DockPanel at=313,243,654,334 by=panel limit=0,0,1280,800 role=none
                118 TextBlock at=313,243,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply 2 changes?" layout=181.06,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply 2 changes?"
                119 StackPanel at=313,465,654,112 by=panel limit=0,0,1280,800 role=none
                  120 Button at=313,465,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    121 TextBlock at=341,476,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,465,654,54 owner=120 paints=#EEEEF0FF role=text size=26 text=No
                  122 Button at=313,523,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    123 TextBlock at=341,534,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,523,654,54 owner=122 paints=#D4D4D4FF role=text size=26 text=Yes
                124 ScrollViewer at=313,294,654,159 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,294,654,159
                  125 TextBlock at=313,294,654,159 box=654,159 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Download and install 1 core from buildbot.libretro.com: LRPS2.\n\nUpdate Beetle PSX; the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=124 layout=626.89,158.4 limit=313,294,654,159 paints=#D4D4D4FF role=text size=22 text="Download and install 1 core from buildbot.libretro.com: LRPS2.\n\nUpdate Beetle PSX; the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
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
