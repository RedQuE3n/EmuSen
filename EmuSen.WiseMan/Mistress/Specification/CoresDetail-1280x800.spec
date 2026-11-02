# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=579,47,637,25 box=637,25 by=panel caps=13.999 drawn face=Barlow heard="8 cores · 6 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=637,24.82 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="8 cores · 6 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=64,40,503,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,503,39 box=503,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,91,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,101,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=13 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              15 Border at=64,91,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,91,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,99,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,91,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,91,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    20 TextBlock at=154,99,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,91,146,44 owner=19 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  21 ToggleButton at=292,91,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    22 TextBlock at=308,99,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,91,160,44 owner=21 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  23 ToggleButton at=460,91,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    24 TextBlock at=476,99,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,91,79,44 owner=23 paints=#D4D4D4FF role=text size=20 text=SNES
            25 Grid at=64,147,1152,599 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,147,691,599 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,147,691,532 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,147,691,532 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,147,691,532 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,158,691,521 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,158,691,521
                        31 ListBoxItem at=64,158,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          32 Border at=88,168,643,34 by=clip in=30 limit=64,158,691,54 paints=#707070FF role=none
                            33 Grid at=88,168,643,33 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: ● Default" in=30 limit=64,158,691,54 paints=#4C9AE8FF role=none
                              34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,168,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                              35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,168,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,168,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        37 ListBoxItem at=64,212,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          38 Border at=88,222,643,34 by=clip in=30 limit=64,212,691,54 paints=#707070FF role=none
                            39 Grid at=88,222,643,33 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,212,691,54 paints=#4C9AE8FF role=none
                              40 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,222,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=Diamond
                              41 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,222,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=37 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,222,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=37 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        43 ListBoxItem at=64,266,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          44 Border at=88,276,643,34 by=clip in=30 limit=64,266,691,54 paints=#707070FF role=none
                            45 Grid at=88,276,643,33 by=clip heard="Mark: not marked, Core: VenusRT, Console: SNES, Status: ● Default" in=30 limit=64,266,691,54 paints=#4C9AE8FF role=none
                              46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,276,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                              47 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,276,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=60.79,31.2 owner=43 paints=#D4D4D4FF role=edit size=26 text=SNES
                              48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,276,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=109.44,32.27 owner=43 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        49 ListBoxItem at=64,320,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,158,691,521 op paints=#4C9AE8FF,#EEEEF0FF role=listitem
                          50 Border at=88,330,643,34 by=clip in=30 limit=64,320,691,54 paints=#707070FF role=none
                            51 Grid at=88,330,643,33 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,320,691,54 paints=#4C9AE8FF role=none
                              52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,330,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=49 paints=#EEEEF0FF role=edit size=26 text="Extra PS2 Core"
                              53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,330,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=49 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                              54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,330,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=156.22,32.27 owner=49 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        55 ListBoxItem at=64,374,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          56 Border at=88,384,643,33 by=clip in=30 limit=64,374,691,54 paints=#707070FF role=none
                            57 Grid at=88,384,643,32 by=clip heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=64,374,691,54 paints=#4C9AE8FF role=none
                              58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,384,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=72.54,31.2 owner=55 paints=#D4D4D4FF role=edit size=26 text=LRPS2
                              59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,384,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=55 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                              60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,384,158,32 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=98.31,31.2 owner=55 paints=#8D8D91FF role=edit size=26 text=Available
                        61 ListBoxItem at=64,428,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: mednafen_psx, Console: PlayStation, Status: ↑ Update" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          62 Border at=88,438,643,34 by=clip in=30 limit=64,428,691,54 paints=#707070FF role=none
                            63 Grid at=88,438,643,33 by=clip heard="Mark: not marked, Core: mednafen_psx, Console: PlayStation, Status: ↑ Update" in=30 limit=64,428,691,54 paints=#4C9AE8FF role=none
                              64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,438,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.6,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text=mednafen_psx
                              65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,438,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,438,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=108.74,32.27 owner=61 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        67 ListBoxItem at=64,482,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          68 Border at=88,492,643,34 by=clip in=30 limit=64,482,691,54 paints=#707070FF role=none
                            69 Grid at=88,492,643,33 by=clip heard="Mark: not marked, Core: snes9x, Console: SNES, Status: ✓ Installed" in=30 limit=64,482,691,54 paints=#4C9AE8FF role=none
                              70 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,492,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=76.91,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text=snes9x
                              71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,492,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=60.79,31.2 owner=67 paints=#D4D4D4FF role=edit size=26 text=SNES
                              72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,492,158,33 box=158,33 caps=18.916 drawn face="Adwaita Sans" heard=Status in=30 layout=123.07,32.27 owner=67 paints=#35A064FF role=edit size=26 text="✓ Installed"
                        73 ListBoxItem at=64,536,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: glonly, Console: PlayStation, Status: Not supported" in=30 limit=64,158,691,521 op paints=#D4D4D4FF role=listitem
                          74 Border at=88,546,643,33 by=clip in=30 limit=64,536,691,54 paints=#707070FF role=none
                            75 Grid at=88,546,643,32 by=clip heard="Mark: not marked, Core: glonly, Console: PlayStation, Status: Not supported" in=30 limit=64,536,691,54 paints=#4C9AE8FF role=none
                              76 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=112,546,245,32 box=245,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=65.16,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=glonly
                              77 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,546,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=73 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                              78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=549,546,158,32 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=157.48,31.2 owner=73 paints=#8D8D91FF role=edit size=26 text="Not supported"
                79 Border at=64,679,691,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  80 Grid at=68,683,683,59 by=panel limit=0,0,1280,800 role=none
                    81 Grid at=68,683,683,27 by=panel limit=0,0,1280,800 role=none
                      82 TextBlock at=76,683,675,27 box=675,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    83 StackPanel at=336,718,415,24 by=panel limit=0,0,1280,800 role=none
                      84 StackPanel at=336,718,415,24 by=panel limit=0,0,1280,800 role=none
                        85 ProgressBar at=336,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        86 TextBlock at=544,718,207,24 box=207,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 11 MB · 410 GB free" layout=206.6,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 11 MB · 410 GB free"
              87 Border at=771,147,445,599 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                88 Border at=795,171,397,551 by=panel limit=0,0,1280,800 role=none
                  89 ScrollViewer at=803,179,381,535 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=803,179,381,535
                    90 StackPanel at=803,179,381,535 by=fixed-scroll in=89 limit=803,179,381,535 name=CoresDetail role=none
                      91 TextBlock at=803,179,381,39 box=381,39 by=fixed-scroll caps=22.3984 drawn face="Barlow Condensed" heard="Extra PS2 Core" in=89 layout=151.58,38.4 limit=803,179,381,535 paints=#EEEEF0FF role=text size=32 text="Extra PS2 Core"
                      92 TextBlock at=803,226,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Added by you · PlayStation 2" in=89 layout=259.03,26.4 limit=803,179,381,535 paints=#8D8D91FF role=text size=22 text="Added by you · PlayStation 2"
                      93 TextBlock at=803,261,381,53 box=381,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Runs PlayStation 2 games, from a file of your own." in=89 layout=363.86,52.8 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games, from a file of your own."
                      94 TextBlock at=803,322,381,27 box=381,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="By Someone Else, under the MIT." in=89 layout=298.98,26.4 limit=803,179,381,535 paints=#D4D4D4FF role=text size=22 text="By Someone Else, under the MIT."
                      95 TextBlock at=803,357,381,54 box=381,54 by=fixed-scroll caps=16.0059 drawn face="Adwaita Sans" heard="⚠ Waiting for your approval. Nothing downloaded runs until you approve it." in=89 layout=347.13,53.71 limit=803,179,381,535 paints=#DAA520FF role=text size=22 text="⚠ Waiting for your approval. Nothing downloaded runs until you approve it."
            96 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              97 HintBar at=96,757,1088,29 by=panel heard="Approve…, Back, Console, List, Apply…, More" limit=0,0,1280,800 role=group
focus 49
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
edge 73 down same
edge 73 left same
edge 73 right 12
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: ● Default)" top=CoresWindow
