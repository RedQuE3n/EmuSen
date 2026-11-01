# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Cores
case CoresPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 CoresPane at=258,136,765,536 by=panel heard=Cores limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,65,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,149,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,149,141,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=364,155,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=347,149,141,44 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=496,149,156,44 by=panel drawn focus heard="PlayStation 2" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=513,155,122,32 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=496,149,156,44 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=660,149,77,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=677,155,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=660,149,77,44 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
          19 Border at=274,206,733,227 by=panel limit=234,124,812.35,548 name=AdaptiveContent role=none
            20 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
              21 Border at=284,206,714,227 by=panel limit=234,124,812.35,548 paints=#1B1B1EFF,#707070FF role=none
                22 DockPanel at=301,223,680,193 by=panel limit=234,124,812.35,548 role=none
                  23 TextBlock at=301,223,680,29 box=680,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Apply these changes?" layout=169.13,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Apply these changes?"
                  24 StackPanel at=789,372,192,44 by=panel limit=234,124,812.35,548 role=none
                    25 Button at=789,372,99,44 by=panel drawn focus heard=Cancel limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Cancel
                      26 TextBlock at=806,378,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=789,372,99,44 owner=25 paints=#EEEEF0FF role=text size=26 text=CANCEL
                    27 Button at=896,372,85,44 by=panel drawn focus heard=Apply limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Apply
                      28 TextBlock at=913,378,51,32 box=51,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=APPLY layout=50.99,31.2 limit=896,372,85,44 owner=27 paints=#D4D4D4FF role=text size=26 text=APPLY
                  29 ScrollViewer at=301,264,680,96 by=panel fades=b limit=234,124,812.35,548 overflow=106 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=301,264,680,96
                    30 TextBlock at=301,264,680,202 box=680,202 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it." in=29 layout=674.81,201.6 limit=301,264,680,96 paints=#D4D4D4FF role=text seen=301,264,680,96 size=24 text="Download and install 1 core from buildbot.libretro.com: Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name).\n\nUpdate Sony - PlayStation (Beetle PSX); the build installed now is kept for Roll Back.\n\nEach core is checked before it is kept, and nothing downloaded runs until you approve it."
        31 Border at=274,441,733,96 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          32 WrapPanel at=274,441,733,96 by=panel limit=234,124,812.35,548 role=none
            33 ActionButton at=274,441,101,44 by=panel drawn focus heard=INSTALL limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Install
              34 TextBlock at=291,447,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=274,441,101,44 owner=33 paints=#D4D4D4FF role=text size=26 text=INSTALL
            35 ActionButton at=387,441,98,44 by=panel drawn focus heard=UPDATE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Update
              36 TextBlock at=404,447,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=387,441,98,44 owner=35 paints=#D4D4D4FF role=text size=26 text=UPDATE
            37 ActionButton at=497,441,101,44 by=panel drawn focus heard=REMOVE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Remove
              38 TextBlock at=514,447,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=497,441,101,44 owner=37 paints=#D4D4D4FF role=text size=26 text=REMOVE
            39 ActionButton at=610,441,127,44 by=panel drawn heard="APPROVE…" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
              40 TextBlock at=627,447,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=610,441,127,44 off owner=39 paints=#707070FF role=text size=26 text="APPROVE…"
            41 ActionButton at=749,441,155,44 by=panel drawn heard="MAKE DEFAULT" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
              42 TextBlock at=766,447,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=749,441,155,44 off owner=41 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
            43 ActionButton at=274,493,121,44 by=panel drawn heard="OPTIONS…" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
              44 TextBlock at=291,499,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=274,493,121,44 off owner=43 paints=#707070FF role=text size=26 text="OPTIONS…"
            45 ActionButton at=407,493,92,44 by=panel drawn focus heard=VERIFY limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Verify
              46 TextBlock at=424,499,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=407,493,92,44 owner=45 paints=#D4D4D4FF role=text size=26 text=VERIFY
            47 ActionButton at=511,493,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
              48 TextBlock at=528,499,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=511,493,205,44 owner=47 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
            49 ActionButton at=728,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              50 TextBlock at=745,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=728,493,80,44 owner=49 paints=#D4D4D4FF role=text size=26 text=MARK
        51 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          52 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            53 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              54 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: install 1, update 1, about 8 MB to download" layout=409.13,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="2 marked: install 1, update 1, about 8 MB to download"
            55 StackPanel at=500,586,519,44 by=panel limit=234,124,812.35,548 role=none
              56 Button at=500,586,102,44 by=panel drawn focus heard="Apply the marked changes" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                57 TextBlock at=517,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=500,586,102,44 owner=56 paints=#D4D4D4FF role=text size=26 text="APPLY…"
              58 StackPanel at=614,586,405,44 by=panel limit=234,124,812.35,548 role=none
                59 ProgressBar at=614,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                60 TextBlock at=822,594,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        61 HintBar at=274,642,733,22 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=234,124,812.35,548 role=group
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
