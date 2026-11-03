# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresRunning
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
                7 TextBlock at=579,51,637,26 box=637,26 by=panel caps=13.999 drawn face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.08,25.7 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=64,44,503,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,44,503,39 box=503,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              11 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=976,100,240,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=13 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              15 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=80,100,34,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=All
                    18 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=17 paints=#D4D4D4FF role=text size=20 text=ALL
                  19 ToggleButton at=146,100,154,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Super Nintendo"
                    20 TextBlock at=146,108,154,24 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=146,100,154,44 owner=19 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  21 ToggleButton at=332,100,114,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=332,108,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=332,100,114,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=478,100,128,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text="PlayStation 2"
                    24 TextBlock at=478,108,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=478,100,128,44 owner=23 paints=#EEEEF0FF role=text size=20 text="PLAYSTATION 2"
            25 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              26 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                27 Border at=64,160,691,424 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  28 Grid at=64,160,691,424 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    29 LunaTable at=64,160,691,424 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=64,160,691,424 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,424
                        31 ListBoxItem at=64,160,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,160,691,424 op paints=#333337FF,#D4D4D4FF role=listitem
                          32 DockPanel at=64,160,691,53 by=clip in=30 limit=64,160,691,54 role=none
                            33 Border at=64,160,691,53 by=clip in=30 limit=64,160,691,54 paints=#0E0E10FF role=none
                              34 Grid at=88,160,667,53 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=64,160,691,54 role=none
                                35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,170,297,32 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=73.45,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                                36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,170,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,170,154,34 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        38 ListBoxItem at=64,214,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,160,691,424 op paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel at=64,214,691,53 by=clip in=30 limit=64,214,691,54 role=none
                            40 Border at=64,214,691,53 by=clip in=30 limit=64,214,691,54 paints=#0E0E10FF role=none
                              41 Grid at=88,214,667,53 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=64,214,691,54 role=none
                                42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,224,297,32 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,224,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,224,154,32 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=38 paints=#8D8D91FF role=edit size=26 text="Built in"
                        45 ListBoxItem at=64,268,691,54 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,160,691,424 op paints=#333337FF,#D4D4D4FF role=listitem
                          46 DockPanel at=64,268,691,53 by=clip in=30 limit=64,268,691,54 role=none
                            47 Border at=64,268,691,53 by=clip in=30 limit=64,268,691,54 paints=#0E0E10FF role=none
                              48 Grid at=88,268,667,53 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=64,268,691,54 role=none
                                49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,278,297,32 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,278,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,278,154,34 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=45 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        52 ListBoxItem at=64,322,691,54 by=fixed-scroll drawn focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Installing…" in=30 limit=64,160,691,424 op paints=#333337FF,#EEEEF0FF role=listitem
                          53 DockPanel at=64,322,691,53 by=clip in=30 limit=64,322,691,54 role=none
                            54 Border at=64,322,691,53 by=clip in=30 limit=64,322,691,54 paints=#26262BFF,#4C9AE8FF role=none
                              55 Grid at=88,322,667,53 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Installing…" in=30 limit=64,322,691,54 role=none
                                56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,332,297,32 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=72.54,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                                57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,332,144,32 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=52 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,332,154,32 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=120.38,31.2 owner=52 paints=#8D8D91FF role=edit size=26 text="Installing…"
                59 Border at=64,584,691,118 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  60 Grid at=64,602,691,100 by=panel limit=0,0,1280,800 role=none
                    61 Grid at=64,602,691,39 by=panel limit=0,0,1280,800 role=none
                      62 ProgressBar at=64,602,691,8 by=panel drawn heard=Progress limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                      63 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Installing LRPS2: 18 MB of 41 MB" layout=290.69,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Installing LRPS2: 18 MB of 41 MB"
                    64 StackPanel at=154,658,601,44 by=panel limit=0,0,1280,800 role=none
                      65 Button at=154,658,170,44 by=panel drawn heard="Apply the marked changes" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button
                        66 StackPanel at=171,664,136,32 by=clip limit=154,658,170,44 off role=none
                          67 PadGlyph at=171,669,22,22 by=clip heard=Menu limit=154,658,170,44 off role=image
                          68 TextBlock at=203,664,83,32 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=154,658,170,44 off owner=65 paints=#707070FF role=text size=26 text="Apply…"
                      69 StackPanel at=336,658,419,44 by=panel limit=0,0,1280,800 role=none
                        70 ProgressBar at=336,673,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        71 TextBlock at=544,668,211,24 box=211,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              72 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                73 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  74 ScrollViewer at=795,184,397,494 by=panel fades=b limit=0,0,1280,800 overflow=12 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=795,184,397,494
                    75 StackPanel at=795,184,397,506 by=scroll in=74 limit=795,184,397,494 name=CoresDetail role=none seen=795,184,397,494
                      76 TextBlock at=795,184,397,39 box=397,39 by=scroll caps=22.3984 drawn face="Barlow Condensed" heard=LRPS2 in=74 layout=64.35,38.4 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text=LRPS2
                      77 TextBlock at=795,231,397,106 box=397,106 by=scroll caps=15.3989 drawn face=Barlow heard="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”" in=74 layout=361.53,105.6 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”"
                      78 TextBlock at=795,345,397,53 box=397,53 by=scroll caps=15.3989 drawn face=Barlow heard="Downloading the nightly build of 31 October 2026 from buildbot.libretro.com, 41 MB." in=74 layout=395.8,52.8 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Downloading the nightly build of 31 October 2026 from buildbot.libretro.com, 41 MB."
                      79 StackPanel at=795,414,397,207 by=scroll in=74 limit=795,184,397,494 role=none
                        80 TextBlock at=795,414,397,55 box=397,55 by=scroll caps=16.1724 drawn face="Adwaita Mono" heard="✓ Secure connection to buildbot.libretro.com" in=74 layout=222.9,54.67 limit=795,184,397,494 paints=#35A064FF role=text size=22 text="✓ Secure connection to buildbot.libretro.com"
                        81 TextBlock at=795,473,397,29 box=397,29 by=scroll caps=16.1724 drawn face="Adwaita Mono" heard="↓ Download · 18 MB of 41 MB" in=74 layout=260.63,28.27 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="↓ Download · 18 MB of 41 MB"
                        82 TextBlock at=795,506,397,53 box=397,53 by=scroll caps=15.3989 drawn face=Barlow heard="· Checksum, one file of the right name, its size" in=74 layout=376.18,52.8 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="· Checksum, one file of the right name, its size"
                        83 TextBlock at=795,563,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="· Built for this machine" in=74 layout=205.92,26.4 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="· Built for this machine"
                        84 TextBlock at=795,594,397,27 box=397,27 by=scroll caps=15.3989 drawn face=Barlow heard="· SHA-256 recorded" in=74 layout=179.63,26.4 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="· SHA-256 recorded"
                      85 TextBlock at=795,637,397,53 box=397,53 by=scroll caps=15.3989 drawn face=Barlow heard="Then it waits for your approval: nothing downloaded runs until you approve it." in=74 layout=362.49,52.8 limit=795,184,397,494 paints=#D4D4D4FF role=text seen=795,637,397,41 size=22 text="Then it waits for your approval: nothing downloaded runs until you approve it."
            86 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              87 HintBar at=64,716,1152,30 by=panel heard="Back, Console, List" limit=0,0,1280,800 role=group
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
edge 52 down same
edge 52 left same
edge 52 right 12
press pageup focus="TextBox Search (Search)" top=CoresWindow
press pagedown focus="TextBox Search (Search)" top=CoresWindow
press first focus="TextBox Search (Search)" top=CoresWindow
press last focus="TextBox Search (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default)" top=CoresWindow
