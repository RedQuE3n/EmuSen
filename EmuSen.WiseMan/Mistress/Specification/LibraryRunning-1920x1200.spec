# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,217.5,1219.05,726
title "Library Manager"
case LibraryRunning
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,368,1221,546 by=panel limit=350.48,368,1219.05,546 paints=#9C9CA0FF role=none seen=350.48,368,1219.05,546
    1 LibraryPane at=386,386,1149,528 by=panel heard="Library Manager" limit=350.48,368,1219.05,546 role=pane
      2 Grid at=386,386,1149,528 by=panel limit=350.48,368,1219.05,546 role=none
        3 DockPanel at=410,404,1101,69 by=panel limit=350.48,368,1219.05,546 role=none
          4 Border at=1241,404,270,69 by=panel limit=350.48,368,1219.05,546 name=AdaptiveSearch role=none
            5 TextBox at=1241,404,270,69 by=panel drawn focus heard=Search limit=350.48,368,1219.05,546 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,414.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,414.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=6 layout=103.74,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search files"
          8 Border at=410,404,831,69 by=panel limit=350.48,368,1219.05,546 name=AdaptiveNavigation role=none
            9 WholeRow at=410,404,831,69 by=panel limit=350.48,368,1219.05,546 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,405.5,99,66 by=panel drawn focus heard=All limit=350.48,368,1219.05,546 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,414.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=410,405.5,99,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=521,405.5,202.5,66 by=panel drawn focus heard="Same Dump" limit=350.48,368,1219.05,546 op paints=#FFFFFFFF role=button text="Same Dump"
                13 TextBlock at=546.5,414.5,151.5,48 box=101,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=100.88,31.2 limit=521,405.5,202.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
              14 ToggleButton at=735.5,405.5,199.5,66 by=panel drawn focus heard="Same Game" limit=350.48,368,1219.05,546 op paints=#FFFFFFFF role=button text="Same Game"
                15 TextBlock at=761,414.5,148.5,48 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=98.36,31.2 limit=735.5,405.5,199.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
        16 Grid at=410,491,1101,64.5 by=panel limit=350.48,368,1219.05,546 role=none
          17 Border at=410,491,1101,64.5 by=panel limit=350.48,368,1219.05,546 name=AdaptiveContent role=none
            18 Grid at=410,491,1101,64.5 by=panel limit=350.48,368,1219.05,546 role=none
              19 LunaTable at=410,491,1101,64.5 by=panel heard=Rows limit=350.48,368,1219.05,546 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
                20 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=428,500,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=39.14,28.8 paints=#9C9CA0FF role=text size=24 text=Mark
                21 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=479,497,9,48 focus heard="Resize Mark" paints=#9C9CA0FF role=thumb
                22 Button allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=488,497,502.5,48 drawn focus heard="Game, not sorted" op paints=#55555AFF,#9C9CA0FF role=button
                  23 StackPanel allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=488,497,75,48 role=none
                    24 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=488,497,75,48 box=50,32 caps=18.1987 drawn face="Barlow Condensed" heard=Game layout=49.89,31.2 owner=22 paints=#9C9CA0FF role=text size=26 text=Game
                25 GridSplitter allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=981.5,497,9,48 focus heard="Resize Game" paints=#9C9CA0FF role=thumb
                26 TextBlock allow="a file's name or finding cut in the Library Manager's table is whole in the detail while its row is chosen" at=990.5,500,502.5,43.5 box=335,29 caps=16.7988 drawn face="Barlow Condensed" heard=Finding layout=60.1,28.8 paints=#9C9CA0FF role=text size=24 text=Finding
        27 Border at=410,567.5,1101,144 by=panel limit=350.48,368,1219.05,546 name=AdaptiveActions role=none
          28 WrapPanel at=410,567.5,1101,144 by=panel limit=350.48,368,1219.05,546 role=none
            29 ActionButton at=410,567.5,112.5,66 by=panel drawn focus heard=HIDE limit=350.48,368,1219.05,546 op paints=#55555AFF,#9C9CA0FF role=button text=Hide
              30 TextBlock at=435.5,576.5,61.5,48 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=40.92,31.2 limit=410,567.5,112.5,66 owner=29 paints=#9C9CA0FF role=text size=26 text=HIDE
            31 ActionButton at=540.5,567.5,358.5,66 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=350.48,368,1219.05,546 op paints=#55555AFF,#9C9CA0FF role=button text="Show in File Manager"
              32 TextBlock at=566,576.5,307.5,48 box=205,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=204.41,31.2 limit=540.5,567.5,358.5,66 owner=31 paints=#9C9CA0FF role=text size=26 text="SHOW IN FILE MANAGER"
            33 ActionButton at=917,567.5,204,66 by=panel drawn focus heard="SAVE LIST…" limit=350.48,368,1219.05,546 op paints=#55555AFF,#9C9CA0FF role=button text="Save List…"
              34 TextBlock at=942.5,576.5,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=101.37,31.2 limit=917,567.5,204,66 owner=33 paints=#9C9CA0FF role=text size=26 text="SAVE LIST…"
            35 ActionButton at=1139,567.5,223.5,66 by=panel drawn heard="CHECK AGAIN" limit=350.48,368,1219.05,546 off paints=#55555AFF,#9C9CA0FF role=button text="Check Again"
              36 TextBlock at=1164.5,576.5,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=114.45,31.2 limit=1139,567.5,223.5,66 off owner=35 paints=#55555AFF role=text size=26 text="CHECK AGAIN"
            37 ActionButton at=1380.5,567.5,117,66 by=panel drawn focus heard=STOP limit=350.48,368,1219.05,546 op paints=#55555AFF,#9C9CA0FF role=button text=Stop
              38 TextBlock at=1406,576.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=1380.5,567.5,117,66 owner=37 paints=#9C9CA0FF role=text size=26 text=STOP
            39 ActionButton at=410,645.5,123,66 by=panel drawn focus heard=MARK limit=350.48,368,1219.05,546 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              40 TextBlock at=435.5,654.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=410,645.5,123,66 owner=39 paints=#9C9CA0FF role=text size=26 text=MARK
        41 Border at=386,723.5,1149,133.5 by=panel limit=350.48,368,1219.05,546 name=AdaptiveStatus role=none
          42 Grid at=392,729.5,1137,121.5 by=panel limit=350.48,368,1219.05,546 role=none
            43 Grid at=392,729.5,1137,43.5 by=panel limit=350.48,368,1219.05,546 role=none
              44 ProgressBar at=392,747.5,180,6 by=panel drawn heard=Progress limit=350.48,368,1219.05,546 paints=#333337FF,#4C9AE8FF role=progressbar
              45 TextBlock at=584,729.5,945,43.5 box=630,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Checking each file: 2 of 4, Amber Saga (USA)" layout=358.1,28.8 limit=350.48,368,1219.05,546 paints=#9C9CA0FF role=text size=24 text="Checking each file: 2 of 4, Amber Saga (USA)"
            46 StackPanel at=1371.5,785,157.5,66 by=panel limit=350.48,368,1219.05,546 role=none
              47 Button at=1371.5,785,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,368,1219.05,546 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                48 TextBlock at=1397,794,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=1371.5,785,157.5,66 off owner=47 paints=#55555AFF role=text size=26 text="APPLY…"
        49 HintBar at=410,869,1101,33 by=panel heard="Open, Back, Section, Apply…, Check Again, Mark" limit=350.48,368,1219.05,546 role=group
focus 5
edge 5 up same
edge 5 down 37
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 21
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 22
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 22
edge 14 left 12
edge 14 right 5
edge 21 up 10
edge 21 down 29
edge 21 left 10
edge 21 right 22
edge 22 up 12
edge 22 down 29
edge 22 left 21
edge 22 right 5
edge 29 up 21
edge 29 down 39
edge 29 left same
edge 29 right 31
edge 31 up 22
edge 31 down 39
edge 31 left 29
edge 31 right 33
edge 33 up 22
edge 33 down 39
edge 33 left 31
edge 33 right 37
edge 37 up 5
edge 37 down 39
edge 37 left 33
edge 37 right same
edge 39 up 29
edge 39 down same
edge 39 left same
edge 39 right 31
press pageup focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press first focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press last focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
