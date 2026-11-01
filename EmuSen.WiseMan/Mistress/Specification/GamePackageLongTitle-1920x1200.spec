# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageLongTitle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,37,1260,1041
title "Game Package"
case GamePackageLongTitle
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 ButtonBar at=1395,958,135,66 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=toolbar
        3 Button at=1395,958,135,66 by=clip drawn focus heard=CLOSE limit=1395,958,135,66 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
          4 TextBlock at=1420.5,967,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,958,135,66 owner=3 paints=#9C9CA0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,229,1140,714 by=panel fades=b limit=330,187,1260,861 overflow=730.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,714
        6 GamePackageDetail at=390,229,1134,1444.5 by=scroll in=5 limit=390,229,1140,714 name=GamePackageDetail paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=390,229,1134,714
          7 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          8 FieldRow at=390,397,1134,148.5 by=scroll heard=Package in=5 limit=390,229,1140,714 paints=#9C9CA0FF role=group
            9 TextBlock at=390,397,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=69.48,28.8 limit=390,397,1134,148.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            10 StackPanel at=390,446.5,1134,99 by=clip heard=Package in=5 limit=390,397,1134,148.5 paints=#9C9CA0FF role=none
              11 TextBlock at=390,446.5,1134,87 box=756,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library." in=5 layout=738.72,57.6 limit=390,397,1134,148.5 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library."
          12 FieldRow at=390,569.5,1134,310.5 by=scroll heard="Soft patches" in=5 limit=390,229,1140,714 paints=#9C9CA0FF role=group
            13 TextBlock at=390,569.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=102.91,28.8 limit=390,569.5,1134,310.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            14 HintText at=390,619,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=727.86,48 limit=390,569.5,1134,310.5 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            15 StackPanel at=390,697,1134,183 by=clip heard="Soft patches" in=5 limit=390,569.5,1134,310.5 role=none
              16 StackPanel at=390,697,1134,183 by=clip in=5 limit=390,569.5,1134,310.5 name=GamePackagePatches role=none
                17 DockPanel at=390,697,1134,87 by=clip in=5 limit=390,569.5,1134,310.5 role=none
                  18 Button at=1366.5,707.5,157.5,66 by=clip drawn focus heard=REMOVE in=5 limit=390,569.5,1134,310.5 name=GamePackagePatch.0.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    19 TextBlock at=1392,716.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=70.56,31.2 limit=1366.5,707.5,157.5,66 owner=18 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  20 TextBlock at=390,697,976.5,87 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)" in=5 layout=619.68,57.6 limit=390,569.5,1134,310.5 paints=#9C9CA0FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)"
                21 DockPanel at=390,793,1134,87 by=clip in=5 limit=390,569.5,1134,310.5 role=none
                  22 Button at=1366.5,803.5,157.5,66 by=clip drawn focus heard=REMOVE in=5 limit=390,569.5,1134,310.5 name=GamePackagePatch.1.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    23 TextBlock at=1392,812.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=70.56,31.2 limit=1366.5,803.5,157.5,66 owner=22 paints=#9C9CA0FF role=text size=26 text=REMOVE
                  24 TextBlock at=390,793,976.5,87 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)" in=5 layout=623.18,57.6 limit=390,569.5,1134,310.5 paints=#9C9CA0FF role=text size=24 text="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)"
          25 FieldRow at=390,904,1134,279 by=scroll heard="Add a patch" in=5 limit=390,229,1140,714 paints=#55555AFF,#86868AFF,#9C9CA0FF role=group seen=390,904,1134,39
            26 TextBlock at=390,904,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a patch" in=5 layout=96.05,28.8 limit=390,904,1134,279 name=PART_Label paints=#9C9CA0FF part role=text seen=390,904,1134,39 size=24 text="Add a patch"
focus 18
edge 3 up 22
edge 3 down item:ButtonBar/0//
edge 3 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 3 right same
edge 18 up same
edge 18 down 22
edge 18 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 18 right same
edge 22 up 18
edge 22 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 22 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 22 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 up 22
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 right 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 right 22
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
