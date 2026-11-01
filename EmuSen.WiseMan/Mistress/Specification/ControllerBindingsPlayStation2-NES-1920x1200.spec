# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsPlayStation2-NES-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindingsPlayStation2
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#D4D4D4FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=118,229,162,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=136,238,126,52.5 box=84,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GENERAL layout=83.25,34.2 limit=118,229,162,72 owner=3 paints=#8D8D91FF role=text size=28.5 text=GENERAL
        5 TabItem at=280,229,90,72 by=clip drawn focus heard=NES limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=NES
          6 TextBlock at=298,238,54,52.5 box=36,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=NES layout=35.82,34.2 limit=280,229,90,72 owner=5 paints=#EEEEF0FF role=text size=28.5 text=NES
        7 TabItem at=370,229,73.5,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=388,238,37.5,52.5 box=25,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GB layout=24.48,34.2 limit=370,229,73.5,72 owner=7 paints=#8D8D91FF role=text size=28.5 text=GB
        9 TabItem at=443.5,229,108,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=461.5,238,72,52.5 box=48,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=SNES layout=47.05,34.2 limit=443.5,229,108,72 owner=9 paints=#8D8D91FF role=text size=28.5 text=SNES
        11 TabItem at=551.5,229,91.5,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=N64
          12 TextBlock at=569.5,238,55.5,52.5 box=37,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=N64 layout=36.08,34.2 limit=551.5,229,91.5,72 owner=11 paints=#8D8D91FF role=text size=28.5 text=N64
        13 TabItem at=643,229,150,72 by=clip drawn focus heard=SG-1000 limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=661,238,114,52.5 box=76,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=SG-1000 layout=75.81,34.2 limit=643,229,150,72 owner=13 paints=#8D8D91FF role=text size=28.5 text=SG-1000
        15 TabItem at=793,229,256.5,72 by=clip drawn focus heard="MASTER SYSTEM" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=811,238,220.5,52.5 box=147,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=146.01,34.2 limit=793,229,256.5,72 owner=15 paints=#8D8D91FF role=text size=28.5 text="MASTER SYSTEM"
        17 TabItem at=1049.5,229,150,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=1067.5,238,114,52.5 box=76,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=GENESIS layout=75.9,34.2 limit=1049.5,229,150,72 owner=17 paints=#8D8D91FF role=text size=28.5 text=GENESIS
        19 TabItem at=1199.5,229,151.5,72 by=clip drawn focus heard="SEGA CD" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=1217.5,238,115.5,52.5 box=77,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="SEGA CD" layout=76.41,34.2 limit=1199.5,229,151.5,72 owner=19 paints=#8D8D91FF role=text size=28.5 text="SEGA CD"
        21 TabItem at=1351,229,211.5,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          22 TextBlock at=1369,238,175.5,52.5 box=117,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard=PLAYSTATION layout=116.54,34.2 limit=1351,229,211.5,72 owner=21 paints=#8D8D91FF role=text size=28.5 text=PLAYSTATION
        23 TabItem at=1562.5,229,237,72 by=clip drawn focus heard="PLAYSTATION 2" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=1580.5,238,201,52.5 box=134,35 by=clip caps=19.9486 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=133.18,34.2 limit=1562.5,229,237,72 owner=23 paints=#8D8D91FF role=text size=28.5 text="PLAYSTATION 2"
        25 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=978 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
          26 StackPanel at=136,304,1650,1659 by=scroll in=25 limit=136,304,1650,681 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=136,304,1650,681
            27 DockPanel at=136,304,1650,675 by=scroll in=25 limit=136,304,1650,681 role=none
              28 ControllerDiagram at=136,304,1650,675 by=scroll caps=11.9682 face="Barlow Condensed" heard="NES controller" in=25 limit=136,304,1650,681 name=Diagram role=group size=17.1
                29 DiagramLabel at=572.5,325,247.5,81 by=clip drawn focus heard=Up in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                30 DiagramLabel at=305.5,877,261,81 by=clip drawn focus heard=Down in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                31 DiagramLabel at=242.5,625,247.5,81 by=clip drawn focus heard=Left in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                32 DiagramLabel at=578.5,877,256.5,81 by=clip drawn focus heard=Right in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                33 DiagramLabel at=845.5,877,265.5,81 by=clip drawn focus heard=Select in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                34 DiagramLabel at=1121.5,877,253.5,81 by=clip drawn focus heard=Start in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                35 DiagramLabel at=1046.5,325,247.5,81 by=clip drawn focus heard=B in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                36 DiagramLabel at=1432,622,247.5,81 by=clip drawn focus heard=A in=25 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
focus 5
edge 5 up same
edge 5 down 29
edge 29 up 5
edge 29 down 30
edge 29 left 31
edge 29 right 32
edge 30 up 29
edge 30 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 30 left 31
edge 30 right 32
edge 31 up 29
edge 31 down 30
edge 31 left same
edge 31 right 32
edge 32 up 29
edge 32 down 30
edge 32 left 31
edge 32 right 33
edge 33 up 32
edge 33 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 33 left 30
edge 33 right 34
edge 34 up 35
edge 34 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 34 left 33
edge 34 right 35
edge 35 up 29
edge 35 down 34
edge 35 left 34
edge 35 right 36
edge 36 up 29
edge 36 down 34
edge 36 left 35
edge 36 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 30
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 up 32
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 up 33
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/6
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/2 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/3/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/2 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/2/3
