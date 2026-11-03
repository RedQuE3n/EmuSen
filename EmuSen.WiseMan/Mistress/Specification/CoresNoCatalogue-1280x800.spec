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
      2 CoresPane at=48,28,1184,732 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,60 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=306,51,910,26 box=910,26 by=panel caps=13.999 drawn face=Barlow heard="4 cores · 4 installed · ⚠ 1 to approve · catalogue could not be read: buildbot.libretro.com could not be reached" layout=909.58,25.7 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="4 cores · 4 installed · ⚠ 1 to approve · catalogue could not be read: buildbot.libretro.com could not be reached"
                8 StackPanel at=64,44,230,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,230,39 box=230,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=13 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  17 SectionTab at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=146,100,154,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    20 TextBlock at=146,108,154,24 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=146,100,154,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  21 SectionTab at=332,100,114,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    22 TextBlock at=332,108,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=332,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 SectionTab at=478,100,128,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="PlayStation 2"
                    24 TextBlock at=478,108,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=478,100,128,44 owner=23 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
            25 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,160,691,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,160,691,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,160,691,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,160,691,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,432
                        31 ListBoxItem at=64,160,691,86 by=fixed-scroll drawn focus heard="NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=30 limit=64,160,691,432 op paints=#333337FF,#D4D4D4FF role=listitem
                          32 DockPanel at=64,160,691,85 by=clip in=30 limit=64,160,691,86 role=none
                            33 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=64,160,691,32 box=643,26 caps=13.999 drawn face=Barlow heard=NINTENDO in=30 layout=90.4,24 owner=31 paints=#8D8D91FF role=text size=20 text=NINTENDO
                            34 Border at=64,192,691,53 by=clip in=30 limit=64,160,691,86 paints=#0E0E10FF role=none
                              35 Grid at=88,192,667,53 by=clip heard="Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=30 limit=64,160,691,86 role=none
                                36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,202,269,32 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                                37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,202,172,32 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=171.05,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="Super Nintendo"
                                38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,202,154,34 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        39 ListBoxItem at=64,246,691,86 by=fixed-scroll drawn focus heard="SONY: Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,160,691,432 op paints=#333337FF,#D4D4D4FF role=listitem
                          40 DockPanel at=64,246,691,85 by=clip in=30 limit=64,246,691,86 role=none
                            41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=64,246,691,32 box=643,26 caps=13.999 drawn face=Barlow heard=SONY in=30 layout=47.02,24 owner=39 paints=#8D8D91FF role=text size=20 text=SONY
                            42 Border at=64,278,691,53 by=clip in=30 limit=64,246,691,86 paints=#0E0E10FF role=none
                              43 Grid at=88,278,667,53 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=64,246,691,86 role=none
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,288,269,32 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=39 paints=#D4D4D4FF role=edit size=26 text=Diamond
                                45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,288,172,32 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=39 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,288,154,34 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=39 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        47 ListBoxItem at=64,332,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,160,691,432 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel at=64,332,691,53 by=clip in=30 limit=64,332,691,54 role=none
                            49 Border at=64,332,691,53 by=clip in=30 limit=64,332,691,54 paints=#0E0E10FF role=none
                              50 Grid at=88,332,667,53 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,332,691,54 role=none
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,342,269,32 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,342,172,32 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,342,154,32 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=47 paints=#8D8D91FF role=edit size=26 text="Built in"
                        54 ListBoxItem at=64,386,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,160,691,432 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel at=64,386,691,53 by=clip in=30 limit=64,386,691,54 role=none
                            56 Border at=64,386,691,53 by=clip in=30 limit=64,386,691,54 paints=#0E0E10FF role=none
                              57 Grid at=88,386,667,53 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,386,691,54 role=none
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,396,269,32 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=381,396,172,32 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,396,154,34 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=54 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                61 Border at=64,592,691,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  62 Grid at=64,610,691,92 by=panel limit=0,0,1280,800 role=none
                    63 Grid at=64,610,691,31 by=panel limit=0,0,1280,800 role=none
                      64 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=402.38,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
                    65 StackPanel at=174,658,581,44 by=panel limit=0,0,1280,800 role=none
                      66 Button at=174,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                        67 StackPanel at=191,664,136,32 by=clip limit=174,658,170,44 off role=none
                          68 PadGlyph at=191,669,22,22 by=clip heard=Menu limit=174,658,170,44 off role=image
                          69 TextBlock at=223,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=174,658,170,44 off owner=66 paints=#707070FF role=text size=26 text="Apply…"
                      70 StackPanel at=356,658,399,44 by=panel limit=0,0,1280,800 role=none
                        71 ProgressBar at=356,673,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        72 TextBlock at=564,668,191,24 box=191,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 0 B · 410 GB free" layout=190.96,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 0 B · 410 GB free"
              73 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                74 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  75 TextBlock at=795,184,397,494 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=315.61,52.8 limit=0,0,1280,800 name=CoresOverview paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
            76 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              77 HintBar at=64,716,1152,30 by=panel heard="Back, Console, More, List, Apply…" limit=0,0,1280,800 role=group
focus 12
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
edge 31 down 39
edge 31 left same
edge 31 right 12
edge 39 up 31
edge 39 down 47
edge 39 left same
edge 39 right 12
edge 47 up 39
edge 47 down 54
edge 47 left same
edge 47 right 12
edge 54 up 47
edge 54 down same
edge 54 left same
edge 54 right 12
press pageup focus="TextBox Search (Search)" top=CoresWindow
press pagedown focus="TextBox Search (Search)" top=CoresWindow
press first focus="TextBox Search (Search)" top=CoresWindow
press last focus="TextBox Search (Search)" top=CoresWindow
press back focus="ListBoxItem  (NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default)" top=CoresWindow
