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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=none seen=350.48,187,1219.05,822
    1 CoresPane at=386,205,1149,804 by=panel heard=Cores limit=350.48,187,1219.05,822 role=pane
      2 Grid at=386,205,1149,804 by=panel limit=350.48,187,1219.05,822 role=none
        3 DockPanel at=410,223,1101,69 by=panel limit=350.48,187,1219.05,822 role=none
          4 Border at=1241,223,270,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveSearch role=none
            5 TextBox at=1241,223,270,69 by=panel drawn focus heard=Search limit=350.48,187,1219.05,822 op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,233.5,237,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,233.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=115.18,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search cores"
          8 Border at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 name=AdaptiveNavigation role=none
            9 WholeRow at=410,223,831,69 by=panel limit=350.48,187,1219.05,822 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,224.5,99,66 by=panel drawn focus heard=All limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,233.5,48,48 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=410,224.5,99,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=521,224.5,223.5,66 by=panel drawn focus heard=PlayStation limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=546.5,233.5,172.5,48 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=521,224.5,223.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=756.5,224.5,246,66 by=panel drawn focus heard="PlayStation 2" limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=782,233.5,195,48 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=129.97,31.2 limit=756.5,224.5,246,66 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=1014.5,224.5,120,66 by=panel drawn focus heard=SNES limit=350.48,187,1219.05,822 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=1040,233.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=1014.5,224.5,120,66 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
          19 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveContent role=none
            20 Grid at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              21 Border at=425,310,1071,340.5 by=panel limit=350.48,187,1219.05,822 paints=#1B1B1EFF,#55555AFF role=none
                22 DockPanel at=450.5,335.5,1020,289.5 by=panel limit=350.48,187,1219.05,822 role=none
                  23 TextBlock at=450.5,335.5,1020,43.5 box=680,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=175.61,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="Apply these changes?"
                  24 StackPanel at=1170.5,559,300,66 by=panel limit=350.48,187,1219.05,822 role=none
                    25 Button at=1170.5,559,154.5,66 by=panel drawn focus heard=Cancel limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
                      26 TextBlock at=1196,568,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1170.5,559,154.5,66 owner=25 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    27 Button at=1337,559,133.5,66 by=panel drawn focus heard=Apply limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Apply
                      28 TextBlock at=1362.5,568,82.5,48 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=54.29,31.2 limit=1337,559,133.5,66 owner=27 paints=#9C9CA0FF role=text size=26 text=APPLY
                  29 ScrollViewer at=450.5,397,1020,144 by=panel fades=b limit=350.48,187,1219.05,822 overflow=159 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=450.5,397,1020,144
                    30 TextBlock at=450.5,397,1020,303 box=680,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=29 layout=675.14,201.6 limit=450.5,397,1020,144 paints=#9C9CA0FF role=text seen=450.5,397,1020,144 size=24 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
        31 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          32 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            33 ActionButton at=410,662.5,157.5,66 by=panel drawn focus heard=INSTALL limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Install
              34 TextBlock at=435.5,671.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=70.23,31.2 limit=410,662.5,157.5,66 owner=33 paints=#9C9CA0FF role=text size=26 text=INSTALL
            35 ActionButton at=585.5,662.5,151.5,66 by=panel drawn focus heard=UPDATE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Update
              36 TextBlock at=611,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=585.5,662.5,151.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text=UPDATE
            37 ActionButton at=755,662.5,157.5,66 by=panel drawn focus heard=REMOVE limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Remove
              38 TextBlock at=780.5,671.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=755,662.5,157.5,66 owner=37 paints=#9C9CA0FF role=text size=26 text=REMOVE
            39 ActionButton at=930.5,662.5,195,66 by=panel drawn heard="APPROVE…" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Approve…"
              40 TextBlock at=956,671.5,144,48 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=95.76,31.2 limit=930.5,662.5,195,66 off owner=39 paints=#55555AFF role=text size=26 text="APPROVE…"
            41 ActionButton at=1143.5,662.5,243,66 by=panel drawn heard="MAKE DEFAULT" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Make Default"
              42 TextBlock at=1169,671.5,192,48 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=127.82,31.2 limit=1143.5,662.5,243,66 off owner=41 paints=#55555AFF role=text size=26 text="MAKE DEFAULT"
            43 ActionButton at=410,740.5,189,66 by=panel drawn heard="OPTIONS…" limit=350.48,187,1219.05,822 off paints=#55555AFF,#9C9CA0FF role=button text="Options…"
              44 TextBlock at=435.5,749.5,138,48 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=91.68,31.2 limit=410,740.5,189,66 off owner=43 paints=#55555AFF role=text size=26 text="OPTIONS…"
            45 ActionButton at=617,740.5,144,66 by=panel drawn focus heard=VERIFY limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Verify
              46 TextBlock at=642.5,749.5,93,48 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=61.2,31.2 limit=617,740.5,144,66 owner=45 paints=#9C9CA0FF role=text size=26 text=VERIFY
            47 ActionButton at=779,740.5,321,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Check for Updates"
              48 TextBlock at=804.5,749.5,270,48 box=180,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=179.58,31.2 limit=779,740.5,321,66 owner=47 paints=#9C9CA0FF role=text size=26 text="CHECK FOR UPDATES"
            49 ActionButton at=1118,740.5,123,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              50 TextBlock at=1143.5,749.5,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=1118,740.5,123,66 owner=49 paints=#9C9CA0FF role=text size=26 text=MARK
        51 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          52 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            53 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              54 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: install 1, update 1, about 8 MB to download" layout=425.21,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="2 marked: install 1, update 1, about 8 MB to download"
            55 StackPanel at=737,880,792,66 by=panel limit=350.48,187,1219.05,822 role=none
              56 Button at=737,880,157.5,66 by=panel drawn focus heard="Apply the marked changes" limit=350.48,187,1219.05,822 op paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                57 TextBlock at=762.5,889,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=737,880,157.5,66 owner=56 paints=#9C9CA0FF role=text size=26 text="APPLY…"
              58 StackPanel at=912.5,880,616.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                59 ProgressBar at=912.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                60 TextBlock at=1224.5,892,304.5,43.5 box=203,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=202.22,28.8 limit=350.48,187,1219.05,822 paints=#9C9CA0FF role=text size=24 text="/Cores 11 MB · 410 GB free"
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
edge 25 down 49
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
edge 49 up 25
edge 49 down 56
edge 49 left 47
edge 49 right 27
edge 56 up 47
edge 56 down same
edge 56 left 33
edge 56 right 49
press pageup focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press pagedown focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press first focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press last focus="Button 'Cancel' (Cancel)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
