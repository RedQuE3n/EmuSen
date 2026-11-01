# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state LibraryPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Library Manager"
case LibraryPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 LibraryPane at=386,205,1149,804 by=panel heard="Library Manager" limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
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
          17 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            18 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              19 Border at=422,310,1075.5,340.5 by=panel limit=350.48,187,1219.05,822 paints=#1B1B1EFF,#707070FF role=none
                20 DockPanel at=447.5,335.5,1024.5,289.5 by=panel limit=350.48,187,1219.05,822 role=none
                  21 TextBlock at=447.5,335.5,1024.5,43.5 box=683,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  22 StackPanel at=1184,559,288,66 by=panel limit=350.48,187,1219.05,822 role=none
                    23 Button at=1184,559,148.5,66 by=panel drawn focus heard=Cancel limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      24 TextBlock at=1209.5,568,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1184,559,148.5,66 owner=23 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    25 Button at=1344.5,559,127.5,66 by=panel drawn focus heard=Apply limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      26 TextBlock at=1370,568,76.5,48 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=1344.5,559,127.5,66 owner=25 paints=#D4D4D4FF role=text size=26 text=APPLY
                  27 ScrollViewer at=447.5,397,1024.5,144 by=panel fades=b limit=350.48,187,1219.05,822 overflow=202.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=447.5,397,1024.5,144
                    28 TextBlock at=447.5,397,1024.5,346.5 box=683,231 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"." in=27 layout=677.52,230.4 limit=447.5,397,1024.5,144 paints=#D4D4D4FF role=text seen=447.5,397,1024.5,144 size=24 text="Hide from the library (2):\n  Amber Saga (USA) [copier].smc\n  Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue\n\nOnly games.db changes: no file in the library is written, moved or removed. A hidden game is shown again from here, or by Preferences' \"List the games hidden from the library\"."
        29 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          30 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            31 ActionButton at=410,662.5,109.5,66 by=panel drawn focus heard=HIDE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Hide
              32 TextBlock at=435.5,671.5,58.5,48 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=410,662.5,109.5,66 owner=31 paints=#D4D4D4FF role=text size=26 text=HIDE
            33 ActionButton at=537.5,662.5,345,66 by=panel drawn focus heard="SHOW IN FILE MANAGER" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Show in File Manager"
              34 TextBlock at=563,671.5,294,48 box=196,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SHOW IN FILE MANAGER" layout=195.86,31.2 limit=537.5,662.5,345,66 owner=33 paints=#D4D4D4FF role=text size=26 text="SHOW IN FILE MANAGER"
            35 ActionButton at=900.5,662.5,198,66 by=panel drawn focus heard="SAVE LIST…" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Save List…"
              36 TextBlock at=926,671.5,147,48 box=98,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SAVE LIST…" layout=97.11,31.2 limit=900.5,662.5,198,66 owner=35 paints=#D4D4D4FF role=text size=26 text="SAVE LIST…"
            37 ActionButton at=1116.5,662.5,216,66 by=panel drawn focus heard="CHECK AGAIN" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Check Again"
              38 TextBlock at=1142,671.5,165,48 box=110,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK AGAIN" layout=109.38,31.2 limit=1116.5,662.5,216,66 owner=37 paints=#D4D4D4FF role=text size=26 text="CHECK AGAIN"
            39 ActionButton at=1350.5,662.5,115.5,66 by=panel drawn heard=STOP limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Stop
              40 TextBlock at=1376,671.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=STOP layout=42.17,31.2 limit=1350.5,662.5,115.5,66 off owner=39 paints=#707070FF role=text size=26 text=STOP
            41 ActionButton at=410,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              42 TextBlock at=435.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=410,740.5,120,66 owner=41 paints=#D4D4D4FF role=text size=26 text=MARK
        43 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          44 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            45 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              46 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 to hide" layout=67.18,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="2 to hide"
            47 StackPanel at=1376,880,153,66 by=panel limit=350.48,187,1219.05,822 role=none
              48 Button at=1376,880,153,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                49 TextBlock at=1401.5,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1376,880,153,66 owner=48 paints=#D4D4D4FF role=text size=26 text="APPLY…"
        50 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Check Again, Mark" limit=350.48,187,1219.05,822 role=group
focus 23
edge 5 up same
edge 5 down 23
edge 5 left 14
edge 5 right same
edge 10 up same
edge 10 down 31
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 33
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 33
edge 14 left 12
edge 14 right 5
edge 23 up 5
edge 23 down 37
edge 23 left 35
edge 23 right 25
edge 25 up 5
edge 25 down 48
edge 25 left 23
edge 25 right same
edge 31 up 10
edge 31 down 41
edge 31 left same
edge 31 right 33
edge 33 up 12
edge 33 down 41
edge 33 left 31
edge 33 right 35
edge 35 up 23
edge 35 down 41
edge 35 left 33
edge 35 right 37
edge 37 up 23
edge 37 down 41
edge 37 left 35
edge 37 right 25
edge 41 up 31
edge 41 down 48
edge 41 left same
edge 41 right 33
edge 48 up 25
edge 48 down same
edge 48 left 37
edge 48 right same
press pageup focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press pagedown focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press first focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press last focus="Button 'Cancel' (Cancel)" title="Library Manager" top=LibraryWindow
press back focus="GridSplitter  (Resize Mark)" title="Library Manager" top=LibraryWindow
