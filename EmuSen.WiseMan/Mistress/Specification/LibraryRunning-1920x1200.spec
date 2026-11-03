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
                7 StackPanel at=1572,78,252,36 by=panel heard="0 files · not checked" limit=0,0,1920,1200 name=AdaptiveHeaderFacts role=none
                  8 TextBlock at=1572,78,76.5,36 box=51,24 by=panel caps=13.999 drawn face=Barlow heard="0 files" layout=50.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="0 files"
                  9 TextBlock at=1648.5,78,19.5,36 box=13,24 by=panel caps=13.999 drawn face=Barlow heard=" · " layout=8.2,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text=" · "
                  10 TextBlock at=1668,78,156,36 box=104,24 by=panel caps=13.999 drawn face=Barlow heard="not checked" layout=103.34,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="not checked"
                11 StackPanel at=96,66,1458,58.5 by=panel limit=0,0,1920,1200 role=none
                  12 TextBlock at=96,66,1458,58.5 box=972,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            13 DockPanel at=96,150,1728,66 by=panel limit=0,0,1920,1200 role=none
              14 Border at=1464,150,360,66 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                15 TextBox at=1464,150,360,66 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  16 ScrollViewer allow="a text box scrolls its own text" at=1480.5,160.5,327,45 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    17 TextBlock allow="a text box scrolls its own text" at=1480.5,165,327,36 box=218,24 caps=13.999 drawn face=Barlow heard="Search files" in=16 layout=98.58,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search files"
              18 Border at=96,150,1368,66 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                19 WholeRow at=96,150,1368,66 by=panel limit=0,0,1920,1200 role=tab
                  20 SectionTab at=120,150,51,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=All
                    21 TextBlock at=120,162,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=120,150,51,66 owner=20 paints=#EEEEF0FF role=text size=20 text=ALL
                  22 SectionTab at=219,150,154.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Dump"
                    23 TextBlock at=219,162,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=219,150,154.5,66 owner=22 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  24 SectionTab at=421.5,150,151.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Same Game"
                    25 TextBlock at=421.5,162,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,150,151.5,66 owner=24 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  26 SectionTab at=621,150,276,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text="Broken or Suspect"
                    27 TextBlock at=621,162,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=621,150,276,66 owner=26 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  28 SectionTab at=945,150,205.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Unrecognised
                    29 TextBlock at=945,162,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=945,150,205.5,66 owner=28 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  30 SectionTab at=1198.5,150,100.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=tabitem text=Hidden
                    31 TextBlock at=1198.5,162,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1198.5,150,100.5,66 owner=30 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            32 Grid at=96,240,1728,813 by=panel limit=0,0,1920,1200 role=none
              33 Grid at=96,240,1036.5,813 by=panel limit=0,0,1920,1200 role=none
                34 Border at=96,240,1036.5,636 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  35 Grid at=96,240,1036.5,636 by=panel limit=0,0,1920,1200 paints=#0E0E10FF role=none
                    36 LunaTable at=96,240,1036.5,636 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      37 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,636 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,636
                38 Border at=96,876,1036.5,177 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  39 Grid at=96,903,1036.5,150 by=panel limit=0,0,1920,1200 role=none
                    40 Grid at=96,903,1036.5,58.5 by=panel limit=0,0,1920,1200 role=none
                      41 ProgressBar at=96,903,1036.5,12 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                      42 TextBlock at=96,921,1036.5,40.5 box=691,27 by=panel caps=15.3989 drawn face=Barlow heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=406.3,26.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Checking each file: 2 of 4, Amber Saga (USA)"
                    43 StackPanel at=744,987,388.5,66 by=panel limit=0,0,1920,1200 role=none
                      44 Button at=744,987,115.5,66 by=panel drawn focus heard=STOP limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                        45 TextBlock at=769.5,996,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=744,987,115.5,66 owner=44 paints=#D4D4D4FF role=text size=26 text=STOP
                      46 Button at=877.5,987,255,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1200 off paints=#707070FF,#D4D4D4FF role=button
                        47 StackPanel at=903,996,204,48 by=clip limit=877.5,987,255,66 off role=none
                          48 PadGlyph at=903,1003.5,33,33 by=clip heard=Menu limit=877.5,987,255,66 off role=image
                          49 TextBlock at=951,996,124.5,48 box=83,32 by=clip caps=18.1987 drawn face=Barlow heard="Apply…" layout=82.71,31.2 limit=877.5,987,255,66 off owner=46 paints=#707070FF role=text size=26 text="Apply…"
              50 Border at=1156.5,240,667.5,813 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                51 Border at=1192.5,276,595.5,741 by=panel limit=0,0,1920,1200 role=none
                  52 TextBlock at=1192.5,276,595.5,741 box=397,494 by=panel caps=15.3989 drawn face=Barlow heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=380.09,184.8 limit=0,0,1920,1200 name=LibraryOverview paints=#D4D4D4FF role=text size=22 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            53 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=AdaptiveHelpBar paints=#707070FF role=none
              54 HintBar at=96,1074,1728,45 by=panel heard="Back, Finding, More, List, Apply…" limit=0,0,1920,1200 role=group
focus 20
edge 15 up same
edge 15 down 44
edge 15 left 30
edge 15 right same
edge 20 up same
edge 20 down 44
edge 20 left same
edge 20 right 22
edge 22 up same
edge 22 down 44
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 44
edge 24 left 22
edge 24 right 26
edge 26 up same
edge 26 down 44
edge 26 left 24
edge 26 right 28
edge 28 up same
edge 28 down 44
edge 28 left 26
edge 28 right 30
edge 30 up same
edge 30 down 44
edge 30 left 28
edge 30 right 15
edge 44 up 26
edge 44 down same
edge 44 left 24
edge 44 right 15
press pageup top=LibraryWindow
press pagedown top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back top=none
