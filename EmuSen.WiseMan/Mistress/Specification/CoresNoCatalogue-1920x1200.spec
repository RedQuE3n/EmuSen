# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresNoCatalogue-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,61,1219.05,978
title Cores
case CoresNoCatalogue
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,211,1221,798 by=panel limit=350.48,211,1219.05,798 paints=#D4D4D4FF role=none seen=350.48,211,1219.05,798
    1 CoresPane at=386,229,1149,780 by=panel heard=Cores limit=350.48,211,1219.05,798 role=pane
      2 Grid at=386,229,1149,780 by=panel limit=350.48,211,1219.05,798 role=none
        3 DockPanel at=410,247,1101,69 by=panel limit=350.48,211,1219.05,798 role=none
          4 Border at=1241,247,270,69 by=panel limit=350.48,211,1219.05,798 name=AdaptiveSearch role=none
            5 TextBox at=1241,247,270,69 by=panel drawn focus heard=Search limit=350.48,211,1219.05,798 op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=1257.5,257.5,237,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=1257.5,257.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
          8 Border at=410,247,831,69 by=panel limit=350.48,211,1219.05,798 name=AdaptiveNavigation role=none
            9 WholeRow at=410,247,831,69 by=panel limit=350.48,211,1219.05,798 paints=#FFFFFFFF role=none
              10 ToggleButton at=410,248.5,97.5,66 by=panel drawn focus heard=All limit=350.48,211,1219.05,798 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=435.5,257.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=410,248.5,97.5,66 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=519.5,248.5,211.5,66 by=panel drawn focus heard=PlayStation limit=350.48,211,1219.05,798 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=545,257.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=519.5,248.5,211.5,66 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=743,248.5,234,66 by=panel drawn focus heard="PlayStation 2" limit=350.48,211,1219.05,798 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=768.5,257.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=743,248.5,234,66 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=989,248.5,115.5,66 by=panel drawn focus heard=SNES limit=350.48,211,1219.05,798 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=1014.5,257.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=989,248.5,115.5,66 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=410,334,1101,316.5 by=panel limit=350.48,211,1219.05,798 role=none
          19 Border at=410,334,1101,316.5 by=panel limit=350.48,211,1219.05,798 name=AdaptiveContent role=none
            20 Grid at=410,334,1101,316.5 by=panel limit=350.48,211,1219.05,798 role=none
              21 LunaTable at=410,334,1101,316.5 by=panel heard=Rows limit=350.48,211,1219.05,798 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
                22 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=428,343,57,43.5 box=38,29 caps=16.7988 drawn face="Barlow Condensed" heard=Mark layout=37.13,28.8 paints=#D4D4D4FF role=text size=24 text=Mark
                23 GridSplitter at=476,340,9,48 by=clip focus heard="Resize Mark" limit=410,334,1101,64.5 paints=#D4D4D4FF role=thumb
                24 Button at=485,340,504,48 by=clip drawn focus heard="Core, not sorted" limit=410,334,1101,64.5 op paints=#707070FF,#D4D4D4FF role=button
                  25 StackPanel at=485,340,58.5,48 by=clip limit=485,340,504,48 role=none
                    26 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=485,340,58.5,48 box=39,32 caps=18.1987 drawn face="Barlow Condensed" heard=Core layout=38.3,31.2 owner=24 paints=#D4D4D4FF role=text size=26 text=Core
                27 GridSplitter at=980,340,9,48 by=clip focus heard="Resize Core" limit=410,334,1101,64.5 paints=#D4D4D4FF role=thumb
                28 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=989,343,504,43.5 box=336,29 caps=16.7988 drawn face="Barlow Condensed" heard=State layout=39.53,28.8 paints=#D4D4D4FF role=text size=24 text=State
                29 ScrollViewer at=410,398.5,1101,252 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=410,398.5,1101,252
                  30 ListBoxItem at=410,398.5,1101,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=410,398.5,1101,252 op paints=#333337FF,#D4D4D4FF role=listitem
                    31 Grid at=425,406,1071,46.5 by=clip heard="Mark: not marked, Core: Sapphire, State: built in" in=29 limit=410,398.5,1101,63 role=none
                      32 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,406,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=30 paints=#D4D4D4FF role=text size=24 text="☐"
                      33 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=482,407.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=69.38,28.8 owner=30 paints=#D4D4D4FF role=edit size=24 text=Sapphire
                      34 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=989,407.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=30 paints=#D4D4D4FF role=edit size=24 text="built in"
                  35 ListBoxItem at=410,461.5,1101,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=410,398.5,1101,252 op paints=#333337FF,#D4D4D4FF role=listitem
                    36 Grid at=425,469,1071,46.5 by=clip heard="Mark: not marked, Core: Diamond, State: built in" in=29 limit=410,461.5,1101,63 role=none
                      37 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,469,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=35 paints=#D4D4D4FF role=text size=24 text="☐"
                      38 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=482,470.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=68.5,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text=Diamond
                      39 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=989,470.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=35 paints=#D4D4D4FF role=edit size=24 text="built in"
                  40 ListBoxItem at=410,524.5,1101,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=410,398.5,1101,252 op paints=#333337FF,#D4D4D4FF role=listitem
                    41 Grid at=425,532,1071,46.5 by=clip heard="Mark: not marked, Core: VenusRT, State: built in" in=29 limit=410,524.5,1101,63 role=none
                      42 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,532,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=40 paints=#D4D4D4FF role=text size=24 text="☐"
                      43 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=482,533.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=65.62,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text=VenusRT
                      44 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=989,533.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=52.08,28.8 owner=40 paints=#D4D4D4FF role=edit size=24 text="built in"
                  45 ListBoxItem at=410,587.5,1101,63 by=fixed-scroll drawn focus heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=410,398.5,1101,252 op paints=#333337FF,#D4D4D4FF role=listitem
                    46 Grid at=425,595,1071,46.5 by=clip heard="Mark: not marked, Core: Extra PS2 Core, State: waiting for approval" in=29 limit=410,587.5,1101,63 role=none
                      47 TextBlock allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=425,595,57,46.5 box=38,31 caps=17.6426 drawn face="Adwaita Mono" heard="☐" in=29 layout=14.4,30.84 owner=45 paints=#D4D4D4FF role=text size=24 text="☐"
                      48 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=482,596.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=Core in=29 layout=113.69,28.8 owner=45 paints=#D4D4D4FF role=edit size=24 text="Extra PS2 Core"
                      49 TableCell allow="a core's name, console or state cut in the Cores table is whole in the detail while its row is chosen" at=989,596.5,507,43.5 box=338,29 caps=16.7988 drawn face="Barlow Condensed" heard=State in=29 layout=155.38,28.8 owner=45 paints=#D4D4D4FF role=edit size=24 text="waiting for approval"
        50 Border at=410,662.5,1101,144 by=panel limit=350.48,211,1219.05,798 name=AdaptiveActions role=none
          51 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,211,1219.05,798 role=none
            52 ActionButton at=410,662.5,151.5,66 by=panel drawn focus heard=INSTALL limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text=Install
              53 TextBlock at=435.5,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=410,662.5,151.5,66 owner=52 paints=#D4D4D4FF role=text size=26 text=INSTALL
            54 ActionButton at=579.5,662.5,147,66 by=panel drawn focus heard=UPDATE limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text=Update
              55 TextBlock at=605,671.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=579.5,662.5,147,66 owner=54 paints=#D4D4D4FF role=text size=26 text=UPDATE
            56 ActionButton at=744.5,662.5,151.5,66 by=panel drawn focus heard=REMOVE limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text=Remove
              57 TextBlock at=770,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=744.5,662.5,151.5,66 owner=56 paints=#D4D4D4FF role=text size=26 text=REMOVE
            58 ActionButton at=914,662.5,190.5,66 by=panel drawn heard="APPROVE…" limit=350.48,211,1219.05,798 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
              59 TextBlock at=939.5,671.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=914,662.5,190.5,66 off owner=58 paints=#707070FF role=text size=26 text="APPROVE…"
            60 ActionButton at=1122.5,662.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=350.48,211,1219.05,798 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
              61 TextBlock at=1148,671.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=1122.5,662.5,232.5,66 off owner=60 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
            62 ActionButton at=410,740.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=350.48,211,1219.05,798 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
              63 TextBlock at=435.5,749.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=410,740.5,181.5,66 off owner=62 paints=#707070FF role=text size=26 text="OPTIONS…"
            64 ActionButton at=609.5,740.5,138,66 by=panel drawn focus heard=VERIFY limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text=Verify
              65 TextBlock at=635,749.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=609.5,740.5,138,66 owner=64 paints=#D4D4D4FF role=text size=26 text=VERIFY
            66 ActionButton at=765.5,740.5,307.5,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
              67 TextBlock at=791,749.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=765.5,740.5,307.5,66 owner=66 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
            68 ActionButton at=1091,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,211,1219.05,798 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              69 TextBlock at=1116.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1091,740.5,120,66 owner=68 paints=#D4D4D4FF role=text size=26 text=MARK
        70 Border at=386,818.5,1149,133.5 by=panel limit=350.48,211,1219.05,798 name=AdaptiveStatus role=none
          71 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,211,1219.05,798 role=none
            72 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,211,1219.05,798 role=none
              73 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com." layout=697.66,28.8 limit=350.48,211,1219.05,798 paints=#D4D4D4FF role=text size=24 text="No catalogue of libretro's cores yet: Check for Updates reads it from buildbot.libretro.com."
            74 StackPanel at=771.5,880,757.5,66 by=panel limit=350.48,211,1219.05,798 role=none
              75 Button at=771.5,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,211,1219.05,798 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                76 TextBlock at=797,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=771.5,880,153,66 off owner=75 paints=#707070FF role=text size=26 text="APPLY…"
              77 StackPanel at=942.5,880,586.5,66 by=panel limit=350.48,211,1219.05,798 role=none
                78 ProgressBar at=942.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,211,1219.05,798 paints=#333337FF,#4C9AE8FF role=progressbar
                79 TextBlock at=1254.5,892,274.5,43.5 box=183,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 0 B · 410 GB free" layout=182.88,28.8 limit=350.48,211,1219.05,798 paints=#D4D4D4FF role=text size=24 text="/Cores 0 B · 410 GB free"
        80 HintBar at=410,964,1101,33 by=panel heard="Open, Back, Section, Apply…, Install, Check for Updates, Mark" limit=350.48,211,1219.05,798 role=group
focus 5
edge 5 up same
edge 5 down 30
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 23
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 24
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 24
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 30
edge 16 left 14
edge 16 right 5
edge 23 up 10
edge 23 down 30
edge 23 left 10
edge 23 right 24
edge 24 up 12
edge 24 down 30
edge 24 left 23
edge 24 right 16
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
edge 45 down 52
edge 45 left same
edge 45 right 5
edge 52 up 45
edge 52 down 62
edge 52 left same
edge 52 right 54
edge 54 up 45
edge 54 down 64
edge 54 left 52
edge 54 right 56
edge 56 up 45
edge 56 down 66
edge 56 left 54
edge 56 right 58
edge 58 up 45
edge 58 down 66
edge 58 left 56
edge 58 right 60
edge 62 up 52
edge 62 down same
edge 62 left same
edge 62 right 64
edge 64 up 54
edge 64 down same
edge 64 left 62
edge 64 right 66
edge 66 up 56
edge 66 down same
edge 66 left 64
edge 66 right 68
edge 68 up 60
edge 68 down same
edge 68 left 66
edge 68 right 5
press pageup focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press first focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press last focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
