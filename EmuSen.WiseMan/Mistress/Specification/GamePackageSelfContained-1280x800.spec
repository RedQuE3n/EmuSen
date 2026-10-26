# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageSelfContained-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,24,840,694
title "Game Package"
case GamePackageSelfContained
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,124,840,574 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=none
    1 DockPanel at=260,152,760,530 by=panel limit=220,124,840,574 role=none
      2 StackPanel at=260,606,760,76 by=panel limit=220,124,840,574 role=none
        3 HintText at=260,606,760,24 by=panel drawn limit=220,124,840,574 name=GamePackageStatus paints=#86868AFF role=text
        4 ButtonBar at=930,638,90,44 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=toolbar
          5 Button at=930,638,90,44 by=clip drawn focus heard=CLOSE limit=930,638,90,44 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=947,644,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=930,638,90,44 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=260,152,760,444 by=panel fades=b limit=220,124,840,574 overflow=191 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=260,152,760,444
        8 StackPanel at=260,152,756,635 by=scroll in=7 limit=260,152,760,444 paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=260,152,756,444
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=260,264,756,128 by=scroll heard=Package in=7 limit=260,152,760,444 paints=#9C9CA0FF role=group
            11 TextBlock at=260,264,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=260,264,756,128 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=260,297,756,95 by=clip heard=Package in=7 limit=260,264,756,128 paints=#9C9CA0FF role=none
              13 TextBlock at=260,297,756,87 box=756,87 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game." in=7 layout=738.72,86.4 limit=260,264,756,128 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game."
          14 FieldRow at=260,408,756,197 by=scroll heard="Soft patches" in=7 limit=260,152,760,444 paints=#9C9CA0FF role=group seen=260,408,756,188
            15 TextBlock at=260,408,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=260,408,756,197 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            16 HintText at=260,441,756,48 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=260,408,756,197 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            17 StackPanel at=260,493,756,112 by=clip heard="Soft patches" in=7 limit=260,408,756,197 role=none seen=260,493,756,103
              18 StackPanel at=260,493,756,58 by=clip in=7 limit=260,408,756,197 name=GamePackagePatches role=none
                19 DockPanel at=260,493,756,58 by=clip in=7 limit=260,408,756,197 role=none
                  20 Button at=911,500,105,44 by=clip drawn focus heard=REMOVE in=7 limit=260,408,756,197 name=GamePackagePatch.0.Remove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
                    21 TextBlock at=928,506,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=7 layout=70.56,31.2 limit=911,500,105,44 owner=20 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  22 TextBlock at=260,493,651,58 box=651,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)" in=7 layout=619.68,57.6 limit=260,408,756,197 paints=#9C9CA0FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)"
              23 HintText at=260,557,756,48 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none." in=7 layout=732.64,48 limit=260,408,756,197 name=GamePackagePatchHint paints=#86868AFF role=text seen=260,557,756,39 size=20 text="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none."
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
