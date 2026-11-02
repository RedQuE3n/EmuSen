# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 DockPanel at=64,40,1152,46 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,47,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=64,40,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,46 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,41,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,49,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,41,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,41,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,49,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,41,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,41,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,49,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,41,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,41,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,49,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,41,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,41,127,44 by=panel drawn focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=547,41,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,41,127,44 by=panel drawn focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,41,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,41,112,44 by=panel drawn focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,49,80,24 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=817,41,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,98,1152,468 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                28 Grid at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
                  29 Border at=283,136,714,391 by=panel limit=0,0,1280,800 paints=#1B1B1EFF,#707070FF role=none
                    30 DockPanel at=300,153,680,357 by=panel limit=0,0,1280,800 role=none
                      31 TextBlock at=300,153,680,39 box=680,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                      32 StackPanel at=300,418,680,92 by=panel limit=0,0,1280,800 role=none
                        33 Button at=300,418,680,44 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=No
                          34 TextBlock at=317,424,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=300,418,680,44 owner=33 paints=#EEEEF0FF role=text size=26 text=NO
                        35 Button at=300,466,680,44 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                          36 TextBlock at=317,472,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=300,466,680,44 owner=35 paints=#D4D4D4FF role=text size=26 text=YES
                      37 ScrollViewer at=300,204,680,202 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=300,204,680,202
                        38 TextBlock at=300,204,680,202 box=680,202 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=37 layout=674.81,201.6 limit=300,204,680,202 paints=#D4D4D4FF role=text size=24 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
            39 Border at=64,574,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              40 WrapPanel at=64,574,1152,96 by=panel limit=0,0,1280,800 role=none
                41 ActionButton at=64,574,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  42 TextBlock at=81,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,574,101,44 owner=41 paints=#D4D4D4FF role=text size=26 text=INSTALL
                43 ActionButton at=177,574,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  44 TextBlock at=194,580,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,574,98,44 owner=43 paints=#D4D4D4FF role=text size=26 text=UPDATE
                45 ActionButton at=287,574,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  46 TextBlock at=304,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,574,101,44 owner=45 paints=#D4D4D4FF role=text size=26 text=REMOVE
                47 ActionButton at=400,574,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  48 TextBlock at=417,580,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,574,127,44 off owner=47 paints=#707070FF role=text size=26 text="APPROVE…"
                49 ActionButton at=539,574,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  50 TextBlock at=556,580,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,574,155,44 off owner=49 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                51 ActionButton at=706,574,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  52 TextBlock at=723,580,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,574,121,44 off owner=51 paints=#707070FF role=text size=26 text="OPTIONS…"
                53 ActionButton at=839,574,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  54 TextBlock at=856,580,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,574,92,44 owner=53 paints=#D4D4D4FF role=text size=26 text=VERIFY
                55 ActionButton at=943,574,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  56 TextBlock at=960,580,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,574,205,44 owner=55 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                57 ActionButton at=64,626,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  58 TextBlock at=81,632,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,626,80,44 owner=57 paints=#D4D4D4FF role=text size=26 text=MARK
            59 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              60 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                61 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  62 TextBlock at=60,682,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: install 1, update 1, about 8 MB to download" layout=409.13,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="2 marked: install 1, update 1, about 8 MB to download"
                63 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                  64 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                    65 ProgressBar at=823,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    66 TextBlock at=1031,719,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            67 HintBar at=64,760,1152,24 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
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
edge 57 down same
edge 57 left same
edge 57 right 43
press pageup focus="Button 'No' (No)" top=CoresWindow
press pagedown focus="Button 'No' (No)" top=CoresWindow
press first focus="Button 'No' (No)" top=CoresWindow
press last focus="Button 'No' (No)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, State: built in)" top=CoresWindow
