# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ControllerBindingsGenesisTeamPlayer-PlayStation-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 38.4,24,1203.2,668
title "Controller Settings"
case ControllerBindingsGenesisTeamPlayer
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=38,124,1204,548 by=panel limit=38.4,124,1203.2,548 paints=#9C9CA0FF role=none seen=38.4,124,1203.2,548
    1 DockPanel at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 role=none
      2 Tabs at=78,152,1124,504 by=panel limit=38.4,124,1203.2,548 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=78,152,116,48 by=clip drawn focus heard=GENERAL limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=General
          4 TextBlock at=90,158,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=78,152,116,48 owner=3 paints=#86868AFF role=text size=30 text=GENERAL
        5 TabItem at=194,152,65,48 by=clip drawn focus heard=NES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=206,158,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=194,152,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=259,152,51,48 by=clip drawn focus heard=GB limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=271,158,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=259,152,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=310,152,77,48 by=clip drawn focus heard=SNES limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=322,158,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=310,152,77,48 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=387,152,64,48 by=clip drawn focus heard=N64 limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=399,158,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=387,152,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=451,152,107,48 by=clip drawn focus heard=SG-1000 limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=463,158,83,36 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=451,152,107,48 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=558,152,186,48 by=clip drawn focus heard="MASTER SYSTEM" limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=570,158,162,36 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=558,152,186,48 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=744,152,110,48 by=clip drawn focus heard=GENESIS limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=756,158,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=744,152,110,48 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=854,152,109,48 by=clip drawn focus heard="SEGA CD" limit=78,152,1124,504 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=866,158,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=854,152,109,48 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=963,152,156,48 by=clip drawn focus heard=PLAYSTATION limit=78,152,1124,504 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
          22 TextBlock at=975,158,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=963,152,156,48 owner=21 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
        23 ScrollViewer at=90,202,1100,454 by=clip fades=b limit=78,202,1124,454 overflow=1452 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=90,202,1100,454
          24 StackPanel at=90,202,1100,1906 by=scroll in=23 limit=90,202,1100,454 paints=#0E0E10FF,#55555AFF,#86868AFF,#9C9CA0FF role=none seen=90,202,1100,454
            25 DockPanel at=90,202,1100,450 by=scroll in=23 limit=90,202,1100,454 role=none
              26 ControllerDiagram at=90,202,1100,450 by=scroll caps=11.9682 face="Barlow Condensed" heard="PlayStation controller" in=23 limit=90,202,1100,454 name=Diagram role=group size=17.1
                27 DiagramLabel at=275,216,165,54 by=clip drawn focus heard=L2 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                28 DiagramLabel at=262,296,169,54 by=clip drawn focus heard=L1 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                29 DiagramLabel at=853,349,167,54 by=clip drawn focus heard=R2 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                30 DiagramLabel at=853,410,177,54 by=clip drawn focus heard=R1 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                31 DiagramLabel at=103,216,165,54 by=clip drawn focus heard=Up in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                32 DiagramLabel at=197,584,175,54 by=clip drawn focus heard=Down in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                33 DiagramLabel at=265,358,166,54 by=clip drawn focus heard=Left in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                34 DiagramLabel at=448,216,173,54 by=clip drawn focus heard=Right in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                35 DiagramLabel at=853,288,191,54 by=clip drawn focus heard=Triangle in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                36 DiagramLabel at=853,472,176,54 by=clip drawn focus heard=Circle in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                37 DiagramLabel at=913,584,175,54 by=clip drawn focus heard=Cross in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                38 DiagramLabel at=993,216,184,54 by=clip drawn focus heard=Square in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                39 DiagramLabel at=628,216,179,54 by=clip drawn focus heard=Select in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                40 DiagramLabel at=814,216,171,54 by=clip drawn focus heard=Start in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                41 DiagramLabel at=379,584,165,54 by=clip drawn focus heard=L3 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                42 DiagramLabel at=741,584,165,54 by=clip drawn focus heard=R3 in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
                43 DiagramLabel at=551,584,182,54 by=clip drawn focus heard=Analog in=23 limit=90,202,1100,450 op paints=#9C9CA0FF role=button
focus 21
edge 21 up same
edge 21 down 31
edge 27 up 21
edge 27 down 28
edge 27 left 33
edge 27 right 28
edge 28 up 27
edge 28 down 31
edge 28 left 27
edge 28 right 39
edge 29 up 35
edge 29 down 30
edge 29 left 30
edge 29 right 36
edge 30 up 29
edge 30 down 35
edge 30 left 40
edge 30 right 29
edge 31 up 28
edge 31 down 32
edge 31 left 33
edge 31 right 34
edge 32 up 31
edge 32 down 41
edge 32 left 33
edge 32 right 34
edge 33 up 31
edge 33 down 32
edge 33 left same
edge 33 right 34
edge 34 up 31
edge 34 down 32
edge 34 left 33
edge 34 right 39
edge 35 up 30
edge 35 down 37
edge 35 left 38
edge 35 right 36
edge 36 up 35
edge 36 down 37
edge 36 left 38
edge 36 right same
edge 37 up 35
edge 37 down 42
edge 37 left 38
edge 37 right 36
edge 38 up 35
edge 38 down 37
edge 38 left 40
edge 38 right 36
edge 39 up 28
edge 39 down 41
edge 39 left 34
edge 39 right 40
edge 40 up 30
edge 40 down 42
edge 40 left 39
edge 40 right 38
edge 41 up 39
edge 41 down 43
edge 41 left 32
edge 41 right 43
edge 42 up 38
edge 42 down 43
edge 42 left 43
edge 42 right 37
edge 43 up 39
edge 43 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1
edge 43 left 41
edge 43 right 42
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 up 43
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/1/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/4/0/3
