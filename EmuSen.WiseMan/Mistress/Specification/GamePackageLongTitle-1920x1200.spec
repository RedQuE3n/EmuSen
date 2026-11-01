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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 ButtonBar at=1399.5,958,130.5,66 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=toolbar
        3 Button at=1399.5,958,130.5,66 by=clip drawn focus heard=CLOSE limit=1399.5,958,130.5,66 name=GamePackageClose op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=1425,967,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1399.5,958,130.5,66 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,229,1140,714 by=panel fades=b limit=330,187,1260,861 overflow=658.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,714
        6 GamePackageDetail at=390,229,1134,1372.5 by=scroll in=5 limit=390,229,1140,714 name=GamePackageDetail paints=#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=390,229,1134,714
          7 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          8 FieldRow at=390,361,1134,148.5 by=scroll heard=Package in=5 limit=390,229,1140,714 paints=#D4D4D4FF role=group
            9 TextBlock at=390,361,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=65.81,28.8 limit=390,361,1134,148.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
            10 StackPanel at=390,410.5,1134,99 by=clip heard=Package in=5 limit=390,361,1134,148.5 paints=#D4D4D4FF role=none
              11 TextBlock at=390,410.5,1134,87 box=756,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library." in=5 layout=748.46,57.6 limit=390,361,1134,148.5 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).sfc, linked to the game where it is in the library."
          12 FieldRow at=390,533.5,1134,274.5 by=scroll heard="Soft patches" in=5 limit=390,229,1140,714 paints=#D4D4D4FF role=group
            13 TextBlock at=390,533.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=97.3,28.8 limit=390,533.5,1134,274.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Soft patches"
            14 HintText at=390,583,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=753.36,24 limit=390,533.5,1134,274.5 paints=#8D8D91FF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            15 StackPanel at=390,625,1134,183 by=clip heard="Soft patches" in=5 limit=390,533.5,1134,274.5 role=none
              16 StackPanel at=390,625,1134,183 by=clip in=5 limit=390,533.5,1134,274.5 name=GamePackagePatches role=none
                17 DockPanel at=390,625,1134,87 by=clip in=5 limit=390,533.5,1134,274.5 role=none
                  18 Button at=1372.5,635.5,151.5,66 by=clip drawn focus heard=REMOVE in=5 limit=390,533.5,1134,274.5 name=GamePackagePatch.0.Remove op paints=#707070FF,#D4D4D4FF role=button text=Remove
                    19 TextBlock at=1398,644.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=66.33,31.2 limit=1372.5,635.5,151.5,66 owner=18 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  20 TextBlock at=390,625,982.5,87 box=655,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)" in=5 layout=636.98,57.6 limit=390,533.5,1134,274.5 paints=#D4D4D4FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips (IPS)"
                21 DockPanel at=390,721,1134,87 by=clip in=5 limit=390,533.5,1134,274.5 role=none
                  22 Button at=1372.5,731.5,151.5,66 by=clip drawn focus heard=REMOVE in=5 limit=390,533.5,1134,274.5 name=GamePackagePatch.1.Remove op paints=#707070FF,#D4D4D4FF role=button text=Remove
                    23 TextBlock at=1398,740.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=66.33,31.2 limit=1372.5,731.5,151.5,66 owner=22 paints=#D4D4D4FF role=text size=26 text=REMOVE
                  24 TextBlock at=390,721,982.5,87 box=655,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)" in=5 layout=640.01,57.6 limit=390,533.5,1134,274.5 paints=#D4D4D4FF role=text size=24 text="2. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta).ips1 (IPS)"
          25 FieldRow at=390,832,1134,279 by=scroll heard="Add a patch" in=5 limit=390,229,1140,714 paints=#D4D4D4FF role=group seen=390,832,1134,111
            26 TextBlock at=390,832,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a patch" in=5 layout=93.74,28.8 limit=390,832,1134,279 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Add a patch"
            27 HintText at=390,881.5,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="An IPS, UPS or BPS file, checked against the game before it is copied into the package." in=5 layout=557.36,24 limit=390,832,1134,279 paints=#8D8D91FF part role=text size=20 text="An IPS, UPS or BPS file, checked against the game before it is copied into the package."
            28 WrapPanel at=390,923.5,1134,187.5 by=clip heard="Add a patch" in=5 limit=390,832,1134,279 name=GamePackageAdds paints=#707070FF,#D4D4D4FF role=none seen=390,923.5,1134,19.5
              29 Button at=390,923.5,1087.5,109.5 by=clip drawn focus heard="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS" in=5 limit=390,832,1134,279 name=GamePackageAdd.0 op paints=#707070FF,#D4D4D4FF role=button seen=390,923.5,1087.5,19.5 text="Add A Very Long Translation Patch Name, Version 2.1 With Every Fix Applied (English).bps"
                30 TextBlock at=415.5,931,1036.5,94.5 box=691,63 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS" in=5 layout=685.52,62.4 limit=390,923.5,1087.5,109.5 owner=29 paints=#D4D4D4FF role=text seen=415.5,931,1036.5,12 size=26 text="ADD A VERY LONG TRANSLATION PATCH NAME, VERSION 2.1 WITH EVERY FIX APPLIED (ENGLISH).BPS"
focus 18
edge 3 up 29
edge 3 down item:ButtonBar/0//
edge 3 left 29
edge 3 right same
edge 18 up same
edge 18 down 22
edge 18 left 29
edge 18 right same
edge 22 up 18
edge 22 down 29
edge 22 left 29
edge 22 right same
edge 29 up 22
edge 29 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1
edge 29 left same
edge 29 right 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 up 29
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/3/0/2/0/1 right 22
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
