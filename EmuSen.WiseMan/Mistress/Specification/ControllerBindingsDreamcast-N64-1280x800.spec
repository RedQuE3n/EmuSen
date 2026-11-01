# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsDreamcast-N64-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 38.4,24,1203.2,668
title "Controller Settings"
case ControllerBindingsDreamcast
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=38,124,1204,548 by=panel limit=38.4,124,1203.2,548 paints=#D4D4D4FF role=none seen=38.4,124,1203.2,548
    1 DockPanel at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
      2 Tabs at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=78,152,96,48 by=clip drawn focus heard=GENERAL limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=90,161,72,30 box=72,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GENERAL layout=71.56,29.4 limit=78,152,96,48 owner=3 paints=#8D8D91FF role=text size=24.5 text=GENERAL
        5 TabItem at=174,152,55,48 by=clip drawn focus heard=NES limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=186,161,31,30 box=31,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=NES layout=30.8,29.4 limit=174,152,55,48 owner=5 paints=#8D8D91FF role=text size=24.5 text=NES
        7 TabItem at=229,152,46,48 by=clip drawn focus heard=GB limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=241,161,22,30 box=22,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GB layout=21.05,29.4 limit=229,152,46,48 owner=7 paints=#8D8D91FF role=text size=24.5 text=GB
        9 TabItem at=275,152,65,48 by=clip drawn focus heard=SNES limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=287,161,41,30 box=41,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=SNES layout=40.45,29.4 limit=275,152,65,48 owner=9 paints=#8D8D91FF role=text size=24.5 text=SNES
        11 TabItem at=340,152,56,48 by=clip drawn focus heard=N64 limit=78,152,1124,504 op paints=#EEEEF0FF role=tabitem sideways text=N64
          12 TextBlock at=352,161,32,30 box=32,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=N64 layout=31.02,29.4 limit=340,152,56,48 owner=11 paints=#EEEEF0FF role=text size=24.5 text=N64
        13 TabItem at=396,152,90,48 by=clip drawn focus heard=SG-1000 limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=408,161,66,30 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=SG-1000 layout=65.17,29.4 limit=396,152,90,48 owner=13 paints=#8D8D91FF role=text size=24.5 text=SG-1000
        15 TabItem at=486,152,150,48 by=clip drawn focus heard="MASTER SYSTEM" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=498,161,126,30 box=126,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=125.51,29.4 limit=486,152,150,48 owner=15 paints=#8D8D91FF role=text size=24.5 text="MASTER SYSTEM"
        17 TabItem at=636,152,90,48 by=clip drawn focus heard=GENESIS limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=648,161,66,30 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GENESIS layout=65.24,29.4 limit=636,152,90,48 owner=17 paints=#8D8D91FF role=text size=24.5 text=GENESIS
        19 TabItem at=726,152,90,48 by=clip drawn focus heard="SEGA CD" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=738,161,66,30 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="SEGA CD" layout=65.68,29.4 limit=726,152,90,48 owner=19 paints=#8D8D91FF role=text size=24.5 text="SEGA CD"
        21 TabItem at=816,152,117,48 by=clip drawn focus heard=DREAMCAST limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=Dreamcast
          22 TextBlock at=828,161,93,30 box=93,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=DREAMCAST layout=92.17,29.4 limit=816,152,117,48 owner=21 paints=#8D8D91FF role=text size=24.5 text=DREAMCAST
        23 TabItem at=933,152,125,48 by=clip drawn focus heard=PLAYSTATION limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          24 TextBlock at=945,161,101,30 box=101,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=PLAYSTATION layout=100.18,29.4 limit=933,152,125,48 owner=23 paints=#8D8D91FF role=text size=24.5 text=PLAYSTATION
        25 TabItem at=1058,152,139,48 by=clip drawn focus heard="PLAYSTATION 2" limit=78,152,1124,504 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          26 TextBlock at=1070,161,115,30 box=115,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=114.49,29.4 limit=1058,152,139,48 owner=25 paints=#8D8D91FF role=text size=24.5 text="PLAYSTATION 2"
        27 ScrollViewer at=90,202,1100,454 by=clip fades=b limit=78,202,1124,454 overflow=1152 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=90,202,1100,454
          28 StackPanel at=90,202,1100,1606 by=scroll in=27 limit=90,202,1100,454 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=90,202,1100,454
            29 DockPanel at=90,202,1100,450 by=scroll in=27 limit=90,202,1100,454 role=none
              30 ControllerDiagram at=90,202,1100,450 by=scroll caps=11.9682 face="Barlow Condensed" heard="N64 controller" in=27 limit=90,202,1100,454 name=Diagram role=group size=17.1
                31 DiagramLabel at=105,216,166,54 by=clip drawn focus heard=L in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                32 DiagramLabel at=1002,216,173,54 by=clip drawn focus heard=R in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                33 DiagramLabel at=384,584,165,54 by=clip drawn focus heard=Z in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                34 DiagramLabel at=214,288,165,54 by=clip drawn focus heard=Up in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                35 DiagramLabel at=205,410,174,54 by=clip drawn focus heard=Down in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                36 DiagramLabel at=214,349,165,54 by=clip drawn focus heard=Left in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                37 DiagramLabel at=278,216,171,54 by=clip drawn focus heard=Right in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                38 DiagramLabel at=653,216,169,54 by=clip drawn focus heard=Start in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                39 DiagramLabel at=457,216,189,54 by=clip drawn focus heard="Stick Up" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                40 DiagramLabel at=767,584,206,54 by=clip drawn focus heard="Stick Down" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                41 DiagramLabel at=181,584,196,54 by=clip drawn focus heard="Stick Left" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                42 DiagramLabel at=557,584,203,54 by=clip drawn focus heard="Stick Right" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                43 DiagramLabel at=829,216,165,54 by=clip drawn focus heard=B in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                44 DiagramLabel at=731,425,165,54 by=clip drawn focus heard=A in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                45 DiagramLabel at=731,288,168,54 by=clip drawn focus heard="C Up" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                46 DiagramLabel at=920,391,185,54 by=clip drawn focus heard="C Down" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                47 DiagramLabel at=920,322,175,54 by=clip drawn focus heard="C Left" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
                48 DiagramLabel at=731,356,182,54 by=clip drawn focus heard="C Right" in=27 limit=90,202,1100,450 op paints=#D4D4D4FF role=button
focus 11
edge 11 up same
edge 11 down 31
edge 31 up 11
edge 31 down 34
edge 31 left 36
edge 31 right 37
edge 32 up 11
edge 32 down 45
edge 32 left 43
edge 32 right 45
edge 33 up 41
edge 33 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 33 left 35
edge 33 right 40
edge 34 up 31
edge 34 down 35
edge 34 left 36
edge 34 right 37
edge 35 up 34
edge 35 down 33
edge 35 left 36
edge 35 right 37
edge 36 up 34
edge 36 down 35
edge 36 left same
edge 36 right 37
edge 37 up 34
edge 37 down 35
edge 37 left 36
edge 37 right 38
edge 38 up 34
edge 38 down 39
edge 38 left 37
edge 38 right 43
edge 39 up 38
edge 39 down 40
edge 39 left 41
edge 39 right 42
edge 40 up 39
edge 40 down 33
edge 40 left 41
edge 40 right 42
edge 41 up 39
edge 41 down 40
edge 41 left 35
edge 41 right 42
edge 42 up 39
edge 42 down 40
edge 42 left 41
edge 42 right 44
edge 43 up 32
edge 43 down 44
edge 43 left 39
edge 43 right 47
edge 44 up 47
edge 44 down 33
edge 44 left 43
edge 44 right 46
edge 45 up 32
edge 45 down 46
edge 45 left 47
edge 45 right 48
edge 46 up 45
edge 46 down 44
edge 46 left 47
edge 46 right 48
edge 47 up 45
edge 47 down 46
edge 47 left 43
edge 47 right 48
edge 48 up 45
edge 48 down 46
edge 48 left 47
edge 48 right 47
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 33
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 up 33
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5
