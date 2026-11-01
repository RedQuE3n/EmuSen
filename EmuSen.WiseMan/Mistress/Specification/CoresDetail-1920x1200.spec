# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title Cores
case CoresDetail
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
          19 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveDetail role=none
            20 Border at=410,310,1101,340.5 by=panel limit=350.48,187,1219.05,822 role=none
              21 ScrollViewer at=422,322,1077,316.5 by=panel fades=t limit=350.48,187,1219.05,822 overflow=256.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=422,322,1077,316.5
                22 StackPanel at=422,65.5,1077,573 by=scroll in=21 limit=422,322,1077,316.5 name=CoresDetail paints=#D4D4D4FF role=none seen=422,322,1077,316.5
                  23 TextBlock at=422,319,1077,87 box=718,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded" in=21 layout=623.16,57.6 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text seen=422,322,1077,84 size=24 text="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded"
                  24 TextBlock at=422,415,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Notes: (!) needs a BIOS · disc images by path" in=21 layout=345.1,28.8 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="Notes: (!) needs a BIOS · disc images by path"
                  25 TextBlock at=422,467.5,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Not tested by EmuSen." in=21 layout=175.22,28.8 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="Not tested by EmuSen."
                  26 TextBlock at=422,520,1077,43.5 box=718,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=21 layout=702.17,28.8 limit=422,322,1077,316.5 paints=#D4D4D4FF role=text size=24 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                  27 WrapPanel at=422,572.5,1077,66 by=scroll in=21 limit=422,322,1077,316.5 role=none
                    28 Button at=422,572.5,280.5,66 by=scroll drawn focus heard="Open Firmware" in=21 limit=422,322,1077,316.5 name=CoresFirmware op paints=#707070FF,#D4D4D4FF role=button text="Open Firmware…"
                      29 TextBlock at=447.5,581.5,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPEN FIRMWARE…" in=21 layout=152.98,31.2 limit=422,572.5,280.5,66 owner=28 paints=#EEEEF0FF role=text size=26 text="OPEN FIRMWARE…"
        30 Border at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 name=AdaptiveActions role=none
          31 WrapPanel at=410,662.5,1101,144 by=panel limit=350.48,187,1219.05,822 role=none
            32 ActionButton at=410,662.5,151.5,66 by=panel drawn focus heard=INSTALL limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Install
              33 TextBlock at=435.5,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=410,662.5,151.5,66 owner=32 paints=#D4D4D4FF role=text size=26 text=INSTALL
            34 ActionButton at=579.5,662.5,147,66 by=panel drawn focus heard=UPDATE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Update
              35 TextBlock at=605,671.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=579.5,662.5,147,66 owner=34 paints=#D4D4D4FF role=text size=26 text=UPDATE
            36 ActionButton at=744.5,662.5,151.5,66 by=panel drawn focus heard=REMOVE limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Remove
              37 TextBlock at=770,671.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=744.5,662.5,151.5,66 owner=36 paints=#D4D4D4FF role=text size=26 text=REMOVE
            38 ActionButton at=914,662.5,190.5,66 by=panel drawn heard="APPROVE…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
              39 TextBlock at=939.5,671.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=914,662.5,190.5,66 off owner=38 paints=#707070FF role=text size=26 text="APPROVE…"
            40 ActionButton at=1122.5,662.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
              41 TextBlock at=1148,671.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=1122.5,662.5,232.5,66 off owner=40 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
            42 ActionButton at=410,740.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
              43 TextBlock at=435.5,749.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=410,740.5,181.5,66 off owner=42 paints=#707070FF role=text size=26 text="OPTIONS…"
            44 ActionButton at=609.5,740.5,138,66 by=panel drawn focus heard=VERIFY limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Verify
              45 TextBlock at=635,749.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=609.5,740.5,138,66 owner=44 paints=#D4D4D4FF role=text size=26 text=VERIFY
            46 ActionButton at=765.5,740.5,307.5,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
              47 TextBlock at=791,749.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=765.5,740.5,307.5,66 owner=46 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
            48 ActionButton at=1091,740.5,120,66 by=panel drawn focus heard=MARK limit=350.48,187,1219.05,822 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              49 TextBlock at=1116.5,749.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=1091,740.5,120,66 owner=48 paints=#D4D4D4FF role=text size=26 text=MARK
        50 Border at=386,818.5,1149,133.5 by=panel limit=350.48,187,1219.05,822 name=AdaptiveStatus role=none
          51 Grid at=392,824.5,1137,121.5 by=panel limit=350.48,187,1219.05,822 role=none
            52 Grid at=392,824.5,1137,43.5 by=panel limit=350.48,187,1219.05,822 role=none
              53 TextBlock at=404,824.5,1125,43.5 box=750,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            54 StackPanel at=750.5,880,778.5,66 by=panel limit=350.48,187,1219.05,822 role=none
              55 Button at=750.5,880,153,66 by=panel drawn heard="Apply the marked changes" limit=350.48,187,1219.05,822 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                56 TextBlock at=776,889,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=750.5,880,153,66 off owner=55 paints=#707070FF role=text size=26 text="APPLY…"
              57 StackPanel at=921.5,880,607.5,66 by=panel limit=350.48,187,1219.05,822 role=none
                58 ProgressBar at=921.5,902.5,300,21 by=panel drawn heard=Space limit=350.48,187,1219.05,822 paints=#333337FF,#4C9AE8FF role=progressbar
                59 TextBlock at=1233.5,892,295.5,43.5 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        60 HintBar at=410,964,1101,33 by=panel heard="Select, Back, Section, Apply…, Install, Check for Updates, Mark" limit=350.48,187,1219.05,822 role=group
focus 28
edge 5 up same
edge 5 down 28
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 28
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 28
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 36
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 46
edge 16 left 14
edge 16 right 5
edge 28 up 10
edge 28 down 32
edge 28 left same
edge 28 right 36
edge 32 up 28
edge 32 down 44
edge 32 left same
edge 32 right 34
edge 34 up 28
edge 34 down 44
edge 34 left 32
edge 34 right 36
edge 36 up 14
edge 36 down 46
edge 36 left 34
edge 36 right 48
edge 44 up 34
edge 44 down same
edge 44 left 32
edge 44 right 46
edge 46 up 36
edge 46 down same
edge 46 left 44
edge 46 right 48
edge 48 up 36
edge 48 down same
edge 48 left 46
edge 48 right 5
press pageup title=Cores top=CoresWindow
press pagedown title=Cores top=CoresWindow
press first focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press last focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
