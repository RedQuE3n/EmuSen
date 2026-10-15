# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsGenesis-GB-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 38.4,24,1203.2,668
title "Controller Settings"
case ControllerBindingsGenesis
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=38,124,1204,548 by=panel limit=38.4,124,1203.2,548 paints=#9C9CA0FF role=none seen=38.4,124,1203.2,548
    1 DockPanel at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
      2 Grid at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
        3 Tabs at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 name=Tabs paints=#9C9CA0FF role=tab
          4 TabItem at=78,152,116,48 by=clip drawn focus heard=GENERAL limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=General
            5 TextBlock at=90,158,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=78,152,116,48 owner=4 paints=#86868AFF role=text size=30 text=GENERAL
          6 TabItem at=194,152,65,48 by=clip drawn focus heard=NES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=NES
            7 TextBlock at=206,158,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=194,152,65,48 owner=6 paints=#86868AFF role=text size=30 text=NES
          8 TabItem at=259,152,51,48 by=clip drawn focus heard=GB limit=78,152,1124,504 op paints=#EEEEF0FF role=tabitem sideways text=GB
            9 TextBlock at=271,158,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=259,152,51,48 owner=8 paints=#EEEEF0FF role=text size=30 text=GB
          10 TabItem at=310,152,77,48 by=clip drawn focus heard=SNES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=SNES
            11 TextBlock at=322,158,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=310,152,77,48 owner=10 paints=#86868AFF role=text size=30 text=SNES
          12 TabItem at=387,152,64,48 by=clip drawn focus heard=N64 limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=N64
            13 TextBlock at=399,158,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=387,152,64,48 owner=12 paints=#86868AFF role=text size=30 text=N64
          14 TabItem at=451,152,110,48 by=clip drawn focus heard=GENESIS limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=Genesis
            15 TextBlock at=463,158,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=451,152,110,48 owner=14 paints=#86868AFF role=text size=30 text=GENESIS
          16 TabItem at=561,152,156,48 by=clip drawn focus heard=PLAYSTATION limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=PlayStation
            17 TextBlock at=573,158,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=561,152,156,48 owner=16 paints=#86868AFF role=text size=30 text=PLAYSTATION
          18 ScrollViewer at=90,202,1100,454 by=clip fades=b limit=78,202,1124,454 overflow=557 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=90,202,1100,454
            19 StackPanel at=90,202,1100,1011 by=scroll in=18 limit=90,202,1100,454 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=90,202,1100,454
              20 DockPanel at=90,202,1100,450 by=scroll in=18 limit=90,202,1100,454 role=none
                21 ControllerDiagram at=90,202,1100,450 by=scroll caps=11.9682 face="Barlow Condensed" heard="GB controller" in=18 limit=90,202,1100,454 name=Diagram role=group size=17.1
                  22 DiagramLabel at=357,356,165,54 by=clip drawn focus heard=Up in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  23 DiagramLabel at=347,478,175,54 by=clip drawn focus heard=Down in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  24 DiagramLabel at=356,417,166,54 by=clip drawn focus heard=Left in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  25 DiagramLabel at=520,585,173,54 by=clip drawn focus heard=Right in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  26 DiagramLabel at=782,442,165,54 by=clip drawn focus heard=B in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  27 DiagramLabel at=782,381,165,54 by=clip drawn focus heard=A in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  28 DiagramLabel at=333,585,179,54 by=clip drawn focus heard=Select in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                  29 DiagramLabel at=700,585,171,54 by=clip drawn focus heard=Start in=18 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
focus 8
edge 8 up same
edge 8 down 22
edge 22 up 8
edge 22 down 23
edge 22 left 24
edge 22 right 25
edge 23 up 22
edge 23 down 28
edge 23 left 24
edge 23 right 25
edge 24 up 22
edge 24 down 23
edge 24 left 8
edge 24 right 25
edge 25 up 22
edge 25 down 23
edge 25 left 24
edge 25 right 26
edge 26 up 27
edge 26 down 29
edge 26 left 25
edge 26 right 27
edge 27 up 22
edge 27 down 29
edge 27 left 26
edge 27 right same
edge 28 up 25
edge 28 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/2
edge 28 left 24
edge 28 right 29
edge 29 up 26
edge 29 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/5
edge 29 left 28
edge 29 right 27
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/2 up 28
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/2 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/1/2
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/2 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/2 right path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/0/1/1/0/0/0/0/2/3/0/3
