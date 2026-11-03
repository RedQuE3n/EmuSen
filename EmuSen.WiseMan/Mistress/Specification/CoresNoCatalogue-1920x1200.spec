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
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard=CORES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 StackPanel at=453,76.5,1371,39 by=panel heard="4 cores · 4 installed · ⚠ 1 to approve · catalogue could not be read: buildbot.libretro.com could not be reached" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=453,78,93,36 box=62,24 by=panel caps=13.999 drawn face=Barlow heard="4 cores" layout=61.06,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="4 cores"
                  9 TextBlock at=546,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=565.5,78,129,36 box=86,24 by=panel caps=13.999 drawn face=Barlow heard="4 installed" layout=85.6,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="4 installed"
                  11 TextBlock at=694.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  12 TextBlock at=714,76.5,187.5,39 box=125,26 by=panel caps=14.7021 drawn face="Adwaita Mono" heard="⚠ 1 to approve" layout=124.34,25.7 limit=0,0,1920,1200 paints=#DAA520FF role=text size=20 text="⚠ 1 to approve"
                  13 TextBlock at=901.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  14 TextBlock at=921,78,903,36 box=602,24 by=panel caps=13.999 drawn face=Barlow heard="catalogue could not be read: buildbot.libretro.com could not be reached" layout=601.98,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="catalogue could not be read: buildbot.libretro.com could not be reached"
                15 StackPanel at=96,66,339,58.5 by=panel limit=0,0,1920,1200 role=none
                  16 TextBlock at=96,66,339,58.5 box=226,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=CORES layout=66.24,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=CORES
            17 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              18 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                19 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  20 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    21 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard=Search in=20 layout=59.32,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text=Search
              22 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                23 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  24 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    25 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=24 paints=#EEEEF0FF role=text size=20 text=ALL
                  26 SectionTab at=219,150,231,66 by=panel drawn focus heard="Super Nintendo" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Super Nintendo"
                    27 TextBlock at=219,162,231,36 box=154,24 by=clip caps=13.999 drawn face=Barlow heard="SUPER NINTENDO" layout=153.46,24 limit=219,150,231,66 owner=26 paints=#D4D4D4FF role=text size=20 text="SUPER NINTENDO"
                  28 SectionTab at=498,150,171,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=PlayStation
                    29 TextBlock at=498,162,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=498,150,171,66 owner=28 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  30 SectionTab at=717,150,192,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="PlayStation 2"
                    31 TextBlock at=717,162,192,36 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=717,150,192,66 owner=30 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
            32 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              33 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                34 Border at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  35 Grid at=96,240,1036.5,648 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    36 LunaTable at=96,240,1036.5,648 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      37 ScrollViewer at=96,240,1036.5,648 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,648
                        38 ListBoxItem at=96,240,1036.5,129 by=fixed-scroll drawn focus heard="NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=37 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          39 DockPanel at=96,240,1036.5,127.5 by=clip in=37 limit=96,240,1036.5,129 role=none
                            40 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=96,240,1036.5,48 box=643,26 caps=13.999 drawn face=Barlow heard=NINTENDO in=37 layout=90.4,24 owner=38 paints=#8D8D91FF role=text size=20 text=NINTENDO
                            41 Border at=96,288,1036.5,79.5 by=clip in=37 limit=96,240,1036.5,129 paints=#0E0E10FF role=none
                              42 Grid at=132,288,1000.5,79.5 by=clip heard="Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default" in=37 limit=96,240,1036.5,129 role=none
                                43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,303,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=37 layout=95.08,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text=VenusRT
                                44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,303,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=37 layout=171.05,31.2 owner=38 paints=#D4D4D4FF role=edit size=26 text="Super Nintendo"
                                45 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,303,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=37 layout=109.56,33.41 owner=38 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        46 ListBoxItem at=96,369,1036.5,129 by=fixed-scroll drawn focus heard="SONY: Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=37 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          47 DockPanel at=96,369,1036.5,127.5 by=clip in=37 limit=96,369,1036.5,129 role=none
                            48 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=96,369,1036.5,48 box=643,26 caps=13.999 drawn face=Barlow heard=SONY in=37 layout=47.02,24 owner=46 paints=#8D8D91FF role=text size=20 text=SONY
                            49 Border at=96,417,1036.5,79.5 by=clip in=37 limit=96,369,1036.5,129 paints=#0E0E10FF role=none
                              50 Grid at=132,417,1000.5,79.5 by=clip heard="Mark: not marked, Core: Diamond, Console: PlayStation, Status: ● Default" in=37 limit=96,369,1036.5,129 role=none
                                51 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,432,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=37 layout=98.67,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text=Diamond
                                52 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,432,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=37 layout=124.9,31.2 owner=46 paints=#D4D4D4FF role=edit size=26 text=PlayStation
                                53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,432,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=37 layout=109.56,33.41 owner=46 paints=#D4D4D4FF role=edit size=26 text="● Default"
                        54 ListBoxItem at=96,498,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=37 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          55 DockPanel at=96,498,1036.5,79.5 by=clip in=37 limit=96,498,1036.5,81 role=none
                            56 Border at=96,498,1036.5,79.5 by=clip in=37 limit=96,498,1036.5,81 paints=#0E0E10FF role=none
                              57 Grid at=132,498,1000.5,79.5 by=clip heard="Mark: not marked, Core: Sapphire, Console: PlayStation 2, Status: Built in" in=37 limit=96,498,1036.5,81 role=none
                                58 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,513,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=37 layout=99.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text=Sapphire
                                59 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,513,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=37 layout=143.29,31.2 owner=54 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                60 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,513,231,48 box=154,32 caps=18.1987 drawn face=Barlow heard=Status in=37 layout=75.79,31.2 owner=54 paints=#8D8D91FF role=edit size=26 text="Built in"
                        61 ListBoxItem at=96,579,1036.5,81 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=37 limit=96,240,1036.5,648 op paints=#333337FF,#D4D4D4FF role=listitem
                          62 DockPanel at=96,579,1036.5,79.5 by=clip in=37 limit=96,579,1036.5,81 role=none
                            63 Border at=96,579,1036.5,79.5 by=clip in=37 limit=96,579,1036.5,81 paints=#0E0E10FF role=none
                              64 Grid at=132,579,1000.5,79.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, Console: PlayStation 2, Status: ⚠ To approve" in=37 limit=96,579,1036.5,81 role=none
                                65 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=132,594,403.5,48 box=269,32 caps=18.1987 drawn face=Barlow heard=Core in=37 layout=162.4,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="Extra PS2 Core"
                                66 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=571.5,594,258,48 box=172,32 caps=18.1987 drawn face=Barlow heard=Console in=37 layout=143.29,31.2 owner=61 paints=#D4D4D4FF role=edit size=26 text="PlayStation 2"
                                67 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=865.5,594,231,51 box=154,34 caps=19.1128 drawn face="Adwaita Mono" heard=Status in=37 layout=153.45,33.41 owner=61 paints=#DAA520FF role=edit size=26 text="⚠ To approve"
                68 Border at=96,888,1036.5,165 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  69 Grid at=96,915,1036.5,138 by=panel limit=0,0,1920,1200 role=none
                    70 Grid at=96,915,1036.5,46.5 by=panel limit=0,0,1920,1200 role=none
                      71 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=402.38,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
                    72 StackPanel at=261,987,871.5,66 by=panel limit=0,0,1920,1200 role=none
                      73 Button at=261,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        74 StackPanel at=286.5,996,204,48 by=clip limit=261,987,255,66 off role=none
                          75 PadGlyph at=286.5,1003.5,33,33 by=clip heard=Menu limit=261,987,255,66 off role=image
                          76 TextBlock at=334.5,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=261,987,255,66 off owner=73 paints=#707070FF role=text size=26 text="Apply…"
                      77 StackPanel at=534,987,598.5,66 by=panel limit=0,0,1920,1200 role=none
                        78 ProgressBar at=534,1009.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                        79 TextBlock at=846,1002,286.5,36 box=191,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 0 B · 410 GB free" layout=190.96,24 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=20 text="/Cores 0 B · 410 GB free"
              80 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                81 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  82 TextBlock at=1192.5,276,595.5,741 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="Buildbot.libretro.com could not be reached." layout=315.61,52.8 limit=0,0,1920,1200 name=CoresOverview paints=#D4D4D4FF role=text size=22 text="Buildbot.libretro.com could not be reached."
            83 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              84 HintBar at=96,1074,1728,45 by=panel heard="Back, Console, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 19
edge 19 up same
edge 19 down 38
edge 19 left 30
edge 19 right same
edge 24 up same
edge 24 down 38
edge 24 left same
edge 24 right 26
edge 26 up same
edge 26 down 38
edge 26 left 24
edge 26 right 28
edge 28 up same
edge 28 down 38
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 38
edge 30 left 28
edge 30 right 19
edge 38 up 24
edge 38 down 46
edge 38 left same
edge 38 right 19
edge 46 up 38
edge 46 down 54
edge 46 left same
edge 46 right 19
edge 54 up 46
edge 54 down 61
edge 54 left same
edge 54 right 19
edge 61 up 54
edge 61 down same
edge 61 left same
edge 61 right 19
press pageup focus="TextBox Search (Search)" top=CoresWindow
press pagedown focus="TextBox Search (Search)" top=CoresWindow
press first focus="TextBox Search (Search)" top=CoresWindow
press last focus="TextBox Search (Search)" top=CoresWindow
press back focus="ListBoxItem  (NINTENDO: Mark: not marked, Core: VenusRT, Console: Super Nintendo, Status: ● Default)" top=CoresWindow
