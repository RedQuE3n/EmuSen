# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsSaturn-N64-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindingsSaturn
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#D4D4D4FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=118,229,148.5,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=136,241,112.5,46.5 box=75,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=GENERAL layout=74.49,30.6 limit=118,229,148.5,72 owner=3 paints=#8D8D91FF role=text size=25.5 text=GENERAL
        5 TabItem at=266.5,229,85.5,72 by=clip drawn focus heard=NES limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=284.5,241,49.5,46.5 box=33,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=NES layout=32.05,30.6 limit=266.5,229,85.5,72 owner=5 paints=#8D8D91FF role=text size=25.5 text=NES
        7 TabItem at=352,229,69,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=370,241,33,46.5 box=22,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=GB layout=21.9,30.6 limit=352,229,69,72 owner=7 paints=#8D8D91FF role=text size=25.5 text=GB
        9 TabItem at=421,229,100.5,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=439,241,64.5,46.5 box=43,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=SNES layout=42.1,30.6 limit=421,229,100.5,72 owner=9 paints=#8D8D91FF role=text size=25.5 text=SNES
        11 TabItem at=521.5,229,85.5,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=N64
          12 TextBlock at=539.5,241,49.5,46.5 box=33,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=N64 layout=32.28,30.6 limit=521.5,229,85.5,72 owner=11 paints=#EEEEF0FF role=text size=25.5 text=N64
        13 TabItem at=607,229,138,72 by=clip drawn focus heard=SG-1000 limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=625,241,102,46.5 box=68,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=SG-1000 layout=67.83,30.6 limit=607,229,138,72 owner=13 paints=#8D8D91FF role=text size=25.5 text=SG-1000
        15 TabItem at=745,229,232.5,72 by=clip drawn focus heard="MASTER SYSTEM" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=763,241,196.5,46.5 box=131,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=130.64,30.6 limit=745,229,232.5,72 owner=15 paints=#8D8D91FF role=text size=25.5 text="MASTER SYSTEM"
        17 TabItem at=977.5,229,138,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=995.5,241,102,46.5 box=68,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=GENESIS layout=67.91,30.6 limit=977.5,229,138,72 owner=17 paints=#8D8D91FF role=text size=25.5 text=GENESIS
        19 TabItem at=1115.5,229,139.5,72 by=clip drawn focus heard="SEGA CD" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=1133.5,241,103.5,46.5 box=69,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard="SEGA CD" layout=68.37,30.6 limit=1115.5,229,139.5,72 owner=19 paints=#8D8D91FF role=text size=25.5 text="SEGA CD"
        21 TabItem at=1255,229,132,72 by=clip drawn focus heard=SATURN limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=Saturn
          22 TextBlock at=1273,241,96,46.5 box=64,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=SATURN layout=63.14,30.6 limit=1255,229,132,72 owner=21 paints=#8D8D91FF role=text size=25.5 text=SATURN
        23 TabItem at=1387,229,193.5,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          24 TextBlock at=1405,241,157.5,46.5 box=105,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard=PLAYSTATION layout=104.27,30.6 limit=1387,229,193.5,72 owner=23 paints=#8D8D91FF role=text size=25.5 text=PLAYSTATION
        25 TabItem at=1580.5,229,216,72 by=clip drawn focus heard="PLAYSTATION 2" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          26 TextBlock at=1598.5,241,180,46.5 box=120,31 by=clip caps=17.8488 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=119.16,30.6 limit=1580.5,229,216,72 owner=25 paints=#8D8D91FF role=text size=25.5 text="PLAYSTATION 2"
        27 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=1728 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
          28 StackPanel at=136,304,1650,2409 by=scroll in=27 limit=136,304,1650,681 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=136,304,1650,681
            29 DockPanel at=136,304,1650,675 by=scroll in=27 limit=136,304,1650,681 role=none
              30 ControllerDiagram at=136,304,1650,675 by=scroll caps=11.9682 face="Barlow Condensed" heard="N64 controller" in=27 limit=136,304,1650,681 name=Diagram role=group size=17.1
                31 DiagramLabel at=158.5,325,249,81 by=clip drawn focus heard=L in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                32 DiagramLabel at=1504,325,259.5,81 by=clip drawn focus heard=R in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                33 DiagramLabel at=577,877,247.5,81 by=clip drawn focus heard=Z in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                34 DiagramLabel at=322,433,247.5,81 by=clip drawn focus heard=Up in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                35 DiagramLabel at=308.5,616,261,81 by=clip drawn focus heard=Down in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                36 DiagramLabel at=322,524.5,247.5,81 by=clip drawn focus heard=Left in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                37 DiagramLabel at=418,325,256.5,81 by=clip drawn focus heard=Right in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                38 DiagramLabel at=980.5,325,253.5,81 by=clip drawn focus heard=Start in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                39 DiagramLabel at=686.5,325,283.5,81 by=clip drawn focus heard="Stick Up" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                40 DiagramLabel at=1151.5,877,309,81 by=clip drawn focus heard="Stick Down" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                41 DiagramLabel at=272.5,877,294,81 by=clip drawn focus heard="Stick Left" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                42 DiagramLabel at=836.5,877,304.5,81 by=clip drawn focus heard="Stick Right" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                43 DiagramLabel at=1244.5,325,247.5,81 by=clip drawn focus heard=B in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                44 DiagramLabel at=1097.5,638.5,247.5,81 by=clip drawn focus heard=A in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                45 DiagramLabel at=1097.5,433,252,81 by=clip drawn focus heard="C Up" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                46 DiagramLabel at=1381,587.5,277.5,81 by=clip drawn focus heard="C Down" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                47 DiagramLabel at=1381,484,262.5,81 by=clip drawn focus heard="C Left" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                48 DiagramLabel at=1097.5,535,273,81 by=clip drawn focus heard="C Right" in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
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
