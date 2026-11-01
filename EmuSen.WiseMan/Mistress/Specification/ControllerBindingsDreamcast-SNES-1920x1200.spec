# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsDreamcast-SNES-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindingsDreamcast
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#D4D4D4FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=118,229,144,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=136,242.5,108,45 box=72,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GENERAL layout=71.56,29.4 limit=118,229,144,72 owner=3 paints=#8D8D91FF role=text size=24.5 text=GENERAL
        5 TabItem at=262,229,82.5,72 by=clip drawn focus heard=NES limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=280,242.5,46.5,45 box=31,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=NES layout=30.8,29.4 limit=262,229,82.5,72 owner=5 paints=#8D8D91FF role=text size=24.5 text=NES
        7 TabItem at=344.5,229,69,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=362.5,242.5,33,45 box=22,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GB layout=21.05,29.4 limit=344.5,229,69,72 owner=7 paints=#8D8D91FF role=text size=24.5 text=GB
        9 TabItem at=413.5,229,97.5,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          10 TextBlock at=431.5,242.5,61.5,45 box=41,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=SNES layout=40.45,29.4 limit=413.5,229,97.5,72 owner=9 paints=#EEEEF0FF role=text size=24.5 text=SNES
        11 TabItem at=511,229,84,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=N64
          12 TextBlock at=529,242.5,48,45 box=32,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=N64 layout=31.02,29.4 limit=511,229,84,72 owner=11 paints=#8D8D91FF role=text size=24.5 text=N64
        13 TabItem at=595,229,135,72 by=clip drawn focus heard=SG-1000 limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=613,242.5,99,45 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=SG-1000 layout=65.17,29.4 limit=595,229,135,72 owner=13 paints=#8D8D91FF role=text size=24.5 text=SG-1000
        15 TabItem at=730,229,225,72 by=clip drawn focus heard="MASTER SYSTEM" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=748,242.5,189,45 box=126,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=125.51,29.4 limit=730,229,225,72 owner=15 paints=#8D8D91FF role=text size=24.5 text="MASTER SYSTEM"
        17 TabItem at=955,229,135,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=973,242.5,99,45 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=GENESIS layout=65.24,29.4 limit=955,229,135,72 owner=17 paints=#8D8D91FF role=text size=24.5 text=GENESIS
        19 TabItem at=1090,229,135,72 by=clip drawn focus heard="SEGA CD" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=1108,242.5,99,45 box=66,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="SEGA CD" layout=65.68,29.4 limit=1090,229,135,72 owner=19 paints=#8D8D91FF role=text size=24.5 text="SEGA CD"
        21 TabItem at=1225,229,175.5,72 by=clip drawn focus heard=DREAMCAST limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=Dreamcast
          22 TextBlock at=1243,242.5,139.5,45 box=93,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=DREAMCAST layout=92.17,29.4 limit=1225,229,175.5,72 owner=21 paints=#8D8D91FF role=text size=24.5 text=DREAMCAST
        23 TabItem at=1400.5,229,187.5,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          24 TextBlock at=1418.5,242.5,151.5,45 box=101,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard=PLAYSTATION layout=100.18,29.4 limit=1400.5,229,187.5,72 owner=23 paints=#8D8D91FF role=text size=24.5 text=PLAYSTATION
        25 TabItem at=1588,229,208.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=118,229,1686,756 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          26 TextBlock at=1606,242.5,172.5,45 box=115,30 by=clip caps=17.1488 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=114.49,29.4 limit=1588,229,208.5,72 owner=25 paints=#8D8D91FF role=text size=24.5 text="PLAYSTATION 2"
        27 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=1278 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
          28 StackPanel at=136,304,1650,1959 by=scroll in=27 limit=136,304,1650,681 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=136,304,1650,681
            29 DockPanel at=136,304,1650,675 by=scroll in=27 limit=136,304,1650,681 role=none
              30 ControllerDiagram at=136,304,1650,675 by=scroll caps=11.9682 face="Barlow Condensed" heard="SNES controller" in=27 limit=136,304,1650,681 name=Diagram role=group size=17.1
                31 DiagramLabel at=178,325,249,81 by=clip drawn focus heard=L in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                32 DiagramLabel at=1430.5,457,259.5,81 by=clip drawn focus heard=R in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                33 DiagramLabel at=439,325,247.5,81 by=clip drawn focus heard=Up in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                34 DiagramLabel at=547,877,261,81 by=clip drawn focus heard=Down in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                35 DiagramLabel at=295,584.5,247.5,81 by=clip drawn focus heard=Left in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                36 DiagramLabel at=820,877,256.5,81 by=clip drawn focus heard=Right in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                37 DiagramLabel at=697,325,265.5,81 by=clip drawn focus heard=Select in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                38 DiagramLabel at=973,325,253.5,81 by=clip drawn focus heard=Start in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                39 DiagramLabel at=1496.5,325,247.5,81 by=clip drawn focus heard=X in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                40 DiagramLabel at=1430.5,613,247.5,81 by=clip drawn focus heard=A in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                41 DiagramLabel at=1087,877,247.5,81 by=clip drawn focus heard=B in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
                42 DiagramLabel at=1238.5,325,247.5,81 by=clip drawn focus heard=Y in=27 limit=136,304,1650,675 op paints=#D4D4D4FF role=button
focus 9
edge 9 up same
edge 9 down 31
edge 31 up 9
edge 31 down 35
edge 31 left same
edge 31 right 33
edge 32 up 39
edge 32 down 40
edge 32 left 39
edge 32 right same
edge 33 up 31
edge 33 down 34
edge 33 left 35
edge 33 right 36
edge 34 up 33
edge 34 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 34 left 35
edge 34 right 36
edge 35 up 33
edge 35 down 34
edge 35 left 31
edge 35 right 36
edge 36 up 33
edge 36 down 34
edge 36 left 35
edge 36 right 37
edge 37 up 31
edge 37 down 34
edge 37 left 36
edge 37 right 38
edge 38 up 32
edge 38 down 41
edge 38 left 37
edge 38 right 42
edge 39 up 32
edge 39 down 41
edge 39 left 42
edge 39 right 40
edge 40 up 39
edge 40 down 41
edge 40 left 42
edge 40 right 32
edge 41 up 39
edge 41 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 41 left 42
edge 41 right 40
edge 42 up 39
edge 42 down 41
edge 42 left 38
edge 42 right 40
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 34
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 up 34
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/1/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/5 up 36
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
