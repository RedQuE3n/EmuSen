# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state Cores-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case Cores
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1098 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=861,76.5,963,39 by=panel heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=861,78,93,36 box=62,24 by=panel caps=13.999 drawn face=Barlow heard="9 cores" layout=61.38,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="9 cores"
                  9 TextBlock at=954,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=973.5,78,127.5,36 box=85,24 by=panel caps=13.999 drawn face=Barlow heard="7 installed" layout=84.88,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="7 installed"
                  11 TextBlock at=1101,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=1120.5,76.5,187.5,39 box=125,26 by=panel caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 1 to approve" layout=124.34,25.7 limit=0,0,1920,1200 paints=#DAA520FF role=text size=20 text="⚠ 1 to approve"
                  13 TextBlock at=1308,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  14 TextBlock at=1327.5,76.5,141,39 box=94,26 by=panel caps=14.7021 drawn face="Adwaita Mono" heard="↑ 1 update" layout=93.56,25.7 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="↑ 1 update"
                  15 TextBlock at=1468.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  16 TextBlock at=1488,78,336,36 box=224,24 by=panel caps=13.999 drawn face=Barlow heard="catalogue read 31 Oct 2026" layout=223.12,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="catalogue read 31 Oct 2026"
                17 StackPanel at=96,66,747,58.5 by=panel limit=0,0,1920,1200 role=none
                  18 TextBlock at=96,66,747,58.5 box=498,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            19 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              20 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                21 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  22 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    23 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=22 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              24 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                25 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  26 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=All
                    27 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=26 paints=#D4D4D4FF role=text size=20 text=ALL
                  28 SectionTab at=219,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    29 TextBlock at=219,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=219,150,231,66 owner=28 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  30 SectionTab at=498,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    31 TextBlock at=498,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=498,150,171,66 owner=30 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  32 SectionTab at=717,150,192,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="PlayStation 2"
                    33 TextBlock at=717,162,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=717,150,192,66 owner=32 paints=#EEEEF0FF role=text size=20 text="PLAYSTATION 2"
            34 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              35 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                36 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  37 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    38 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      39 ScrollViewer at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        40 ListBoxItem at=96,240,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=39 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          41 DockPanel at=96,240,1036.5,79.5 by=clip in=39 limit=96,240,1036.5,81 role=none
                            42 Border at=96,240,1036.5,79.5 by=clip in=39 limit=96,240,1036.5,81 paints=#0E0E10FF role=none
                              43 Grid at=132,240,1000.5,79.5 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=39 limit=96,240,1036.5,81 role=none
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,255,445.5,48 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=39 layout=73.45,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                                45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=613.5,255,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=39 layout=143.29,31.2 owner=40 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,255,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=39 layout=109.56,33.41 owner=40 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        47 ListBoxItem at=96,321,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=39 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel at=96,321,1036.5,79.5 by=clip in=39 limit=96,321,1036.5,81 role=none
                            49 Border at=96,321,1036.5,79.5 by=clip in=39 limit=96,321,1036.5,81 paints=#0E0E10FF role=none
                              50 Grid at=132,321,1000.5,79.5 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=39 limit=96,321,1036.5,81 role=none
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,336,445.5,48 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=39 layout=99.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=613.5,336,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=39 layout=143.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,336,231,48 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=39 layout=75.79,31.2 owner=47 paints=#8D8D91FF role=edit size=26 text="Built in"
                        54 ListBoxItem at=96,402,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=39 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel at=96,402,1036.5,79.5 by=clip in=39 limit=96,402,1036.5,81 role=none
                            56 Border at=96,402,1036.5,79.5 by=clip in=39 limit=96,402,1036.5,81 paints=#0E0E10FF role=none
                              57 Grid at=132,402,1000.5,79.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=39 limit=96,402,1036.5,81 role=none
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,417,445.5,48 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=39 layout=162.4,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=613.5,417,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=39 layout=143.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,417,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=39 layout=153.45,33.41 owner=54 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        61 ListBoxItem at=96,483,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=39 limit=96,240,1036.5,648 op paints=#333337FF,#EEEEF0FF role=listitem
                          62 DockPanel at=96,483,1036.5,79.5 by=clip in=39 limit=96,483,1036.5,81 role=none
                            63 Border at=96,483,1036.5,79.5 by=clip in=39 limit=96,483,1036.5,81 paints=#26262BFF,#4C9AE8FF role=none
                              64 Grid at=132,483,1000.5,79.5 by=clip heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=39 limit=96,483,1036.5,81 role=none
                                65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,498,445.5,48 box=297,32 caps=18.1987 drawn face=Barlow heard=Core in=39 layout=72.54,31.2 owner=61 paints=#EEEEF0FF role=edit size=26 text=LRPS2
                                66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=613.5,498,216,48 box=144,32 caps=18.1987 drawn face=Barlow heard=Console in=39 layout=143.29,31.2 owner=61 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                                67 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,498,231,48 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=39 layout=98.31,31.2 owner=61 paints=#8D8D91FF role=edit size=26 text=Available
                68 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  69 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    70 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      71 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    72 StackPanel at=231,987,901.5,66 by=panel limit=0,0,1920,1200 role=none
                      73 Button at=231,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        74 StackPanel at=256.5,996,204,48 by=clip limit=231,987,255,66 off role=none
                          75 PadGlyph at=256.5,1003.5,33,33 by=clip heard=Menu limit=231,987,255,66 off role=image
                          76 TextBlock at=304.5,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=231,987,255,66 off owner=73 paints=#707070FF role=text size=26 text="Apply…"
                      77 StackPanel at=504,987,628.5,66 by=panel limit=0,0,1920,1200 role=none
                        78 ProgressBar at=504,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        79 TextBlock at=816,1002,316.5,36 box=211,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              80 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                81 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  82 ScrollViewer at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1192.5,276,595.5,741
                    83 StackPanel at=1192.5,276,595.5,741 by=fixed-scroll in=82 limit=1192.5,276,595.5,741 name=CoresDetail role=none
                      84 TextBlock at=1192.5,276,595.5,58.5 box=397,39 by=fixed-scroll caps=22.3984 drawn face="Barlow Condensed" heard=LRPS2 in=82 layout=64.35,38.4 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text=LRPS2
                      85 TextBlock at=1192.5,346.5,595.5,159 box=397,106 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”" in=82 layout=361.53,105.6 limit=1192.5,276,595.5,741 paints=#8D8D91FF role=text size=22 text="From libretro's buildbot · PlayStation 2 · listed there as “Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)”"
                      86 TextBlock at=1192.5,517.5,595.5,120 box=397,80 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own." in=82 layout=382.6,79.2 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games through EmuSen's libretro host, in a sandbox of its own."
                      87 TextBlock at=1192.5,649.5,595.5,79.5 box=397,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="By PCSX2 Team, libretro, under the GPLv3. Nightly build of 31 October 2026." in=82 layout=387.66,52.8 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="By PCSX2 Team, libretro, under the GPLv3. Nightly build of 31 October 2026."
                      88 TextBlock at=1192.5,741,595.5,79.5 box=397,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads." in=82 layout=364.5,52.8 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Needs a PlayStation 2 BIOS of your own, which EmuSen never downloads."
                      89 TextBlock at=1192.5,832.5,595.5,120 box=397,80 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=82 layout=358.86,79.2 limit=1192.5,276,595.5,741 paints=#8D8D91FF role=text size=22 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      90 TextBlock at=1192.5,964.5,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Not tested by EmuSen." in=82 layout=210.43,26.4 limit=1192.5,276,595.5,741 paints=#8D8D91FF role=text size=22 text="Not tested by EmuSen."
            91 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              92 HintBar at=96,1074,1728,45 by=panel heard="Install…, Back, Console, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 61
edge 21 up same
edge 21 down 40
edge 21 left 32
edge 21 right same
edge 26 up same
edge 26 down 40
edge 26 left same
edge 26 right 28
edge 28 up same
edge 28 down 40
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 40
edge 30 left 28
edge 30 right 32
edge 32 up same
edge 32 down 40
edge 32 left 30
edge 32 right 21
edge 40 up 26
edge 40 down 47
edge 40 left same
edge 40 right 21
edge 47 up 40
edge 47 down 54
edge 47 left same
edge 47 right 21
edge 54 up 47
edge 54 down 61
edge 54 left same
edge 54 right 21
edge 61 up 54
edge 61 down same
edge 61 left same
edge 61 right 21
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search (Search)" top=CoresWindow
press last focus="TextBox Search (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default)" top=CoresWindow
