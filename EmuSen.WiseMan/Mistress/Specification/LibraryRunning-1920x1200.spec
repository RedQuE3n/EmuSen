# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case LibraryRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 LibraryPane at=72,42,1776,1098 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1098 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
          4 Grid at=72,42,1776,1098 by=panel limit=0,0,1920,1200 role=none
            5 AdaptiveHeaderRow at=72,42,1776,90 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,72 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1134,78,690,36 box=460,24 by=panel caps=13.999 drawn face=Barlow heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=459.1,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=96,66,1020,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,66,1020,58.5 box=680,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=13 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              15 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  17 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    18 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 SectionTab at=219,150,154.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Dump"
                    20 TextBlock at=219,162,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=219,150,154.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 SectionTab at=421.5,150,151.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Game"
                    22 TextBlock at=421.5,162,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,150,151.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 SectionTab at=621,150,276,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    24 TextBlock at=621,162,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=621,150,276,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 SectionTab at=945,150,205.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Unrecognised
                    26 TextBlock at=945,162,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=945,150,205.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 SectionTab at=1198.5,150,100.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Hidden
                    28 TextBlock at=1198.5,162,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1198.5,150,100.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,240,1036.5,636 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,240,1036.5,636 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    33 LunaTable at=96,240,1036.5,636 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,636 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,636
                35 Border at=96,876,1036.5,177 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  36 Grid at=96,903,1036.5,150 by=panel limit=0,0,1920,1200 role=none
                    37 Grid at=96,903,1036.5,58.5 by=panel limit=0,0,1920,1200 role=none
                      38 ProgressBar at=96,903,1036.5,12 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                      39 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=406.3,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    40 StackPanel at=744,987,388.5,66 by=panel limit=0,0,1920,1200 role=none
                      41 Button at=744,987,115.5,66 by=panel drawn focus heard=STOP limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                        42 TextBlock at=769.5,996,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=744,987,115.5,66 owner=41 paints=#D4D4D4FF role=text size=26 text=STOP
                      43 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        44 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          45 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          46 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=43 paints=#707070FF role=text size=26 text="Apply…"
              47 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                48 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  49 TextBlock at=1192.5,276,595.5,741 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=380.09,184.8 limit=0,0,1920,1200 name=LibraryOverview paints=#D4D4D4FF role=text size=22 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            50 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              51 HintBar at=96,1074,1728,45 by=panel heard="Back, Finding, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 17
edge 12 up same
edge 12 down 41
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 41
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 41
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 41
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 41
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 41
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 41
edge 27 left 25
edge 27 right 12
edge 41 up 23
edge 41 down same
edge 41 left 21
edge 41 right 12
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back top=none
