# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Cores
case CoresRunning
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 CoresPane at=386,205,1149,804 by=panel heard=Cores limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,97.5,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=410,224.5,97.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=519.5,224.5,211.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=545,233.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=519.5,224.5,211.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=743,224.5,234,66 by=panel drawn focus heard="PlayStation 2" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=768.5,233.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=743,224.5,234,66 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=989,224.5,115.5,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=1014.5,233.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=989,224.5,115.5,66 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
          19 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            20 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              21 LunaTable at=410,310,1101,340.5 by=panel heard=Rows limit=350.48,187,1219.05,822 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                22 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=428,319,112.5,43.5 box=75,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                23 GridSplitter at=531.5,316,9,48 by=clip focus heard="Resize Mark" limit=410,310,1101,64.5 paints=#D4D4D4FF role=thumb
                24 Button at=540.5,316,477,48 by=clip drawn focus heard="Core, not sorted" limit=410,310,1101,64.5 op paints=#707070FF,#D4D4D4FF role=button
                  25 StackPanel at=540.5,316,58.5,48 by=clip limit=540.5,316,477,48 role=none
                    26 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=540.5,316,58.5,48 box=39,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=38.3,31.2 owner=24 paints=#D4D4D4FF role=text size=26 text=Core
                27 GridSplitter at=1008.5,316,9,48 by=clip focus heard="Resize Core" limit=410,310,1101,64.5 paints=#D4D4D4FF role=thumb
                28 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,319,475.5,43.5 box=317,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                29 ScrollViewer at=410,374.5,1101,276 fades=tb name=PART_ScrollViewer overflow=228 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=410,374.5,1101,276
                  30 ListBoxItem at=410,335.5,1101,63 by=scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=410,374.5,1101,276 op paints=#333337FF,#D4D4D4FF role=listitem seen=410,374.5,1101,24
                    31 Grid at=425,343,1071,46.5 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=410,335.5,1101,63 role=none seen=425,374.5,1071,15
                      32 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,343,112.5,46.5 box=75,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=30 paints=#D4D4D4FF role=text seen=425,374.5,112.5,15 size=24 text="☐"
                      33 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=537.5,344.5,480,43.5 box=320,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=69.38,28.8 owner=30 paints=#D4D4D4FF role=edit seen=537.5,374.5,480,13.5 size=24 text=Sapphire
                      34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,344.5,478.5,43.5 box=319,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=30 paints=#D4D4D4FF role=edit seen=1017.5,374.5,478.5,13.5 size=24 text="built in"
                  35 ListBoxItem at=410,398.5,1101,63 by=scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=410,374.5,1101,276 op paints=#333337FF,#D4D4D4FF role=listitem
                    36 Grid at=425,406,1071,46.5 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=410,398.5,1101,63 role=none
                      37 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,406,112.5,46.5 box=75,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=35 paints=#D4D4D4FF role=text size=24 text="☐"
                      38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=537.5,407.5,480,43.5 box=320,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=68.5,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text=Diamond
                      39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,407.5,478.5,43.5 box=319,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text="built in"
                  40 ListBoxItem at=410,461.5,1101,63 by=scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=410,374.5,1101,276 op paints=#333337FF,#D4D4D4FF role=listitem
                    41 Grid at=425,469,1071,46.5 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=410,461.5,1101,63 role=none
                      42 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,469,112.5,46.5 box=75,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#D4D4D4FF role=text size=24 text="☐"
                      43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=537.5,470.5,480,43.5 box=320,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=65.62,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text=VenusRT
                      44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,470.5,478.5,43.5 box=319,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text="built in"
                  45 ListBoxItem at=410,524.5,1101,63 by=scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=410,374.5,1101,276 op paints=#333337FF,#D4D4D4FF role=listitem
                    46 Grid at=425,532,1071,46.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=410,524.5,1101,63 role=none
                      47 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,532,112.5,46.5 box=75,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=45 paints=#D4D4D4FF role=text size=24 text="☐"
                      48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=537.5,533.5,480,43.5 box=320,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=113.69,28.8 owner=45 paints=#D4D4D4FF role=edit size=24 text="Extra PS2 Core"
                      49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,533.5,478.5,43.5 box=319,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=155.38,28.8 owner=45 paints=#D4D4D4FF role=edit size=24 text="waiting for approval"
                  50 ListBoxItem at=410,587.5,1101,63 by=scroll drawn focus heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=29 limit=410,374.5,1101,276 op paints=#333337FF,#D4D4D4FF role=listitem
                    51 Grid at=425,595,1071,46.5 by=clip heard="Mark: install, Core: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name), State: available" in=29 limit=410,587.5,1101,63 role=none
                      52 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,595,112.5,46.5 box=75,31 caps=17.6426 drawn face="Adwaita Mono" heard="☑ install" in=29 layout=74.52,30.84 owner=50 paints=#EEEEF0FF role=text size=24 text="☑ install"
                      53 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=537.5,596.5,480,43.5 box=320,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Core in=29 layout=318.74,28.8 owner=50 paints=#EEEEF0FF role=edit size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                      54 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=1017.5,596.5,478.5,43.5 box=319,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=68.26,28.8 owner=50 paints=#EEEEF0FF role=edit size=24 text=available
        55 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          56 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            57 ActionButton at=410,662.5,151.5,66 by=panel drawn heard=INSTALL limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Install
              58 TextBlock at=435.5,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=410,662.5,151.5,66 off owner=57 paints=#707070FF role=text size=26 text=INSTALL
            59 ActionButton at=579.5,662.5,147,66 by=panel drawn heard=UPDATE limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Update
              60 TextBlock at=605,671.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=579.5,662.5,147,66 off owner=59 paints=#707070FF role=text size=26 text=UPDATE
            61 ActionButton at=744.5,662.5,151.5,66 by=panel drawn heard=REMOVE limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Remove
              62 TextBlock at=770,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=744.5,662.5,151.5,66 off owner=61 paints=#707070FF role=text size=26 text=REMOVE
            63 ActionButton at=914,662.5,190.5,66 by=panel drawn heard="APPROVE…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
              64 TextBlock at=939.5,671.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=914,662.5,190.5,66 off owner=63 paints=#707070FF role=text size=26 text="APPROVE…"
            65 ActionButton at=1122.5,662.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
              66 TextBlock at=1148,671.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=1122.5,662.5,232.5,66 off owner=65 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
            67 ActionButton at=410,740.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
              68 TextBlock at=435.5,749.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=410,740.5,181.5,66 off owner=67 paints=#707070FF role=text size=26 text="OPTIONS…"
            69 ActionButton at=609.5,740.5,138,66 by=panel drawn heard=VERIFY limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Verify
              70 TextBlock at=635,749.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=609.5,740.5,138,66 off owner=69 paints=#707070FF role=text size=26 text=VERIFY
            71 ActionButton at=765.5,740.5,307.5,66 by=panel drawn heard="CHECK FOR UPDATES" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
              72 TextBlock at=791,749.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=765.5,740.5,307.5,66 off owner=71 paints=#707070FF role=text size=26 text="CHECK FOR UPDATES"
            73 ActionButton at=1091,740.5,120,66 by=panel drawn heard=MARK limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text=Mark
              74 TextBlock at=1116.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1091,740.5,120,66 off owner=73 paints=#707070FF role=text size=26 text=MARK
        75 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          76 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            77 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              78 ProgressBar at=392,842.5,180,6 by=panel drawn heard=Progress limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
              79 TextBlock at=584,824.5,945,43.5 box=630,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Installing pcsx2: 18 MB of 41 MB" layout=239.74,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Installing pcsx2: 18 MB of 41 MB"
            80 StackPanel at=750.5,880,778.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              81 Button at=750.5,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                82 TextBlock at=776,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=750.5,880,153,66 off owner=81 paints=#707070FF role=text size=26 text="APPLY…"
              83 StackPanel at=921.5,880,607.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                84 ProgressBar at=921.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                85 TextBlock at=1233.5,892,295.5,43.5 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        86 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=350.48,187,1219.05,822 role=group
focus 5
edge 5 up same
edge 5 down 30
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 30
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 23
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 24
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 27
edge 16 left 14
edge 16 right 5
edge 23 up 12
edge 23 down 30
edge 23 left 10
edge 23 right 24
edge 24 up 12
edge 24 down 30
edge 24 left 23
edge 24 right 5
edge 27 up 16
edge 27 down 30
edge 27 left 23
edge 27 right 30
edge 30 up 23
edge 30 down 35
edge 30 left same
edge 30 right 5
edge 35 up 30
edge 35 down 40
edge 35 left same
edge 35 right 5
edge 40 up 35
edge 40 down 45
edge 40 left same
edge 40 right 5
edge 45 up 40
edge 45 down 50
edge 45 left same
edge 45 right 5
edge 50 up 45
edge 50 down "row:ListBox/5/Mark: update, Core: Sony - PlayStation (Beetle PSX), State: update ↑"
edge 50 left same
edge 50 right 5
press pageup focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press first focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press last focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
