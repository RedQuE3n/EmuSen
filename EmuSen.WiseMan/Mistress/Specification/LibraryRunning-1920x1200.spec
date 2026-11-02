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
      2 LibraryPane at=72,42,1776,1146 by=panel heard="Library Manager" limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard="LIBRARY MANAGER" limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1273.5,72,550.5,36 box=367,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="29 files · 0 same dumps · 0 broken or suspect · 0 hidden" layout=366.62,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="29 files · 0 same dumps · 0 broken or suspect · 0 hidden"
                8 StackPanel at=96,60,1159.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1159.5,58.5 box=773,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="LIBRARY MANAGER" layout=188.99,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="LIBRARY MANAGER"
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=13 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,138,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,138,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,138,202.5,66 by=panel drawn focus heard="Same Dump" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Dump"
                    20 TextBlock at=231,150,154.5,36 box=103,24 by=clip caps=13.999 drawn face=Barlow heard="SAME DUMP" layout=102.88,24 limit=207,138,202.5,66 owner=19 paints=#D4D4D4FF role=text size=20 text="SAME DUMP"
                  21 ToggleButton at=421.5,138,199.5,66 by=panel drawn focus heard="Same Game" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Same Game"
                    22 TextBlock at=445.5,150,151.5,36 box=101,24 by=clip caps=13.999 drawn face=Barlow heard="SAME GAME" layout=100.84,24 limit=421.5,138,199.5,66 owner=21 paints=#D4D4D4FF role=text size=20 text="SAME GAME"
                  23 ToggleButton at=633,138,324,66 by=panel drawn focus heard="Broken or Suspect" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Broken or Suspect"
                    24 TextBlock at=657,150,276,36 box=184,24 by=clip caps=13.999 drawn face=Barlow heard="BROKEN OR SUSPECT" layout=183.44,24 limit=633,138,324,66 owner=23 paints=#D4D4D4FF role=text size=20 text="BROKEN OR SUSPECT"
                  25 ToggleButton at=969,138,253.5,66 by=panel drawn focus heard=Unrecognised limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Unrecognised
                    26 TextBlock at=993,150,205.5,36 box=137,24 by=clip caps=13.999 drawn face=Barlow heard=UNRECOGNISED layout=136.92,24 limit=969,138,253.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=UNRECOGNISED
                  27 ToggleButton at=1234.5,138,148.5,66 by=panel drawn focus heard=Hidden limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=Hidden
                    28 TextBlock at=1258.5,150,100.5,36 box=67,24 by=clip caps=13.999 drawn face=Barlow heard=HIDDEN layout=66.66,24 limit=1234.5,138,148.5,66 owner=27 paints=#D4D4D4FF role=text size=20 text=HIDDEN
            29 Grid at=96,223.5,1728,814.5 by=panel limit=0,0,1920,1200 role=none
              30 Grid at=96,223.5,1036.5,814.5 by=panel limit=0,0,1920,1200 role=none
                31 Border at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                  32 Grid at=96,223.5,1036.5,747 by=panel limit=0,0,1920,1200 role=none
                    33 LunaTable at=96,223.5,1036.5,747 by=panel heard=Rows limit=0,0,1920,1200 paints=#0E0E10FF,#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                      34 ScrollViewer allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=96,240,1036.5,730.5 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,240,1036.5,730.5
                35 Border at=96,970.5,1036.5,67.5 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
                  36 Grid at=102,976.5,1024.5,55.5 by=panel limit=0,0,1920,1200 role=none
                    37 Grid at=102,976.5,1024.5,43.5 by=panel limit=0,0,1920,1200 role=none
                      38 ProgressBar at=102,994.5,180,6 by=panel drawn heard=Progress limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                      39 TextBlock at=294,976.5,832.5,43.5 box=555,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=343.82,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="Checking each file: 2 of 4, Amber Saga (USA)"
              40 Border at=1156.5,223.5,667.5,814.5 by=panel limit=0,0,1920,1200 name=AdaptiveDetail paints=#26262BFF role=none
                41 Border at=1192.5,259.5,595.5,742.5 by=panel limit=0,0,1920,1200 role=none
                  42 TextBlock at=1204.5,271.5,571.5,718.5 box=381,479 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db." layout=380.86,172.8 limit=0,0,1920,1200 name=LibraryOverview paints=#D4D4D4FF role=text size=24 text="The library's files have not all been read. Findings: same dump 0, same game 0, broken or suspect 0, unrecognised 0. Header checksums are not checked yet. Nothing in the library is written, moved or removed; hiding a game changes only games.db."
            43 Border at=96,1050,1728,66 by=panel limit=0,0,1920,1200 name=AdaptiveActions role=none
              44 WrapPanel at=96,1050,1728,66 by=panel limit=0,0,1920,1200 role=none
                45 ActionButton at=96,1050,115.5,66 by=panel drawn focus heard=STOP limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Stop
                  46 TextBlock at=121.5,1059,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=96,1050,115.5,66 owner=45 paints=#D4D4D4FF role=text size=26 text=STOP
            47 HintBar at=96,1140,1728,36 by=panel heard="Back, Finding, List, Apply…, More" limit=0,0,1920,1200 role=group
focus 12
edge 12 up same
edge 12 down 45
edge 12 left 27
edge 12 right same
edge 17 up same
edge 17 down 45
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 45
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 45
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 45
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 45
edge 25 left 23
edge 25 right 27
edge 27 up same
edge 27 down 45
edge 27 left 25
edge 27 right 12
edge 45 up 17
edge 45 down same
edge 45 left same
edge 45 right 25
press pageup focus="TextBox Search files (Search)" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" top=LibraryWindow
press first focus="TextBox Search files (Search)" top=LibraryWindow
press last focus="TextBox Search files (Search)" top=LibraryWindow
press back top=none
