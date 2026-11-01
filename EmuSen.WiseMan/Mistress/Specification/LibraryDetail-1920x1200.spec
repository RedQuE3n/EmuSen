# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Library Manager"
case LibraryDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 LibraryPane at=386,205,1149,804 by=panel heard="Library Manager" limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=6 layout=103.74,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search files"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,99,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=410,224.5,99,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=521,224.5,202.5,66 by=panel drawn focus heard="Same Dump" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Same Dump"
                13 TextBlock at=546.5,233.5,151.5,48 box=101,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=100.88,31.2 limit=521,224.5,202.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
              14 ToggleButton at=735.5,224.5,199.5,66 by=panel drawn focus heard="Same Game" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Same Game"
                15 TextBlock at=761,233.5,148.5,48 box=99,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=98.36,31.2 limit=735.5,224.5,199.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
        16 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
          17 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            18 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              19 ScrollViewer at=422,322,1077,316.5 by=panel fades=b limit=350.48,187,1219.05,822 overflow=216 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,316.5
                20 StackPanel at=422,322,1077,532.5 by=scroll in=19 limit=422,322,1077,316.5 name=LibraryDetail paints=#9C9CA0FF role=none seen=422,322,1077,316.5
                  21 TextBlock at=422,322,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=19 layout=698.02,57.6 limit=422,322,1077,316.5 paints=#9C9CA0FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                  22 TextBlock at=422,418,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=19 layout=674.42,57.6 limit=422,322,1077,316.5 paints=#9C9CA0FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                  23 TextBlock at=422,514,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=19 layout=681.79,57.6 limit=422,322,1077,316.5 paints=#9C9CA0FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                  24 TextBlock at=422,610,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=19 layout=97.3,28.8 limit=422,322,1077,316.5 paints=#9C9CA0FF role=text seen=422,610,1077,28.5 size=24 text="System: psx"
        25 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          26 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            27 ActionButton at=410,662.5,112.5,66 by=panel drawn focus heard=HIDE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Hide
              28 TextBlock at=435.5,671.5,61.5,48 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=40.92,31.2 limit=410,662.5,112.5,66 owner=27 paints=#9C9CA0FF role=text size=26 text=HIDE
            29 ActionButton at=540.5,662.5,358.5,66 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Show in File Manager"
              30 TextBlock at=566,671.5,307.5,48 box=205,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=204.41,31.2 limit=540.5,662.5,358.5,66 owner=29 paints=#9C9CA0FF role=text size=26 text="SHOW IN FILE MANAGER"
            31 ActionButton at=917,662.5,204,66 by=panel drawn focus heard="SAVE LIST…" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Save List…"
              32 TextBlock at=942.5,671.5,153,48 box=102,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=101.37,31.2 limit=917,662.5,204,66 owner=31 paints=#9C9CA0FF role=text size=26 text="SAVE LIST…"
            33 ActionButton at=1139,662.5,223.5,66 by=panel drawn focus heard="CHECK AGAIN" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Check Again"
              34 TextBlock at=1164.5,671.5,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=114.45,31.2 limit=1139,662.5,223.5,66 owner=33 paints=#9C9CA0FF role=text size=26 text="CHECK AGAIN"
            35 ActionButton at=1380.5,662.5,117,66 by=panel drawn heard=STOP limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text=Stop
              36 TextBlock at=1406,671.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=43.99,31.2 limit=1380.5,662.5,117,66 off owner=35 paints=#55555AFF role=text size=26 text=STOP
            37 ActionButton at=410,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              38 TextBlock at=435.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=410,740.5,123,66 owner=37 paints=#9C9CA0FF role=text size=26 text=MARK
        39 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          40 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            41 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              42 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            43 StackPanel at=1371.5,880,157.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              44 Button at=1371.5,880,157.5,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                45 TextBlock at=1397,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=1371.5,880,157.5,66 off owner=44 paints=#55555AFF role=text size=26 text="APPLY…"
        46 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Check Again, Mark" limit=350.48,187,1219.05,822 role=group
focus 5
edge 5 up same
edge 5 down 33
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 27
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 29
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 29
edge 14 left 12
edge 14 right 5
edge 27 up 10
edge 27 down 37
edge 27 left same
edge 27 right 29
edge 29 up 12
edge 29 down 37
edge 29 left 27
edge 29 right 31
edge 31 up 5
edge 31 down 37
edge 31 left 29
edge 31 right 33
edge 33 up 5
edge 33 down 37
edge 33 left 31
edge 33 right same
edge 37 up 27
edge 37 down same
edge 37 left same
edge 37 right 29
press pageup focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press first focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press last focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
