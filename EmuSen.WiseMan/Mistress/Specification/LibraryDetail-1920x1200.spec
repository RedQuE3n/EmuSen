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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 LibraryPane at=386,205,1149,804 by=panel heard="Library Manager" limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search files" in=6 layout=99.84,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search files"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,97.5,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=410,224.5,97.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=519.5,224.5,196.5,66 by=panel drawn focus heard="Same Dump" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Same Dump"
                13 TextBlock at=545,233.5,145.5,48 box=97,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME DUMP" layout=96.28,31.2 limit=519.5,224.5,196.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text="SAME DUMP"
              14 ToggleButton at=728,224.5,193.5,66 by=panel drawn focus heard="Same Game" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="Same Game"
                15 TextBlock at=753.5,233.5,142.5,48 box=95,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAME GAME" layout=94.48,31.2 limit=728,224.5,193.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text="SAME GAME"
        16 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
          17 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            18 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              19 ScrollViewer at=422,322,1077,316.5 by=panel fades=b limit=350.48,187,1219.05,822 overflow=172.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,316.5
                20 StackPanel at=422,322,1077,489 by=scroll in=19 limit=422,322,1077,316.5 name=LibraryDetail paints=#D4D4D4FF role=none seen=422,322,1077,316.5
                  21 TextBlock at=422,322,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)" in=19 layout=706.97,28.8 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3)"
                  22 TextBlock at=422,374.5,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue" in=19 layout=702.29,57.6 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue"
                  23 TextBlock at=422,470.5,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=19 layout=652.54,57.6 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="Broken or suspect: Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
                  24 TextBlock at=422,566.5,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="System: psx" in=19 layout=93.91,28.8 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="System: psx"
                  25 TextBlock at=422,619,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there" in=19 layout=681.17,57.6 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text seen=422,619,1077,19.5 size=24 text="Disc sheet: it lists Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (Track 2).bin, which is not there"
        26 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          27 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            28 ActionButton at=410,662.5,109.5,66 by=panel drawn focus heard=HIDE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Hide
              29 TextBlock at=435.5,671.5,58.5,48 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=410,662.5,109.5,66 owner=28 paints=#D4D4D4FF role=text size=26 text=HIDE
            30 ActionButton at=537.5,662.5,345,66 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Show in File Manager"
              31 TextBlock at=563,671.5,294,48 box=196,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=195.86,31.2 limit=537.5,662.5,345,66 owner=30 paints=#D4D4D4FF role=text size=26 text="SHOW IN FILE MANAGER"
            32 ActionButton at=900.5,662.5,198,66 by=panel drawn focus heard="SAVE LIST…" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Save List…"
              33 TextBlock at=926,671.5,147,48 box=98,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=97.11,31.2 limit=900.5,662.5,198,66 owner=32 paints=#D4D4D4FF role=text size=26 text="SAVE LIST…"
            34 ActionButton at=1116.5,662.5,216,66 by=panel drawn focus heard="CHECK AGAIN" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Check Again"
              35 TextBlock at=1142,671.5,165,48 box=110,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=109.38,31.2 limit=1116.5,662.5,216,66 owner=34 paints=#D4D4D4FF role=text size=26 text="CHECK AGAIN"
            36 ActionButton at=1350.5,662.5,115.5,66 by=panel drawn heard=STOP limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              37 TextBlock at=1376,671.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=1350.5,662.5,115.5,66 off owner=36 paints=#707070FF role=text size=26 text=STOP
            38 ActionButton at=410,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              39 TextBlock at=435.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=410,740.5,120,66 owner=38 paints=#D4D4D4FF role=text size=26 text=MARK
        40 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          41 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            42 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              43 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            44 StackPanel at=1376,880,153,66 by=panel limit=350.48,187,1219.05,822 role=none
              45 Button at=1376,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                46 TextBlock at=1401.5,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1376,880,153,66 off owner=45 paints=#707070FF role=text size=26 text="APPLY…"
        47 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Check Again, Mark" limit=350.48,187,1219.05,822 role=group
focus 5
edge 5 up same
edge 5 down 34
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 28
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 30
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 30
edge 14 left 12
edge 14 right 5
edge 28 up 10
edge 28 down 38
edge 28 left same
edge 28 right 30
edge 30 up 12
edge 30 down 38
edge 30 left 28
edge 30 right 32
edge 32 up 5
edge 32 down 38
edge 32 left 30
edge 32 right 34
edge 34 up 5
edge 34 down 38
edge 34 left 32
edge 34 right same
edge 38 up 28
edge 38 down same
edge 38 left same
edge 38 right 30
press pageup focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press pagedown focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press first focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press last focus="TextBox Search files (Search)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
