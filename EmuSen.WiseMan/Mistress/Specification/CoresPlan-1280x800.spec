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
            5 Border at=48,28,1184,51 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=579,47,637,25 box=637,25 by=panel caps=13.999 face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.52,24.82 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=64,40,503,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,503,39 box=503,39 by=panel caps=22.3984 face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,146,44 by=panel focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    20 TextBlock at=154,99,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,91,146,44 owner=19 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  21 ToggleButton at=292,91,160,44 by=panel focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    22 TextBlock at=308,99,128,24 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,91,160,44 owner=21 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  23 ToggleButton at=460,91,79,44 by=panel focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=476,99,47,24 box=47,24 by=clip caps=13.999 face=Barlow heard=SNES layout=46.76,24 limit=460,91,79,44 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,147,691,532 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,147,691,532 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,147,691,532 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,158,691,521 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,521
                        31 ListBoxItem at=64,158,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          32 Border at=88,168,643,33 by=clip in=30 limit=64,158,691,54 paints=#707070FF role=none
                            33 Grid at=88,168,643,32 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,158,691,54 role=none
                              34 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,168,108,32 in=30 owner=31 paints=#4C9AE8FF role=text
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,168,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,168,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,168,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=75.79,31.2 owner=31 paints=#8D8D91FF role=edit size=26 text="Built in"
                        38 ListBoxItem at=64,212,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          39 Border at=88,222,643,34 by=clip in=30 limit=64,212,691,54 paints=#707070FF role=none
                            40 Grid at=88,222,643,33 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,212,691,54 role=none
                              41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,222,108,32 in=30 owner=38 paints=#4C9AE8FF role=text
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,222,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=98.67,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,222,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,222,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=38 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        45 ListBoxItem at=64,266,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          46 Border at=88,276,643,34 by=clip in=30 limit=64,266,691,54 paints=#707070FF role=none
                            47 Grid at=88,276,643,33 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,266,691,54 role=none
                              48 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,276,108,32 in=30 owner=45 paints=#4C9AE8FF role=text
                              49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,276,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=95.08,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,276,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=SNES
                              51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,276,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=45 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        52 ListBoxItem at=64,320,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          53 Border at=88,330,643,34 by=clip in=30 limit=64,320,691,54 paints=#707070FF role=none
                            54 Grid at=88,330,643,33 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,320,691,54 role=none
                              55 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,330,108,32 in=30 owner=52 paints=#4C9AE8FF role=text
                              56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,330,137,32 box=137,32 caps=18.1987 ellipsis face=Barlow heard=Core in=30 layout=132.18,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                              57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,330,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,330,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=156.22,32.27 owner=52 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        59 ListBoxItem at=64,374,691,54 by=fixed-scroll focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,158,691,521 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          60 Border at=88,384,643,34 by=clip in=30 limit=64,374,691,54 paints=#707070FF role=none
                            61 Grid at=88,384,643,33 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,374,691,54 role=none
                              62 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,384,108,33 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ install" in=30 layout=95.95,32.27 owner=59 paints=#4C9AE8FF role=text size=26 text="✓ install"
                              63 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,384,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=72.54,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,384,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=59 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,384,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=98.31,31.2 owner=59 paints=#8D8D91FF role=edit size=26 text=Available
                        66 ListBoxItem at=64,428,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          67 Border at=88,438,643,34 by=clip in=30 limit=64,428,691,54 paints=#707070FF role=none
                            68 Grid at=88,438,643,33 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,428,691,54 role=none
                              69 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,438,108,32 in=30 owner=66 paints=#4C9AE8FF role=text
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,438,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=73.45,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,438,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=143.29,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,438,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=66 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        73 ListBoxItem at=64,482,691,54 by=fixed-scroll focus heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          74 Border at=88,492,643,34 by=clip in=30 limit=64,482,691,54 paints=#707070FF role=none
                            75 Grid at=88,492,643,33 by=clip heard="Mark: update, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=64,482,691,54 role=none
                              76 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,492,108,33 box=108,33 caps=18.916 face="Adwaita Sans" heard="✓ update" in=30 layout=107.34,32.27 owner=73 paints=#4C9AE8FF role=text size=26 text="✓ update"
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,492,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=118.22,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,492,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              79 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,492,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=108.74,32.27 owner=73 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        80 ListBoxItem at=64,536,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          81 Border at=88,546,643,34 by=clip in=30 limit=64,536,691,54 paints=#707070FF role=none
                            82 Grid at=88,546,643,33 by=clip heard="Mark: not marked, Core: Snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,536,691,54 role=none
                              83 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,546,108,32 in=30 owner=80 paints=#4C9AE8FF role=text
                              84 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,546,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=79.27,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                              85 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,546,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=60.79,31.2 owner=80 paints=#D4D4D4FF role=edit size=26 text=SNES
                              86 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,546,158,33 box=158,33 caps=18.916 face="Adwaita Sans" heard=Status in=30 layout=123.07,32.27 owner=80 paints=#35A064FF role=edit size=26 text="✓ Installed"
                        87 ListBoxItem at=64,590,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          88 Border at=88,600,643,33 by=clip in=30 limit=64,590,691,54 paints=#707070FF role=none
                            89 Grid at=88,600,643,32 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=64,590,691,54 role=none
                              90 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,600,108,32 in=30 owner=87 paints=#4C9AE8FF role=text
                              91 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=220,600,137,32 box=137,32 caps=18.1987 face=Barlow heard=Core in=30 layout=81.77,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                              92 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,600,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=30 layout=124.9,31.2 owner=87 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              93 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,600,158,32 box=158,32 caps=18.1987 face=Barlow heard=Status in=30 layout=157.48,31.2 owner=87 paints=#8D8D91FF role=edit size=26 text="Not supported"
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
