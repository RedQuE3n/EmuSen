# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Cores
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 CoresPane at=386,205,1149,804 by=panel heard=Cores limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
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
              21 Border at=425,310,1071,340.5 by=panel limit=350.48,187,1219.05,822 paints=#1B1B1EFF,#707070FF role=none
                22 DockPanel at=450.5,335.5,1020,289.5 by=panel limit=350.48,187,1219.05,822 role=none
                  23 TextBlock at=450.5,335.5,1020,43.5 box=680,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  24 StackPanel at=1182.5,559,288,66 by=panel limit=350.48,187,1219.05,822 role=none
                    25 Button at=1182.5,559,148.5,66 by=panel drawn focus heard=Cancel limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      26 TextBlock at=1208,568,97.5,48 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=1182.5,559,148.5,66 owner=25 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    27 Button at=1343,559,127.5,66 by=panel drawn focus heard=Apply limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      28 TextBlock at=1368.5,568,76.5,48 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=1343,559,127.5,66 owner=27 paints=#D4D4D4FF role=text size=26 text=APPLY
                  29 ScrollViewer at=450.5,397,1020,144 by=panel fades=b limit=350.48,187,1219.05,822 overflow=159 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=450.5,397,1020,144
                    30 TextBlock at=450.5,397,1020,303 box=680,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=29 layout=674.81,201.6 limit=450.5,397,1020,144 paints=#D4D4D4FF role=text seen=450.5,397,1020,144 size=24 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
        31 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          32 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            33 ActionButton at=410,662.5,151.5,66 by=panel drawn focus heard=INSTALL limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Install
              34 TextBlock at=435.5,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=410,662.5,151.5,66 owner=33 paints=#D4D4D4FF role=text size=26 text=INSTALL
            35 ActionButton at=579.5,662.5,147,66 by=panel drawn focus heard=UPDATE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Update
              36 TextBlock at=605,671.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=579.5,662.5,147,66 owner=35 paints=#D4D4D4FF role=text size=26 text=UPDATE
            37 ActionButton at=744.5,662.5,151.5,66 by=panel drawn focus heard=REMOVE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Remove
              38 TextBlock at=770,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=744.5,662.5,151.5,66 owner=37 paints=#D4D4D4FF role=text size=26 text=REMOVE
            39 ActionButton at=914,662.5,190.5,66 by=panel drawn heard="APPROVE…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
              40 TextBlock at=939.5,671.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=914,662.5,190.5,66 off owner=39 paints=#707070FF role=text size=26 text="APPROVE…"
            41 ActionButton at=1122.5,662.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
              42 TextBlock at=1148,671.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=1122.5,662.5,232.5,66 off owner=41 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
            43 ActionButton at=410,740.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
              44 TextBlock at=435.5,749.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=410,740.5,181.5,66 off owner=43 paints=#707070FF role=text size=26 text="OPTIONS…"
            45 ActionButton at=609.5,740.5,138,66 by=panel drawn focus heard=VERIFY limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Verify
              46 TextBlock at=635,749.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=609.5,740.5,138,66 owner=45 paints=#D4D4D4FF role=text size=26 text=VERIFY
            47 ActionButton at=765.5,740.5,307.5,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
              48 TextBlock at=791,749.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=765.5,740.5,307.5,66 owner=47 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
            49 ActionButton at=1091,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              50 TextBlock at=1116.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1091,740.5,120,66 owner=49 paints=#D4D4D4FF role=text size=26 text=MARK
        51 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          52 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            53 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              54 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: install 1, update 1, about 8 MB to download" layout=409.13,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="2 marked: install 1, update 1, about 8 MB to download"
            55 StackPanel at=750.5,880,778.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              56 Button at=750.5,880,153,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                57 TextBlock at=776,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=750.5,880,153,66 owner=56 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              58 StackPanel at=921.5,880,607.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                59 ProgressBar at=921.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                60 TextBlock at=1233.5,892,295.5,43.5 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        61 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=350.48,187,1219.05,822 role=group
focus 25
edge 5 up same
edge 5 down 25
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 33
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 33
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 37
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 47
edge 16 left 14
edge 16 right 5
edge 25 up 5
edge 25 down 37
edge 25 left 47
edge 25 right 27
edge 27 up 5
edge 27 down 37
edge 27 left 25
edge 27 right same
edge 33 up 10
edge 33 down 45
edge 33 left same
edge 33 right 35
edge 35 up 12
edge 35 down 45
edge 35 left 33
edge 35 right 37
edge 37 up 14
edge 37 down 47
edge 37 left 35
edge 37 right 49
edge 45 up 35
edge 45 down 56
edge 45 left 33
edge 45 right 47
edge 47 up 37
edge 47 down 56
edge 47 left 45
edge 47 right 49
edge 49 up 37
edge 49 down 56
edge 49 left 47
edge 49 right 27
edge 56 up 47
edge 56 down same
edge 56 left 45
edge 56 right 49
press pageup focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press first focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press last focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
