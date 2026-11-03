# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case CoresNoCatalogue
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1098 by=panel heard=Cores limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,90 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=459,76.5,1365,39 box=910,26 by=panel caps=13.999 drawn face=Barlow heard="4 cores · 4 installed · ⚠ 1 to approve · catalogue could not be read: buildbot.libretro.com could not be reached" layout=909.58,25.7 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="4 cores · 4 installed · ⚠ 1 to approve · catalogue could not be read: buildbot.libretro.com could not be reached"
                8 StackPanel at=96,66,345,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,345,58.5 box=230,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=13 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  17 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=219,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    20 TextBlock at=219,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=219,150,231,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  21 SectionTab at=498,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    22 TextBlock at=498,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=498,150,171,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 SectionTab at=717,150,192,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="PlayStation 2"
                    24 TextBlock at=717,162,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=717,150,192,66 owner=23 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
            25 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              26 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                27 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  28 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    29 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      30 ScrollViewer at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        31 ListBoxItem at=96,240,1036.5,129 by=fixed-scroll drawn focus heard="NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          32 DockPanel at=96,240,1036.5,127.5 by=clip in=30 limit=96,240,1036.5,129 role=none
                            33 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=96,240,1036.5,48 box=643,26 caps=13.999 drawn face=Barlow heard=NINTENDO in=30 layout=90.4,24 owner=31 paints=#8D8D91FF role=text size=20 text=NINTENDO
                            34 Border at=96,288,1036.5,79.5 by=clip in=30 limit=96,240,1036.5,129 paints=#0E0E10FF role=none
                              35 Grid at=132,288,1000.5,79.5 by=clip heard="Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=30 limit=96,240,1036.5,129 role=none
                                36 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,303,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=95.08,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                                37 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,303,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=171.05,31.2 owner=31 paints=#D4D4D4FF role=edit size=26 text="Super Nintendo"
                                38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,303,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=31 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        39 ListBoxItem at=96,369,1036.5,129 by=fixed-scroll drawn focus heard="SONY: Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          40 DockPanel at=96,369,1036.5,127.5 by=clip in=30 limit=96,369,1036.5,129 role=none
                            41 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=96,369,1036.5,48 box=643,26 caps=13.999 drawn face=Barlow heard=SONY in=30 layout=47.02,24 owner=39 paints=#8D8D91FF role=text size=20 text=SONY
                            42 Border at=96,417,1036.5,79.5 by=clip in=30 limit=96,369,1036.5,129 paints=#0E0E10FF role=none
                              43 Grid at=132,417,1000.5,79.5 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=30 limit=96,369,1036.5,129 role=none
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,432,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=98.67,31.2 owner=39 paints=#D4D4D4FF role=edit size=26 text=Diamond
                                45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,432,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=124.9,31.2 owner=39 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                46 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,432,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=109.56,33.41 owner=39 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        47 ListBoxItem at=96,498,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          48 DockPanel at=96,498,1036.5,79.5 by=clip in=30 limit=96,498,1036.5,81 role=none
                            49 Border at=96,498,1036.5,79.5 by=clip in=30 limit=96,498,1036.5,81 paints=#0E0E10FF role=none
                              50 Grid at=132,498,1000.5,79.5 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=30 limit=96,498,1036.5,81 role=none
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,513,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=99.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,513,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=47 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,513,231,48 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=30 layout=75.79,31.2 owner=47 paints=#8D8D91FF role=edit size=26 text="Built in"
                        54 ListBoxItem at=96,579,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel at=96,579,1036.5,79.5 by=clip in=30 limit=96,579,1036.5,81 role=none
                            56 Border at=96,579,1036.5,79.5 by=clip in=30 limit=96,579,1036.5,81 paints=#0E0E10FF role=none
                              57 Grid at=132,579,1000.5,79.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=30 limit=96,579,1036.5,81 role=none
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,594,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=30 layout=162.4,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,594,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=30 layout=143.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,594,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=30 layout=153.45,33.41 owner=54 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                61 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  62 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    63 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      64 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=402.38,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
                    65 StackPanel at=261,987,871.5,66 by=panel limit=0,0,1920,1200 role=none
                      66 Button at=261,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        67 StackPanel at=286.5,996,204,48 by=clip limit=261,987,255,66 off role=none
                          68 PadGlyph at=286.5,1003.5,33,33 by=clip heard=Menu limit=261,987,255,66 off role=image
                          69 TextBlock at=334.5,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=261,987,255,66 off owner=66 paints=#707070FF role=text size=26 text="Apply…"
                      70 StackPanel at=534,987,598.5,66 by=panel limit=0,0,1920,1200 role=none
                        71 ProgressBar at=534,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        72 TextBlock at=846,1002,286.5,36 box=191,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 0 B · 410 GB free" layout=190.96,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 0 B · 410 GB free"
              73 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                74 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  75 TextBlock at=1192.5,276,595.5,741 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=315.61,52.8 limit=0,0,1920,1200 name=CoresOverview paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
            76 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              77 HintBar at=96,1074,1728,45 by=panel heard="Back, Console, More, List, Apply…" limit=0,0,1920,1200 role=group
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
