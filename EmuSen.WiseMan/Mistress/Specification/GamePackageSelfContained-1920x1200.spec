# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageSelfContained-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,37,1260,1041
title "Game Package"
case GamePackageSelfContained
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 StackPanel at=390,910,1140,114 by=panel limit=330,187,1260,861 role=none
        3 HintText at=390,910,1140,36 by=panel drawn limit=330,187,1260,861 name=GamePackageStatus paints=#86868AFF role=text
        4 ButtonBar at=1395,958,135,66 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=toolbar
          5 Button at=1395,958,135,66 by=clip drawn focus heard=CLOSE limit=1395,958,135,66 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=1420.5,967,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,958,135,66 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=390,229,1140,666 by=panel fades=b limit=330,187,1260,861 overflow=286.5 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,666
        8 StackPanel at=390,229,1134,952.5 by=scroll in=7 limit=390,229,1140,666 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=390,229,1134,666
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=390,397,1134,192 by=scroll heard=Package in=7 limit=390,229,1140,666 paints=#9C9CA0FF role=group
            11 TextBlock at=390,397,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=390,397,1134,192 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=390,446.5,1134,142.5 by=clip heard=Package in=7 limit=390,397,1134,192 paints=#9C9CA0FF role=none
              13 TextBlock at=390,446.5,1134,130.5 box=756,87 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game." in=7 layout=738.72,86.4 limit=390,397,1134,192 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game."
          14 FieldRow at=390,613,1134,295.5 by=scroll heard="Soft patches" in=7 limit=390,229,1140,666 paints=#9C9CA0FF role=group seen=390,613,1134,282
            15 TextBlock at=390,613,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=390,613,1134,295.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            16 HintText at=390,662.5,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=390,613,1134,295.5 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            17 StackPanel at=390,740.5,1134,168 by=clip heard="Soft patches" in=7 limit=390,613,1134,295.5 role=none seen=390,740.5,1134,154.5
              18 StackPanel at=390,740.5,1134,87 by=clip in=7 limit=390,613,1134,295.5 name=GamePackagePatches role=none
                19 DockPanel at=390,740.5,1134,87 by=clip in=7 limit=390,613,1134,295.5 role=none
                  20 Button at=1366.5,751,157.5,66 by=clip drawn focus heard=REMOVE in=7 limit=390,613,1134,295.5 name=GamePackagePatch.0.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    21 TextBlock at=1392,760,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=1366.5,751,157.5,66 owner=20 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  22 TextBlock at=390,740.5,976.5,87 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)" in=7 layout=619.68,57.6 limit=390,613,1134,295.5 paints=#9C9CA0FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)"
              23 HintText at=390,836.5,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none." in=7 layout=732.64,48 limit=390,613,1134,295.5 name=GamePackagePatchHint paints=#86868AFF role=text seen=390,836.5,1134,58.5 size=20 text="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none."
focus 20
edge 5 up item:ButtonBar/0//
edge 5 down same
edge 5 left same
edge 5 right same
edge 20 up same
edge 20 down item:ButtonBar/0//
edge 20 left same
edge 20 right same
edge item:ButtonBar/0// up 20
edge item:ButtonBar/0// down 5
edge item:ButtonBar/0// left same
edge item:ButtonBar/0// right same
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
