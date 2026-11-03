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
      2 CoresPane at=72,42,1776,1098 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=868.5,76.5,955.5,39 box=637,26 by=panel caps=13.999 drawn face=Barlow heard="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026" layout=636.08,25.7 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="9 cores · 7 installed · ⚠ 1 to approve · ↑ 1 update · catalogue read 31 Oct 2026"
                8 StackPanel at=96,66,754.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,754.5,58.5 box=503,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=13 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=219,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Super Nintendo"
                    20 TextBlock at=219,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=219,150,231,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  21 ToggleButton at=498,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=498,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=498,150,171,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=717,150,192,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    24 TextBlock at=717,162,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=717,150,192,66 owner=23 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
            25 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=96,240,1036.5,648 fades=tb name=PART_ScrollViewer overflow=177 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        31 ListBoxItem at=96,774,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#EEEEF0FF role=listitem
                          32 DockPanel at=96,774,1036.5,79.5 by=clip in=30 limit=96,774,1036.5,81 role=none
                            33 Border at=96,774,1036.5,79.5 by=clip in=30 limit=96,774,1036.5,81 paints=#26262BFF,#4C9AE8FF role=none
                              34 Grid at=132,774,1000.5,79.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,774,1036.5,81 role=none
                                35 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,789,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="Extra PS2 Core"
                                36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,789,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=31 paints=#EEEEF0FF role=edit size=26 text="PlayStation 2"
                                37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,789,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=31 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                        38 ListBoxItem at=96,693,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel at=96,693,1036.5,79.5 by=clip in=30 limit=96,693,1036.5,81 role=none
                            40 Border at=96,693,1036.5,79.5 by=clip in=30 limit=96,693,1036.5,81 paints=#0E0E10FF role=none
                              41 Grid at=132,693,1000.5,79.5 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,693,1036.5,81 role=none
                                42 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,708,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,708,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,708,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=38 paints=#8D8D91FF role=edit size=26 text="Built in"
                        45 ListBoxItem at=96,612,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          46 DockPanel at=96,612,1036.5,79.5 by=clip in=30 limit=96,612,1036.5,81 role=none
                            47 Border at=96,612,1036.5,79.5 by=clip in=30 limit=96,612,1036.5,81 paints=#0E0E10FF role=none
                              48 Grid at=132,612,1000.5,79.5 by=clip heard="Mark: not marked, Core: PCEE2, Console: PlayStation 2, Status: ● Default" in=30 limit=96,612,1036.5,81 role=none
                                49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,627,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=73.45,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text=PCEE2
                                50 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,627,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=45 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,627,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=45 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        52 ListBoxItem at=96,531,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          53 DockPanel at=96,531,1036.5,79.5 by=clip in=30 limit=96,531,1036.5,81 role=none
                            54 Border at=96,531,1036.5,79.5 by=clip in=30 limit=96,531,1036.5,81 paints=#0E0E10FF role=none
                              55 Grid at=132,531,1000.5,79.5 by=clip heard="Mark: not marked, Core: Only GL, Console: PlayStation, Status: Not supported" in=30 limit=96,531,1036.5,81 role=none
                                56 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,546,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=81.77,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text="Only GL"
                                57 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,546,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=52 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,546,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=157.48,31.2 owner=52 paints=#8D8D91FF role=edit size=26 text="Not supported"
                        59 ListBoxItem at=96,450,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          60 DockPanel at=96,450,1036.5,79.5 by=clip in=30 limit=96,450,1036.5,81 role=none
                            61 Border at=96,450,1036.5,79.5 by=clip in=30 limit=96,450,1036.5,81 paints=#0E0E10FF role=none
                              62 Grid at=132,450,1000.5,79.5 by=clip heard="Mark: not marked, Core: Beetle PSX, Console: PlayStation, Status: ↑ Update" in=30 limit=96,450,1036.5,81 role=none
                                63 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,465,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=118.22,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text="Beetle PSX"
                                64 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,465,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=59 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,465,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=110.94,33.41 owner=59 paints=#DAA520FF role=edit size=26 text="↑ Update"
                        66 ListBoxItem at=96,321,1036.5,129 by=scroll drawn focus heard="SONY: Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          67 DockPanel at=96,321,1036.5,127.5 by=clip in=30 limit=96,321,1036.5,129 role=none
                            68 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=96,321,1036.5,48 box=643,26 caps=13.999 drawn face=Barlow heard=SONY in=30 layout=47.02,24 owner=66 paints=#8D8D91FF role=text size=20 text=SONY
                            69 Border at=96,369,1036.5,79.5 by=clip in=30 limit=96,321,1036.5,129 paints=#0E0E10FF role=none
                              70 Grid at=132,369,1000.5,79.5 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,321,1036.5,129 role=none
                                71 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,384,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=Diamond
                                72 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,384,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=66 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                73 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,384,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=66 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        74 ListBoxItem at=96,240,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: Snes9x, Console: Super Nintendo, Status: ✓ Installed" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          75 DockPanel at=96,240,1036.5,79.5 by=clip in=30 limit=96,240,1036.5,81 role=none
                            76 Border at=96,240,1036.5,79.5 by=clip in=30 limit=96,240,1036.5,81 paints=#0E0E10FF role=none
                              77 Grid at=132,240,1000.5,79.5 by=clip heard="Mark: not marked, Core: Snes9x, Console: Super Nintendo, Status: ✓ Installed" in=30 limit=96,240,1036.5,81 role=none
                                78 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,255,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=79.27,31.2 owner=74 paints=#D4D4D4FF role=edit size=26 text=Snes9x
                                79 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,255,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=171.05,31.2 owner=74 paints=#D4D4D4FF role=edit size=26 text="Super Nintendo"
                                80 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,255,237,51 box=158,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=123.97,33.41 owner=74 paints=#35A064FF role=edit size=26 text="✓ Installed"
                        81 ListBoxItem at=96,855,1036.5,81 by=scroll drawn focus heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem seen=96,855,1036.5,33
                          82 DockPanel at=96,855,1036.5,79.5 by=clip in=30 limit=96,855,1036.5,81 role=none seen=96,855,1036.5,33
                            83 Border at=96,855,1036.5,79.5 by=clip in=30 limit=96,855,1036.5,81 paints=#0E0E10FF role=none seen=96,855,1036.5,33
                              84 Grid at=132,855,1000.5,79.5 by=clip heard="Mark: not marked, Core: LRPS2, Console: PlayStation 2, Status: Available" in=30 limit=96,855,1036.5,81 role=none seen=132,855,1000.5,33
                                85 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,870,397.5,48 box=265,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=72.54,31.2 owner=81 paints=#D4D4D4FF role=edit seen=132,870,397.5,18 size=26 text=LRPS2
                                86 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=565.5,870,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=81 paints=#D4D4D4FF role=edit seen=565.5,870,258,18 size=26 text="PlayStation 2"
                                87 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=859.5,870,237,48 box=158,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=98.31,31.2 owner=81 paints=#8D8D91FF role=edit seen=859.5,870,237,18 size=26 text=Available
                88 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  89 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    90 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      91 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                    92 StackPanel at=231,987,901.5,66 by=panel limit=0,0,1920,1200 role=none
                      93 Button at=231,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        94 StackPanel at=256.5,996,204,48 by=clip limit=231,987,255,66 off role=none
                          95 PadGlyph at=256.5,1003.5,33,33 by=clip heard=Menu limit=231,987,255,66 off role=image
                          96 TextBlock at=304.5,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=231,987,255,66 off owner=93 paints=#707070FF role=text size=26 text="Apply…"
                      97 StackPanel at=504,987,628.5,66 by=panel limit=0,0,1920,1200 role=none
                        98 ProgressBar at=504,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        99 TextBlock at=816,1002,316.5,36 box=211,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 51 MB · 410 GB free" layout=210.14,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 51 MB · 410 GB free"
              100 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                101 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  102 ScrollViewer at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=1192.5,276,595.5,741
                    103 StackPanel at=1192.5,276,595.5,741 by=fixed-scroll in=102 limit=1192.5,276,595.5,741 name=CoresDetail role=none
                      104 TextBlock at=1192.5,276,595.5,58.5 box=397,39 by=fixed-scroll caps=22.3984 drawn face="Barlow Condensed" heard="Extra PS2 Core" in=102 layout=151.58,38.4 limit=1192.5,276,595.5,741 paints=#EEEEF0FF role=text size=32 text="Extra PS2 Core"
                      105 TextBlock at=1192.5,346.5,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Added by you · PlayStation 2" in=102 layout=259.03,26.4 limit=1192.5,276,595.5,741 paints=#8D8D91FF role=text size=22 text="Added by you · PlayStation 2"
                      106 TextBlock at=1192.5,399,595.5,79.5 box=397,53 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Runs PlayStation 2 games, from a file of your own." in=102 layout=363.86,52.8 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="Runs PlayStation 2 games, from a file of your own."
                      107 TextBlock at=1192.5,490.5,595.5,40.5 box=397,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="By Someone Else, under the MIT." in=102 layout=298.98,26.4 limit=1192.5,276,595.5,741 paints=#D4D4D4FF role=text size=22 text="By Someone Else, under the MIT."
                      108 TextBlock at=1192.5,543,595.5,82.5 box=397,55 by=fixed-scroll caps=16.1724 drawn face="Adwaita Mono" heard="⚠ Waiting for your approval. Nothing downloaded runs until you approve it." in=102 layout=346.9,54.67 limit=1192.5,276,595.5,741 paints=#DAA520FF role=text size=22 text="⚠ Waiting for your approval. Nothing downloaded runs until you approve it."
            109 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              110 HintBar at=96,1074,1728,45 by=panel heard="Approve…, Back, Console, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 31
edge 12 up same
edge 12 down 74
edge 12 left 23
edge 12 right same
edge 17 up same
edge 17 down 66
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 66
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 74
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 66
edge 23 left 21
edge 23 right 12
edge 31 up 38
edge 31 down 81
edge 31 left same
edge 31 right 12
edge 38 up 45
edge 38 down 31
edge 38 left same
edge 38 right 12
edge 45 up 52
edge 45 down 38
edge 45 left same
edge 45 right 12
edge 52 up 59
edge 52 down 45
edge 52 left same
edge 52 right 12
edge 59 up 66
edge 59 down 52
edge 59 left same
edge 59 right 12
edge 66 up 74
edge 66 down 59
edge 66 left same
edge 66 right 12
edge 74 up "row:ListBox/0/NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default"
edge 81 up 31
edge 81 down same
edge 81 left same
edge 81 right 12
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search (Search)" top=CoresWindow
press last focus="TextBox Search (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve)" top=CoresWindow
