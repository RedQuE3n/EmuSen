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
      2 CoresPane at=48,28,1184,732 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,732 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
          4 Grid at=48,28,1184,732 by=panel limit=0,0,1280,800 role=none
            5 AdaptiveHeaderRow at=48,28,1184,60 by=panel heard=CORES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,48 by=panel limit=0,0,1280,800 role=none
                7 StackPanel at=574,51,642,26 by=panel heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" limit=0,0,1280,800 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=574,52,62,24 box=62,24 by=panel caps=13.999 face=Barlow heard="9 cores" layout=61.38,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="9 cores"
                  9 TextBlock at=636,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=649,52,85,24 box=85,24 by=panel caps=13.999 face=Barlow heard="7 installed" layout=84.88,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="7 installed"
                  11 TextBlock at=734,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=747,51,125,26 box=125,26 by=panel caps=14.7021 face="Adwaita Mono" heard="⚠ 1 to approve" layout=124.34,25.7 limit=0,0,1280,800 paints=#DAA520FF role=text size=20 text="⚠ 1 to approve"
                  13 TextBlock at=872,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  14 TextBlock at=885,51,94,26 box=94,26 by=panel caps=14.7021 face="Adwaita Mono" heard="↑ 1 update" layout=93.56,25.7 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="↑ 1 update"
                  15 TextBlock at=979,52,13,24 box=13,24 by=panel caps=13.999 face=Barlow heard=" · " layout=8.2,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text=" · "
                  16 TextBlock at=992,52,224,24 box=224,24 by=panel caps=13.999 face=Barlow heard="catalogue read 31 Oct 2026" layout=223.12,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="catalogue read 31 Oct 2026"
                17 StackPanel at=64,44,498,39 by=panel limit=0,0,1280,800 role=none
                  18 TextBlock at=64,44,498,39 box=498,39 by=panel caps=22.3984 face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=CORES
            19 DockPanel at=64,100,1152,44 by=panel limit=0,0,1280,800 role=none
              20 Border at=976,100,240,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                21 TextBox at=976,100,240,44 by=panel focus heard=Search limit=0,0,1280,800 paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  22 ScrollViewer allow="a text box scrolls its own text" at=987,107,218,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    23 TextBlock allow="a text box scrolls its own text" at=987,110,218,24 box=218,24 caps=13.999 face=Barlow heard=Search in=22 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              24 Border at=64,100,912,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                25 WholeRow at=64,100,912,44 by=panel limit=0,0,1280,800 role=tab
                  26 SectionTab at=80,100,34,44 by=panel focus heard=All limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=All
                    27 TextBlock at=80,108,34,24 box=34,24 by=clip caps=13.999 face=Barlow heard=ALL layout=33.56,24 limit=80,100,34,44 owner=26 paints=#D4D4D4FF role=text size=20 text=ALL
                  28 SectionTab at=146,100,154,44 by=panel focus heard="Super Nintendo" limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    29 TextBlock at=146,108,154,24 box=154,24 by=clip caps=13.999 face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=146,100,154,44 owner=28 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  30 SectionTab at=332,100,114,44 by=panel focus heard=PlayStation limit=0,0,1280,800 paints=#D4D4D4FF role=tabitem text=PlayStation
                    31 TextBlock at=332,108,114,24 box=114,24 by=clip caps=13.999 face=Barlow heard=PLAYSTATION layout=113.56,24 limit=332,100,114,44 owner=30 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  32 SectionTab at=478,100,128,44 by=panel focus heard="PlayStation 2" limit=0,0,1280,800 paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="PlayStation 2"
                    33 TextBlock at=478,108,128,24 box=128,24 by=clip caps=13.999 face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=478,100,128,44 owner=32 paints=#EEEEF0FF role=text size=20 text="PLAYSTATION 2"
            34 Grid at=64,160,1152,542 by=panel limit=0,0,1280,800 role=none
              35 Grid at=64,160,691,542 by=panel limit=0,0,1280,800 role=none
                36 Border at=64,160,691,432 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                  37 Grid at=64,160,691,432 by=panel limit=0,0,1280,800 paints=#0E0E10FF role=none
                    38 LunaTable at=64,160,691,432 by=panel heard=Rows limit=0,0,1280,800 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      39 ScrollViewer at=64,160,691,432 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,160,691,432
                        40 ListBoxItem at=64,160,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=39 limit=64,160,691,432 paints=#333337FF,#D4D4D4FF role=listitem
                          41 DockPanel at=64,160,691,53 by=clip in=39 limit=64,160,691,54 role=none
                            42 Border at=64,160,691,53 by=clip in=39 limit=64,160,691,54 paints=#0E0E10FF role=none
                              43 Grid at=88,160,667,53 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=39 limit=64,160,691,54 role=none
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=209,170,176,32 box=176,32 caps=18.1987 face=Barlow heard=Core in=39 layout=73.45,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                                45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,170,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=39 layout=143.29,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,170,154,34 box=154,34 caps=19.1128 face="Adwaita Mono" heard=Status in=39 layout=109.56,33.41 owner=40 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        47 ListBoxItem at=64,214,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=39 limit=64,160,691,432 paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel at=64,214,691,53 by=clip in=39 limit=64,214,691,54 role=none
                            49 Border at=64,214,691,53 by=clip in=39 limit=64,214,691,54 paints=#0E0E10FF role=none
                              50 Grid at=88,214,667,53 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=39 limit=64,214,691,54 role=none
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=209,224,176,32 box=176,32 caps=18.1987 face=Barlow heard=Core in=39 layout=99.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,224,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=39 layout=143.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,224,154,32 box=154,32 caps=18.1987 face=Barlow heard=Status in=39 layout=75.79,31.2 owner=47 paints=#8D8D91FF role=edit size=26 text="Built in"
                        54 ListBoxItem at=64,268,691,54 by=fixed-scroll focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=39 limit=64,160,691,432 paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel at=64,268,691,53 by=clip in=39 limit=64,268,691,54 role=none
                            56 Border at=64,268,691,53 by=clip in=39 limit=64,268,691,54 paints=#0E0E10FF role=none
                              57 Grid at=88,268,667,53 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=39 limit=64,268,691,54 role=none
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=209,278,176,32 box=176,32 caps=18.1987 face=Barlow heard=Core in=39 layout=162.4,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,278,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=39 layout=143.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,278,154,34 box=154,34 caps=19.1128 face="Adwaita Mono" heard=Status in=39 layout=153.45,33.41 owner=54 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        61 ListBoxItem at=64,322,691,54 by=fixed-scroll focus heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=39 limit=64,160,691,432 paints=#333337FF,#EEEEF0FF role=listitem
                          62 DockPanel at=64,322,691,53 by=clip in=39 limit=64,322,691,54 role=none
                            63 Border at=64,322,691,53 by=clip in=39 limit=64,322,691,54 paints=#26262BFF,#4C9AE8FF role=none
                              64 Grid at=88,322,667,53 by=clip heard="Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available" in=39 limit=64,322,691,54 role=none
                                65 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=88,332,97,34 box=97,34 caps=19.1128 face="Adwaita Mono" heard="✓ install" in=39 layout=96.85,33.41 owner=61 paints=#4C9AE8FF role=text size=26 text="✓ install"
                                66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=209,332,176,32 box=176,32 caps=18.1987 face=Barlow heard=Core in=39 layout=72.54,31.2 owner=61 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                                67 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=409,332,144,32 box=144,32 caps=18.1987 face=Barlow heard=Console in=39 layout=143.29,31.2 owner=61 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                                68 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=577,332,154,32 box=154,32 caps=18.1987 face=Barlow heard=Status in=39 layout=98.31,31.2 owner=61 paints=#8D8D91FF role=edit size=26 text=Available
                69 Border at=64,592,691,110 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
                  70 Grid at=64,610,691,92 by=panel limit=0,0,1280,800 role=none
                    71 Grid at=64,610,691,31 by=panel limit=0,0,1280,800 role=none
                      72 TextBlock at=64,614,691,27 box=691,27 by=panel caps=15.3989 face=Barlow heard="2 marked: install 1, update 1, about 8 MB to download" layout=479.95,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="2 marked: install 1, update 1, about 8 MB to download"
                    73 StackPanel at=154,658,601,44 by=panel limit=0,0,1280,800 role=none
                      74 Button at=154,658,170,44 by=panel focus heard="Apply the marked changes" limit=0,0,1280,800 paints=#707070FF,#D4D4D4FF role=button
                        75 StackPanel at=171,664,136,32 by=clip limit=154,658,170,44 role=none
                          76 PadGlyph at=171,669,22,22 by=clip heard=Menu limit=154,658,170,44 role=image
                          77 TextBlock at=203,664,83,32 box=83,32 by=clip caps=18.1987 face=Barlow heard="Apply…" layout=82.71,31.2 limit=154,658,170,44 owner=74 paints=#D4D4D4FF role=text size=26 text="Apply…"
                      78 StackPanel at=336,658,419,44 by=panel limit=0,0,1280,800 role=none
                        79 ProgressBar at=336,673,200,14 by=panel heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                        80 TextBlock at=544,668,211,24 box=211,24 by=panel caps=13.999 face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              81 Border at=771,160,445,542 by=panel limit=0,0,1280,800 name=AdaptiveDetail paints=#26262BFF role=none
                82 Border at=795,184,397,494 by=panel limit=0,0,1280,800 role=none
                  83 ScrollViewer at=795,184,397,494 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=795,184,397,494
                    84 StackPanel at=795,184,397,494 by=fixed-scroll in=83 limit=795,184,397,494 name=CoresDetail role=none
                      85 TextBlock at=795,184,397,39 box=397,39 by=fixed-scroll caps=22.3984 face="Barlow Condensed" heard=LRPS2 in=83 layout=64.35,38.4 limit=795,184,397,494 paints=#EEEEF0FF role=text size=32 text=LRPS2
                      86 TextBlock at=795,231,397,106 box=397,106 by=fixed-scroll caps=15.3989 face=Barlow heard="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”" in=83 layout=361.53,105.6 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”"
                      87 TextBlock at=795,345,397,80 box=397,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own." in=83 layout=382.6,79.2 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own."
                      88 TextBlock at=795,433,397,53 box=397,53 by=fixed-scroll caps=15.3989 face=Barlow heard="By PCSX2 Team, libretro, under the GPLv3. Nightly build of 31 October 2026." in=83 layout=387.66,52.8 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="By PCSX2 Team, libretro, under the GPLv3. Nightly build of 31 October 2026."
                      89 TextBlock at=795,494,397,53 box=397,53 by=fixed-scroll caps=15.3989 face=Barlow heard="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads." in=83 layout=364.5,52.8 limit=795,184,397,494 paints=#D4D4D4FF role=text size=22 text="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads."
                      90 TextBlock at=795,555,397,80 box=397,80 by=fixed-scroll caps=15.3989 face=Barlow heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=83 layout=358.86,79.2 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      91 TextBlock at=795,643,397,27 box=397,27 by=fixed-scroll caps=15.3989 face=Barlow heard="Not tested by EmuSen." in=83 layout=210.43,26.4 limit=795,184,397,494 paints=#8D8D91FF role=text size=22 text="Not tested by EmuSen."
            92 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              93 HintBar at=64,716,1152,30 by=panel heard="Choose, No" limit=0,0,1280,800 role=group
          94 Panel at=0,0,1280,702 by=panel limit=0,0,1280,800 name=AdaptiveCover paints=#000000B8 role=none
            95 Border at=280,169,720,364 by=panel limit=0,0,1280,800 paints=#26262BFF,#707070FF role=none
              96 DockPanel at=313,202,654,298 by=panel limit=0,0,1280,800 role=none
                97 TextBlock at=313,202,654,39 box=654,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply 2 changes?" layout=181.06,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply 2 changes?"
                98 StackPanel at=313,388,654,112 by=panel limit=0,0,1280,800 role=none
                  99 Button at=313,388,654,54 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    100 TextBlock at=341,399,31,32 box=31,32 by=clip caps=18.1987 drawn face=Barlow heard=No layout=30.68,31.2 limit=313,388,654,54 owner=99 paints=#EEEEF0FF role=text size=26 text=No
                  101 Button at=313,446,654,54 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#D4D4D4FF role=button
                    102 TextBlock at=341,457,40,32 box=40,32 by=clip caps=18.1987 drawn face=Barlow heard=Yes layout=39.36,31.2 limit=313,446,654,54 owner=101 paints=#D4D4D4FF role=text size=26 text=Yes
                103 ScrollViewer at=313,253,654,123 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=313,253,654,123
                  104 StackPanel at=313,253,654,123 by=fixed-scroll in=103 limit=313,253,654,123 role=none
                    105 TextBlock at=313,253,654,27 box=654,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Download and install 1 core from buildbot.libretro.com: LRPS2." in=103 layout=578.45,26.4 limit=313,253,654,123 paints=#D4D4D4FF role=text size=22 text="Download and install 1 core from buildbot.libretro.com: LRPS2."
                    106 TextBlock at=313,288,654,27 box=654,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Update Beetle PSX; the build installed now is kept for Roll Back." in=103 layout=579.92,26.4 limit=313,253,654,123 paints=#D4D4D4FF role=text size=22 text="Update Beetle PSX; the build installed now is kept for Roll Back."
                    107 TextBlock at=313,323,654,53 box=654,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Each core is checked before it is kept, and nothing downloaded runs until you approve it." in=103 layout=626.89,52.8 limit=313,253,654,123 paints=#D4D4D4FF role=text size=22 text="Each core is checked before it is kept, and nothing downloaded runs until you approve it."
focus 99
edge 99 up same
edge 99 down 101
edge 99 left same
edge 99 right same
edge 101 up 99
edge 101 down same
edge 101 left same
edge 101 right same
press pageup focus="Button  (No)" top=CoresWindow
press pagedown focus="Button  (No)" top=CoresWindow
press first focus="Button  (No)" top=CoresWindow
press last focus="Button  (No)" top=CoresWindow
press back focus="ListBoxItem  (Mark: install, Core: LRPS2, Console: PlayStation 2, Status: Available)" top=CoresWindow
