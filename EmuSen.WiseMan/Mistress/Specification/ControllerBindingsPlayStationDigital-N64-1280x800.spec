# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsPlayStationDigital-N64-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 38.4,24,1203.2,668
title "Controller Settings"
case ControllerBindingsPlayStationDigital
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=38,124,1204,548 by=panel limit=38.4,124,1203.2,548 paints=#D4D4D4FF role=none seen=38.4,124,1203.2,548
    1 DockPanel at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
      2 Tabs at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=78,152,108,48 by=clip drawn focus heard=GENERAL limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=90,158,84,35 box=84,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GENERAL layout=83.25,34.2 limit=78,152,108,48 owner=3 paints=#8D8D91FF role=text size=28.5 text=GENERAL
        5 TabItem at=186,152,60,48 by=clip drawn focus heard=NES limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=198,158,36,35 box=36,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=NES layout=35.82,34.2 limit=186,152,60,48 owner=5 paints=#8D8D91FF role=text size=28.5 text=NES
        7 TabItem at=246,152,49,48 by=clip drawn focus heard=GB limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=258,158,25,35 box=25,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GB layout=24.48,34.2 limit=246,152,49,48 owner=7 paints=#8D8D91FF role=text size=28.5 text=GB
        9 TabItem at=295,152,72,48 by=clip drawn focus heard=SNES limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=307,158,48,35 box=48,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=SNES layout=47.05,34.2 limit=295,152,72,48 owner=9 paints=#8D8D91FF role=text size=28.5 text=SNES
        11 TabItem at=367,152,61,48 by=clip drawn focus heard=N64 limit=78,152,1124,504 op paints=#EEEEF0FF role=tabitem sideways text=N64
          12 TextBlock at=379,158,37,35 box=37,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=N64 layout=36.08,34.2 limit=367,152,61,48 owner=11 paints=#EEEEF0FF role=text size=28.5 text=N64
        13 TabItem at=428,152,100,48 by=clip drawn focus heard=SG-1000 limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=440,158,76,35 box=76,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=SG-1000 layout=75.81,34.2 limit=428,152,100,48 owner=13 paints=#8D8D91FF role=text size=28.5 text=SG-1000
        15 TabItem at=528,152,171,48 by=clip drawn focus heard="MASTER SYSTEM" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=540,158,147,35 box=147,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=146.01,34.2 limit=528,152,171,48 owner=15 paints=#8D8D91FF role=text size=28.5 text="MASTER SYSTEM"
        17 TabItem at=699,152,100,48 by=clip drawn focus heard=GENESIS limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=711,158,76,35 box=76,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GENESIS layout=75.9,34.2 limit=699,152,100,48 owner=17 paints=#8D8D91FF role=text size=28.5 text=GENESIS
        19 TabItem at=799,152,101,48 by=clip drawn focus heard="SEGA CD" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=811,158,77,35 box=77,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="SEGA CD" layout=76.41,34.2 limit=799,152,101,48 owner=19 paints=#8D8D91FF role=text size=28.5 text="SEGA CD"
        21 TabItem at=900,152,141,48 by=clip drawn focus heard=PLAYSTATION limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          22 TextBlock at=912,158,117,35 box=117,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=PLAYSTATION layout=116.54,34.2 limit=900,152,141,48 owner=21 paints=#8D8D91FF role=text size=28.5 text=PLAYSTATION
        23 TabItem at=1041,152,158,48 by=clip drawn focus heard="PLAYSTATION 2" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=1053,158,134,35 box=134,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=133.18,34.2 limit=1041,152,158,48 owner=23 paints=#8D8D91FF role=text size=28.5 text="PLAYSTATION 2"
        25 ScrollViewer at=90,202,1100,454 by=clip fades=b limit=78,202,1124,454 overflow=1152 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=90,202,1100,454
          26 StackPanel at=90,202,1100,1606 by=scroll in=25 limit=90,202,1100,454 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=90,202,1100,454
            27 DockPanel at=90,202,1100,450 by=scroll in=25 limit=90,202,1100,454 role=none
              28 ControllerDiagram at=90,202,1100,450 by=scroll caps=11.9682 face="Barlow Condensed" heard="N64 controller" in=25 limit=90,202,1100,454 name=Diagram role=group size=17.1
                29 DiagramLabel at=105,216,166,54 by=clip drawn focus heard=L in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                30 DiagramLabel at=1002,216,173,54 by=clip drawn focus heard=R in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                31 DiagramLabel at=384,584,165,54 by=clip drawn focus heard=Z in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                32 DiagramLabel at=214,288,165,54 by=clip drawn focus heard=Up in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                33 DiagramLabel at=205,410,174,54 by=clip drawn focus heard=Down in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                34 DiagramLabel at=214,349,165,54 by=clip drawn focus heard=Left in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                35 DiagramLabel at=278,216,171,54 by=clip drawn focus heard=Right in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                36 DiagramLabel at=653,216,169,54 by=clip drawn focus heard=Start in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                37 DiagramLabel at=457,216,189,54 by=clip drawn focus heard="Stick Up" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                38 DiagramLabel at=767,584,206,54 by=clip drawn focus heard="Stick Down" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                39 DiagramLabel at=181,584,196,54 by=clip drawn focus heard="Stick Left" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                40 DiagramLabel at=557,584,203,54 by=clip drawn focus heard="Stick Right" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                41 DiagramLabel at=829,216,165,54 by=clip drawn focus heard=B in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                42 DiagramLabel at=731,425,165,54 by=clip drawn focus heard=A in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                43 DiagramLabel at=731,288,168,54 by=clip drawn focus heard="C Up" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                44 DiagramLabel at=920,391,185,54 by=clip drawn focus heard="C Down" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                45 DiagramLabel at=920,322,175,54 by=clip drawn focus heard="C Left" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                46 DiagramLabel at=731,356,182,54 by=clip drawn focus heard="C Right" in=25 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
focus 11
edge 11 up same
edge 11 down 29
edge 29 up 11
edge 29 down 32
edge 29 left 34
edge 29 right 35
edge 30 up 11
edge 30 down 43
edge 30 left 41
edge 30 right 43
edge 31 up 39
edge 31 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 31 left 33
edge 31 right 38
edge 32 up 29
edge 32 down 33
edge 32 left 34
edge 32 right 35
edge 33 up 32
edge 33 down 31
edge 33 left 34
edge 33 right 35
edge 34 up 32
edge 34 down 33
edge 34 left same
edge 34 right 35
edge 35 up 32
edge 35 down 33
edge 35 left 34
edge 35 right 36
edge 36 up 32
edge 36 down 37
edge 36 left 35
edge 36 right 41
edge 37 up 36
edge 37 down 38
edge 37 left 39
edge 37 right 40
edge 38 up 37
edge 38 down 31
edge 38 left 39
edge 38 right 40
edge 39 up 37
edge 39 down 38
edge 39 left 33
edge 39 right 40
edge 40 up 37
edge 40 down 38
edge 40 left 39
edge 40 right 42
edge 41 up 30
edge 41 down 42
edge 41 left 37
edge 41 right 45
edge 42 up 45
edge 42 down 31
edge 42 left 41
edge 42 right 44
edge 43 up 30
edge 43 down 44
edge 43 left 45
edge 43 right 46
edge 44 up 43
edge 44 down 42
edge 44 left 45
edge 44 right 46
edge 45 up 43
edge 45 down 44
edge 45 left 41
edge 45 right 46
edge 46 up 43
edge 46 down 44
edge 46 left 45
edge 46 right 45
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 31
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 up 31
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5
