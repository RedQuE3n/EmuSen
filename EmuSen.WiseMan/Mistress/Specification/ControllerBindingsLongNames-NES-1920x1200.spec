# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsLongNames-NES-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindingsLongNames
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#9C9CA0FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=118,229,174,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=General
          4 TextBlock at=136,238,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=118,229,174,72 owner=3 paints=#86868AFF role=text size=30 text=GENERAL
        5 TabItem at=292,229,97.5,72 by=clip drawn focus heard=NES limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=NES
          6 TextBlock at=310,238,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=292,229,97.5,72 owner=5 paints=#EEEEF0FF role=text size=30 text=NES
        7 TabItem at=389.5,229,76.5,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=407.5,238,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=389.5,229,76.5,72 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=466,229,115.5,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=484,238,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=466,229,115.5,72 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=581.5,229,96,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=599.5,238,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=581.5,229,96,72 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=677.5,229,160.5,72 by=clip drawn focus heard=SG-1000 limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=695.5,238,124.5,54 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=677.5,229,160.5,72 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=838,229,279,72 by=clip drawn focus heard="MASTER SYSTEM" limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=856,238,243,54 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=838,229,279,72 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=1117,229,165,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=1135,238,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=1117,229,165,72 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=1282,229,163.5,72 by=clip drawn focus heard="SEGA CD" limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=1300,238,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=1282,229,163.5,72 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=1445.5,229,234,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=1463.5,238,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=1445.5,229,234,72 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=978 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
          24 StackPanel at=136,304,1650,1659 by=scroll in=23 limit=136,304,1650,681 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF role=none seen=136,304,1650,681
            25 DockPanel at=136,304,1650,675 by=scroll in=23 limit=136,304,1650,681 role=none
              26 ControllerDiagram at=136,304,1650,675 by=scroll caps=11.9682 face="Barlow Condensed" heard="NES controller" in=23 limit=136,304,1650,681 name=Diagram role=group size=17.1
                27 DiagramLabel at=539.5,325,247.5,81 by=clip drawn focus heard=Up in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                28 DiagramLabel at=262,877,262.5,81 by=clip drawn focus heard=Down in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                29 DiagramLabel at=209.5,625,249,81 by=clip drawn focus heard=Left in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                30 DiagramLabel at=535,877,259.5,81 by=clip drawn focus heard=Right in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                31 DiagramLabel at=806.5,877,268.5,81 by=clip drawn focus heard=Select in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                32 DiagramLabel at=1085.5,877,315,81 by=clip drawn focus heard=Start in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                33 DiagramLabel at=1015,325,247.5,81 by=clip drawn focus heard=B in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                34 DiagramLabel at=1399,622,313.5,81 by=clip drawn focus heard=A in=23 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
focus 5
edge 5 up same
edge 5 down 27
edge 27 up 5
edge 27 down 28
edge 27 left 29
edge 27 right 30
edge 28 up 27
edge 28 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 28 left 29
edge 28 right 30
edge 29 up 27
edge 29 down 28
edge 29 left same
edge 29 right 30
edge 30 up 27
edge 30 down 28
edge 30 left 29
edge 30 right 31
edge 31 up 30
edge 31 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 31 left 28
edge 31 right 32
edge 32 up 33
edge 32 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 32 left 31
edge 32 right 33
edge 33 up 27
edge 33 down 32
edge 33 left 32
edge 33 right 34
edge 34 up 27
edge 34 down 32
edge 34 left 33
edge 34 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 28
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 up 30
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 up 30
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 up 32
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
