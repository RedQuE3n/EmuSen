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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,124,840,574 by=panel limit=220,124,840,574 paints=#D4D4D4FF role=none
    1 DockPanel at=260,152,760,530 by=panel limit=220,124,840,574 role=none
      2 ButtonBar at=933,638,87,44 by=panel limit=220,124,840,574 paints=#D4D4D4FF role=toolbar
        3 Button at=933,638,87,44 by=clip drawn focus heard=CLOSE limit=933,638,87,44 name=GamePackageClose op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=950,644,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,638,87,44 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 ScrollViewer at=260,152,760,476 by=panel fades=b limit=220,124,840,574 overflow=439 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=260,152,760,476
        6 GamePackageDetail at=260,152,756,915 by=scroll in=5 limit=260,152,760,476 name=GamePackageDetail paints=#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=260,152,756,476
          7 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          8 FieldRow at=260,240,756,99 by=scroll heard=Package in=5 limit=260,152,760,476 paints=#D4D4D4FF role=group
            9 TextBlock at=260,240,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=65.81,28.8 limit=260,240,756,99 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
            10 StackPanel at=260,273,756,66 by=clip heard=Package in=5 limit=260,240,756,99 paints=#D4D4D4FF role=none
              11 TextBlock at=260,273,756,58 box=756,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library." in=5 layout=748.46,57.6 limit=260,240,756,99 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library."
          12 FieldRow at=260,355,756,183 by=scroll heard="Soft patches" in=5 limit=260,152,760,476 paints=#D4D4D4FF role=group
            13 TextBlock at=260,355,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=97.3,28.8 limit=260,355,756,183 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Soft patches"
            14 HintText at=260,388,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=753.36,24 limit=260,355,756,183 paints=#8D8D91FF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            15 StackPanel at=260,416,756,122 by=clip heard="Soft patches" in=5 limit=260,355,756,183 role=none
              16 StackPanel at=260,416,756,122 by=clip in=5 limit=260,355,756,183 name=GamePackagePatches role=none
                17 DockPanel at=260,416,756,58 by=clip in=5 limit=260,355,756,183 role=none
                  18 Button at=915,423,101,44 by=clip drawn focus heard=REMOVE in=5 limit=260,355,756,183 name=GamePackagePatch.0.Remove op paints=#707070FF,#D4D4D4FF role=button text=Remove
                    19 TextBlock at=932,429,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=66.33,31.2 limit=915,423,101,44 owner=18 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  20 TextBlock at=260,416,655,58 box=655,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)" in=5 layout=636.98,57.6 limit=260,355,756,183 paints=#D4D4D4FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)"
                21 DockPanel at=260,480,756,58 by=clip in=5 limit=260,355,756,183 role=none
                  22 Button at=915,487,101,44 by=clip drawn focus heard=REMOVE in=5 limit=260,355,756,183 name=GamePackagePatch.1.Remove op paints=#707070FF,#D4D4D4FF role=button text=Remove
                    23 TextBlock at=932,493,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=66.33,31.2 limit=915,487,101,44 owner=22 paints=#D4D4D4FF role=text size=26 text=REMOVE
                  24 TextBlock at=260,480,655,58 box=655,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)" in=5 layout=640.01,57.6 limit=260,355,756,183 paints=#D4D4D4FF role=text size=24 text="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)"
          25 FieldRow at=260,554,756,186 by=scroll heard="Add a patch" in=5 limit=260,152,760,476 paints=#D4D4D4FF role=group seen=260,554,756,74
            26 TextBlock at=260,554,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a patch" in=5 layout=93.74,28.8 limit=260,554,756,186 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Add a patch"
            27 HintText at=260,587,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="An IPS, UPS or BPS file, checked against the game before it is copied into the package." in=5 layout=557.36,24 limit=260,554,756,186 paints=#8D8D91FF part role=text size=20 text="An IPS, UPS or BPS file, checked against the game before it is copied into the package."
            28 WrapPanel at=260,615,756,125 by=clip heard="Add a patch" in=5 limit=260,554,756,186 name=GamePackageAdds paints=#707070FF,#D4D4D4FF role=none seen=260,615,756,13
              29 Button at=260,615,725,73 by=clip drawn focus heard="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS" in=5 limit=260,554,756,186 name=GamePackageAdd.0 op paints=#707070FF,#D4D4D4FF role=button seen=260,615,725,13 text="Add A Very Long Translation Patch Name, Version 2.1 With Every Fix Applied (English).bps"
                30 TextBlock at=277,620,691,63 box=691,63 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS" in=5 layout=685.52,62.4 limit=260,615,725,73 owner=29 paints=#D4D4D4FF role=text seen=277,620,691,8 size=26 text="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS"
focus 18
edge 3 up item:ButtonBar/0//
edge 3 down same
edge 3 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 3 right same
edge 18 up same
edge 18 down 22
edge 18 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 18 right same
edge 22 up 18
edge 22 down 29
edge 22 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 22 right same
edge 29 up 22
edge 29 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 29 left same
edge 29 right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 up 29
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 right 22
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 down 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 right item:ButtonBar/0//
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
