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
      2 StackPanel at=390,874,1140,150 by=panel limit=330,187,1260,861 role=none
        3 HintText at=390,874,1140,72 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Added A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1. It is named as RetroArch finds it." layout=689.92,48 limit=330,187,1260,861 name=GamePackageStatus paints=#86868AFF role=text size=20 text="Added A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1. It is named as RetroArch finds it."
        4 ButtonBar at=1395,958,135,66 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=toolbar
          5 Button at=1395,958,135,66 by=clip drawn focus heard=CLOSE limit=1395,958,135,66 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=1420.5,967,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,958,135,66 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=390,229,1140,630 by=panel fades=b limit=330,187,1260,861 overflow=597 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,630
        8 StackPanel at=390,229,1134,1227 by=scroll in=7 limit=390,229,1140,630 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=390,229,1134,630
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=390,397,1134,148.5 by=scroll heard=Package in=7 limit=390,229,1140,630 paints=#9C9CA0FF role=group
            11 TextBlock at=390,397,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=390,397,1134,148.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=390,446.5,1134,99 by=clip heard=Package in=7 limit=390,397,1134,148.5 paints=#9C9CA0FF role=none
              13 TextBlock at=390,446.5,1134,87 box=756,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library." in=7 layout=738.72,57.6 limit=390,397,1134,148.5 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library."
          14 FieldRow at=390,569.5,1134,310.5 by=scroll heard="Soft patches" in=7 limit=390,229,1140,630 paints=#9C9CA0FF role=group seen=390,569.5,1134,289.5
            15 TextBlock at=390,569.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=390,569.5,1134,310.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            16 HintText at=390,619,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=390,569.5,1134,310.5 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            17 StackPanel at=390,697,1134,183 by=clip heard="Soft patches" in=7 limit=390,569.5,1134,310.5 role=none seen=390,697,1134,162
              18 StackPanel at=390,697,1134,183 by=clip in=7 limit=390,569.5,1134,310.5 name=GamePackagePatches role=none seen=390,697,1134,162
                19 DockPanel at=390,697,1134,87 by=clip in=7 limit=390,569.5,1134,310.5 role=none
                  20 Button at=1366.5,707.5,157.5,66 by=clip drawn focus heard=REMOVE in=7 limit=390,569.5,1134,310.5 name=GamePackagePatch.0.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    21 TextBlock at=1392,716.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=1366.5,707.5,157.5,66 owner=20 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  22 TextBlock at=390,697,976.5,87 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)" in=7 layout=619.68,57.6 limit=390,569.5,1134,310.5 paints=#9C9CA0FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)"
                23 DockPanel at=390,793,1134,87 by=clip in=7 limit=390,569.5,1134,310.5 role=none seen=390,793,1134,66
                  24 Button at=1366.5,803.5,157.5,66 by=clip drawn focus heard=REMOVE in=7 limit=390,569.5,1134,310.5 name=GamePackagePatch.1.Remove op paints=#55555AFF,#9C9CA0FF role=button seen=1366.5,803.5,157.5,55.5 text=Remove
                    25 TextBlock at=1392,812.5,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=1366.5,803.5,157.5,66 owner=24 paints=#9C9CA0FF role=text seen=1392,812.5,106.5,46.5 size=26 text=REMOVE
                  26 TextBlock at=390,793,976.5,87 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)" in=7 layout=623.18,57.6 limit=390,569.5,1134,310.5 paints=#9C9CA0FF role=text seen=390,793,976.5,66 size=24 text="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)"
focus 20
edge 5 up item:ButtonBar/0//
edge 5 down same
edge 5 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 5 right same
edge 20 up same
edge 20 down 24
edge 20 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 20 right same
edge 24 up 20
edge 24 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 24 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 24 right same
edge item:ButtonBar/0// up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge item:ButtonBar/0// down 5
edge item:ButtonBar/0// left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge item:ButtonBar/0// right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 up 24
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 right 24
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 down item:ButtonBar/0//
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 right 24
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
