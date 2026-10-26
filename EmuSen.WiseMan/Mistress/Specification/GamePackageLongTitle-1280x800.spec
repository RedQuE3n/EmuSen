# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageLongTitle-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,24,840,694
title "Game Package"
case GamePackageLongTitle
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,124,840,574 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=none
    1 DockPanel at=260,152,760,530 by=panel limit=220,124,840,574 role=none
      2 StackPanel at=260,582,760,100 by=panel limit=220,124,840,574 role=none
        3 HintText at=260,582,760,48 box=760,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="Added A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1. It is named as RetroArch finds it." layout=689.92,48 limit=220,124,840,574 name=GamePackageStatus paints=#86868AFF role=text size=20 text="Added A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1. It is named as RetroArch finds it."
        4 ButtonBar at=930,638,90,44 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=toolbar
          5 Button at=930,638,90,44 by=clip drawn focus heard=CLOSE limit=930,638,90,44 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=947,644,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=930,638,90,44 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=260,152,760,420 by=panel fades=b limit=220,124,840,574 overflow=398 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=260,152,760,420
        8 StackPanel at=260,152,756,818 by=scroll in=7 limit=260,152,760,420 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=260,152,756,420
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=260,264,756,99 by=scroll heard=Package in=7 limit=260,152,760,420 paints=#9C9CA0FF role=group
            11 TextBlock at=260,264,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=260,264,756,99 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=260,297,756,66 by=clip heard=Package in=7 limit=260,264,756,99 paints=#9C9CA0FF role=none
              13 TextBlock at=260,297,756,58 box=756,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library." in=7 layout=738.72,57.6 limit=260,264,756,99 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library."
          14 FieldRow at=260,379,756,207 by=scroll heard="Soft patches" in=7 limit=260,152,760,420 paints=#9C9CA0FF role=group seen=260,379,756,193
            15 TextBlock at=260,379,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=260,379,756,207 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            16 HintText at=260,412,756,48 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=260,379,756,207 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            17 StackPanel at=260,464,756,122 by=clip heard="Soft patches" in=7 limit=260,379,756,207 role=none seen=260,464,756,108
              18 StackPanel at=260,464,756,122 by=clip in=7 limit=260,379,756,207 name=GamePackagePatches role=none seen=260,464,756,108
                19 DockPanel at=260,464,756,58 by=clip in=7 limit=260,379,756,207 role=none
                  20 Button at=911,471,105,44 by=clip drawn focus heard=REMOVE in=7 limit=260,379,756,207 name=GamePackagePatch.0.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    21 TextBlock at=928,477,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=911,471,105,44 owner=20 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  22 TextBlock at=260,464,651,58 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)" in=7 layout=619.68,57.6 limit=260,379,756,207 paints=#9C9CA0FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)"
                23 DockPanel at=260,528,756,58 by=clip in=7 limit=260,379,756,207 role=none seen=260,528,756,44
                  24 Button at=911,535,105,44 by=clip drawn focus heard=REMOVE in=7 limit=260,379,756,207 name=GamePackagePatch.1.Remove op paints=#55555AFF,#9C9CA0FF role=button seen=911,535,105,37 text=Remove
                    25 TextBlock at=928,541,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=911,535,105,44 owner=24 paints=#9C9CA0FF role=text seen=928,541,71,31 size=26 text=REMOVE
                  26 TextBlock at=260,528,651,58 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)" in=7 layout=623.18,57.6 limit=260,379,756,207 paints=#9C9CA0FF role=text seen=260,528,651,44 size=24 text="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)"
focus 20
edge 5 up item:ButtonBar/0//
edge 5 down same
edge 5 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 5 right same
edge 20 up same
edge 20 down 24
edge 20 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 20 right same
edge 24 up 20
edge 24 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge 24 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 24 right same
edge item:ButtonBar/0// up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge item:ButtonBar/0// down 5
edge item:ButtonBar/0// left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge item:ButtonBar/0// right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 up 24
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 down item:ButtonBar/0//
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 right 24
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
