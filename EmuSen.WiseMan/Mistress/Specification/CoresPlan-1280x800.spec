# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 ContentControl at=0,0,1280,774 by=panel limit=0,0,1280,774 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,738 by=panel heard=Cores limit=0,0,1280,774 role=pane
        3 Panel at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
          4 Grid at=48,28,1184,738 by=panel limit=0,0,1280,774 role=none
            5 DockPanel at=64,40,1152,46 by=panel limit=0,0,1280,774 role=none
              6 Border at=1036,40,180,46 by=panel limit=0,0,1280,774 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,46 by=panel drawn focus heard=Search limit=0,0,1280,774 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,47,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=64,40,972,46 by=panel limit=0,0,1280,774 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,46 by=panel limit=0,0,1280,774 paints=#FFFFFFFF role=none
                  12 ToggleButton at=64,41,65,44 by=panel drawn focus heard=All limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=All
                    13 TextBlock at=81,47,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=64,41,65,44 owner=12 paints=#EEEEF0FF role=text size=26 text=ALL
                  14 ToggleButton at=137,41,141,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=PlayStation
                    15 TextBlock at=154,47,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=137,41,141,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  16 ToggleButton at=286,41,156,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,774 op paints=#FFFFFFFF role=button text="PlayStation 2"
                    17 TextBlock at=303,47,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=286,41,156,44 owner=16 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
                  18 ToggleButton at=450,41,77,44 by=panel drawn focus heard=SNES limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=SNES
                    19 TextBlock at=467,47,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=450,41,77,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
                  20 ToggleButton at=535,41,122,44 by=panel drawn focus heard=Installed limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Installed
                    21 TextBlock at=552,47,88,32 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALLED layout=87.88,31.2 limit=535,41,122,44 owner=20 paints=#FFFFFFFF role=text size=26 text=INSTALLED
                  22 ToggleButton at=665,41,122,44 by=panel drawn focus heard=Available limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Available
                    23 TextBlock at=682,47,88,32 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=AVAILABLE layout=87.59,31.2 limit=665,41,122,44 owner=22 paints=#FFFFFFFF role=text size=26 text=AVAILABLE
                  24 ToggleButton at=795,41,108,44 by=panel drawn focus heard=Updates limit=0,0,1280,774 op paints=#FFFFFFFF role=button text=Updates
                    25 TextBlock at=812,47,74,32 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATES layout=73.87,31.2 limit=795,41,108,44 owner=24 paints=#FFFFFFFF role=text size=26 text=UPDATES
            26 Grid at=64,98,1152,429 by=panel limit=0,0,1280,774 role=none
              27 Border at=64,98,1152,429 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
                28 Grid at=64,98,1152,429 by=panel limit=0,0,1280,774 role=none
                  29 Border at=283,117,714,391 by=panel limit=0,0,1280,774 paints=#1B1B1EFF,#707070FF role=none
                    30 DockPanel at=300,134,680,357 by=panel limit=0,0,1280,774 role=none
                      31 TextBlock at=300,134,680,39 box=680,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                      32 StackPanel at=300,399,680,92 by=panel limit=0,0,1280,774 role=none
                        33 Button at=300,399,680,44 by=panel drawn focus heard=No limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=No
                          34 TextBlock at=317,405,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=300,399,680,44 owner=33 paints=#EEEEF0FF role=text size=26 text=NO
                        35 Button at=300,447,680,44 by=panel drawn focus heard=Yes limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                          36 TextBlock at=317,453,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=300,447,680,44 owner=35 paints=#D4D4D4FF role=text size=26 text=YES
                      37 ScrollViewer at=300,185,680,202 by=panel limit=0,0,1280,774 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=300,185,680,202
                        38 TextBlock at=300,185,680,202 box=680,202 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=37 layout=674.81,201.6 limit=300,185,680,202 paints=#D4D4D4FF role=text size=24 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
            39 Border at=64,535,1152,96 by=panel limit=0,0,1280,774 name=AdaptiveActions role=none
              40 WrapPanel at=64,535,1152,96 by=panel limit=0,0,1280,774 role=none
                41 ActionButton at=64,535,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  42 TextBlock at=81,541,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,535,101,44 owner=41 paints=#D4D4D4FF role=text size=26 text=INSTALL
                43 ActionButton at=177,535,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  44 TextBlock at=194,541,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,535,98,44 owner=43 paints=#D4D4D4FF role=text size=26 text=UPDATE
                45 ActionButton at=287,535,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  46 TextBlock at=304,541,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,535,101,44 owner=45 paints=#D4D4D4FF role=text size=26 text=REMOVE
                47 ActionButton at=400,535,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  48 TextBlock at=417,541,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,535,127,44 off owner=47 paints=#707070FF role=text size=26 text="APPROVE…"
                49 ActionButton at=539,535,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  50 TextBlock at=556,541,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,535,155,44 off owner=49 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                51 ActionButton at=706,535,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,774 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  52 TextBlock at=723,541,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,535,121,44 off owner=51 paints=#707070FF role=text size=26 text="OPTIONS…"
                53 ActionButton at=839,535,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  54 TextBlock at=856,541,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,535,92,44 owner=53 paints=#D4D4D4FF role=text size=26 text=VERIFY
                55 ActionButton at=943,535,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  56 TextBlock at=960,541,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,535,205,44 owner=55 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                57 ActionButton at=64,587,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  58 TextBlock at=81,593,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,587,80,44 owner=57 paints=#D4D4D4FF role=text size=26 text=MARK
            59 Border at=48,639,1184,89 by=panel limit=0,0,1280,774 name=AdaptiveStatus role=none
              60 Grid at=52,643,1176,81 by=panel limit=0,0,1280,774 role=none
                61 Grid at=52,643,1176,29 by=panel limit=0,0,1280,774 role=none
                  62 TextBlock at=60,643,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: install 1, update 1, about 8 MB to download" layout=409.13,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="2 marked: install 1, update 1, about 8 MB to download"
                63 StackPanel at=709,680,519,44 by=panel limit=0,0,1280,774 role=none
                  64 Button at=709,680,102,44 by=panel drawn focus heard="Apply the marked changes" limit=0,0,1280,774 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    65 TextBlock at=726,686,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=709,680,102,44 owner=64 paints=#D4D4D4FF role=text size=26 text="APPLY…"
                  66 StackPanel at=823,680,405,44 by=panel limit=0,0,1280,774 role=none
                    67 ProgressBar at=823,695,200,14 by=panel drawn heard=Space limit=0,0,1280,774 paints=#333337FF,#4C9AE8FF role=progressbar
                    68 TextBlock at=1031,688,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1280,774 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            69 HintBar at=64,736,1152,22 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,774 role=group
focus 33
edge 7 up same
edge 7 down 55
edge 7 left 24
edge 7 right same
edge 12 up same
edge 12 down 41
edge 12 left same
edge 12 right 14
edge 14 up same
edge 14 down 41
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 33
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 33
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 33
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 33
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 33
edge 24 left 22
edge 24 right 7
edge 33 up 16
edge 33 down 35
edge 33 left 43
edge 33 right 7
edge 35 up 33
edge 35 down 45
edge 35 left 43
edge 35 right 7
edge 41 up 12
edge 41 down 57
edge 41 left same
edge 41 right 43
edge 43 up 14
edge 43 down 57
edge 43 left 41
edge 43 right 45
edge 45 up 35
edge 45 down 57
edge 45 left 43
edge 45 right 53
edge 53 up 35
edge 53 down 57
edge 53 left 45
edge 53 right 55
edge 55 up 7
edge 55 down 57
edge 55 left 53
edge 55 right same
edge 57 up 41
edge 57 down 64
edge 57 left same
edge 57 right 43
edge 64 up 35
edge 64 down same
edge 64 left 45
edge 64 right 53
press pageup focus="Button 'No' (No)" top=CoresWindow
press pagedown focus="Button 'No' (No)" top=CoresWindow
press first focus="Button 'No' (No)" top=CoresWindow
press last focus="Button 'No' (No)" top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" top=CoresWindow
