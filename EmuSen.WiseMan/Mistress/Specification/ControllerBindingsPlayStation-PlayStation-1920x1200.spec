# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsPlayStation-PlayStation-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 57.6,37,1804.8,1002
title "Controller Settings"
case ControllerBindingsPlayStation
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=58,187,1806,822 by=panel limit=58,187,1804.4,822 paints=#9C9CA0FF role=none seen=58,187,1804.4,822
    1 DockPanel at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
      2 Grid at=118,229,1686,756 by=panel limit=58,187,1804.4,822 role=none
        3 Tabs at=118,229,1686,756 by=panel limit=58,187,1804.4,822 name=Tabs paints=#9C9CA0FF role=tab
          4 TabItem at=118,229,174,72 by=clip drawn focus heard=GENERAL limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=General
            5 TextBlock at=136,238,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=118,229,174,72 owner=4 paints=#86868AFF role=text size=30 text=GENERAL
          6 TabItem at=292,229,97.5,72 by=clip drawn focus heard=NES limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=NES
            7 TextBlock at=310,238,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=292,229,97.5,72 owner=6 paints=#86868AFF role=text size=30 text=NES
          8 TabItem at=389.5,229,76.5,72 by=clip drawn focus heard=GB limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=GB
            9 TextBlock at=407.5,238,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=389.5,229,76.5,72 owner=8 paints=#86868AFF role=text size=30 text=GB
          10 TabItem at=466,229,115.5,72 by=clip drawn focus heard=SNES limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=SNES
            11 TextBlock at=484,238,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=466,229,115.5,72 owner=10 paints=#86868AFF role=text size=30 text=SNES
          12 TabItem at=581.5,229,96,72 by=clip drawn focus heard=N64 limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=N64
            13 TextBlock at=599.5,238,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=581.5,229,96,72 owner=12 paints=#86868AFF role=text size=30 text=N64
          14 TabItem at=677.5,229,165,72 by=clip drawn focus heard=GENESIS limit=118,229,1686,756 paints=#86868AFF role=tabitem sideways text=Genesis
            15 TextBlock at=695.5,238,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=677.5,229,165,72 owner=14 paints=#86868AFF role=text size=30 text=GENESIS
          16 TabItem at=842.5,229,234,72 by=clip drawn focus heard=PLAYSTATION limit=118,229,1686,756 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
            17 TextBlock at=860.5,238,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=842.5,229,234,72 owner=16 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
          18 ScrollViewer at=136,304,1650,681 by=clip fades=b limit=118,304,1686,681 overflow=2094 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=136,304,1650,681
            19 StackPanel at=136,304,1650,2775 by=scroll in=18 limit=136,304,1650,681 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=136,304,1650,681
              20 DockPanel at=136,304,1650,675 by=scroll in=18 limit=136,304,1650,681 role=none
                21 ControllerDiagram at=136,304,1650,675 by=scroll caps=11.9682 face="Barlow Condensed" heard="PlayStation controller" in=18 limit=136,304,1650,681 name=Diagram role=group size=17.1
                  22 DiagramLabel at=413.5,325,247.5,81 by=clip drawn focus heard=L2 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  23 DiagramLabel at=394,445,253.5,81 by=clip drawn focus heard=L1 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  24 DiagramLabel at=1280.5,524.5,250.5,81 by=clip drawn focus heard=R2 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  25 DiagramLabel at=1280.5,616,265.5,81 by=clip drawn focus heard=R1 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  26 DiagramLabel at=155.5,325,247.5,81 by=clip drawn focus heard=Up in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  27 DiagramLabel at=296.5,877,262.5,81 by=clip drawn focus heard=Down in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  28 DiagramLabel at=398.5,538,249,81 by=clip drawn focus heard=Left in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  29 DiagramLabel at=673,325,259.5,81 by=clip drawn focus heard=Right in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  30 DiagramLabel at=1280.5,433,286.5,81 by=clip drawn focus heard=Triangle in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  31 DiagramLabel at=1280.5,709,264,81 by=clip drawn focus heard=Circle in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  32 DiagramLabel at=1370.5,877,262.5,81 by=clip drawn focus heard=Cross in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  33 DiagramLabel at=1490.5,325,276,81 by=clip drawn focus heard=Square in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  34 DiagramLabel at=943,325,268.5,81 by=clip drawn focus heard=Select in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  35 DiagramLabel at=1222,325,256.5,81 by=clip drawn focus heard=Start in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  36 DiagramLabel at=569.5,877,247.5,81 by=clip drawn focus heard=L3 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  37 DiagramLabel at=1112.5,877,247.5,81 by=clip drawn focus heard=R3 in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
                  38 DiagramLabel at=827.5,877,273,81 by=clip drawn focus heard=Analog in=18 limit=136,304,1650,675 op paints=#9C9CA0FF role=button
        39 StackPanel at=1255,229,549,66 by=panel limit=58,187,1804.4,822 name=PlayerBar role=none
          40 TextBlock at=1255,241,76.5,43.5 box=51,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard=Player layout=50.16,28.8 limit=58,187,1804.4,822 paints=#9C9CA0FF role=text size=24 text=Player
          41 Dropdown at=1346.5,229,195,66 by=panel drawn focus heard=Player limit=58,187,1804.4,822 name=PlayerSelector op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
            42 TextBlock at=1364.5,236.5,129,48 box=86,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Player 1" layout=66.2,31.2 limit=1364.5,236.5,129,48 owner=41 paints=#9C9CA0FF role=text size=26 text="Player 1"
            43 ViewboxContainer at=1508.5,257.1,18,9.81 by=clip limit=1508.5,257.1,18,9.81 nohit role=none
          44 Button at=1556.5,229,247.5,66 by=panel drawn heard="USE PLAYER 1'S" limit=58,187,1804.4,822 name=UsePlayer1Button off paints=#55555AFF,#9C9CA0FF role=button text="Use Player 1's"
            45 TextBlock at=1582,238,196.5,48 box=131,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="USE PLAYER 1'S" layout=130.94,31.2 limit=1556.5,229,247.5,66 off owner=44 paints=#55555AFF role=text size=26 text="USE PLAYER 1'S"
focus 16
edge 16 up same
edge 16 down 26
edge 22 up 41
edge 22 down 23
edge 22 left 28
edge 22 right 23
edge 23 up 22
edge 23 down 26
edge 23 left 22
edge 23 right 34
edge 24 up 41
edge 24 down 25
edge 24 left 25
edge 24 right 31
edge 25 up 24
edge 25 down 30
edge 25 left 35
edge 25 right 24
edge 26 up 23
edge 26 down 27
edge 26 left 28
edge 26 right 29
edge 27 up 26
edge 27 down 36
edge 27 left 28
edge 27 right 29
edge 28 up 26
edge 28 down 27
edge 28 left 16
edge 28 right 29
edge 29 up 26
edge 29 down 27
edge 29 left 28
edge 29 right 34
edge 30 up 25
edge 30 down 32
edge 30 left 33
edge 30 right 31
edge 31 up 30
edge 31 down 32
edge 31 left 33
edge 31 right 33
edge 32 up 30
edge 32 down 37
edge 32 left 33
edge 32 right 31
edge 33 up 30
edge 33 down 32
edge 33 left 35
edge 33 right 31
edge 34 up 23
edge 34 down 36
edge 34 left 29
edge 34 right 35
edge 35 up 25
edge 35 down 37
edge 35 left 34
edge 35 right 33
edge 36 up 34
edge 36 down 38
edge 36 left 27
edge 36 right 38
edge 37 up 33
edge 37 down 38
edge 37 left 38
edge 37 right 32
edge 38 up 34
edge 38 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/5
edge 38 left 36
edge 38 right 37
edge 41 up 16
edge 41 down 35
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/5 up 38
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/5 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/1/5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/5 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/5 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/4/0/6
