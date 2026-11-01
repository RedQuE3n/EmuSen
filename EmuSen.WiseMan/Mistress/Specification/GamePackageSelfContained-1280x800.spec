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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,124,840,574 by=panel limit=220,124,840,574 paints=#D4D4D4FF role=none
    1 DockPanel at=260,152,760,530 by=panel limit=220,124,840,574 role=none
      2 ButtonBar at=933,638,87,44 by=panel limit=220,124,840,574 paints=#D4D4D4FF role=toolbar
        3 Button at=933,638,87,44 by=clip drawn focus heard=CLOSE limit=933,638,87,44 name=GamePackageClose op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=950,644,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,638,87,44 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 ScrollViewer at=260,152,760,476 by=panel fades=b limit=220,124,840,574 overflow=232 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=260,152,760,476
        6 GamePackageDetail at=260,152,756,708 by=scroll in=5 limit=260,152,760,476 name=GamePackageDetail paints=#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=260,152,756,476
          7 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          8 FieldRow at=260,240,756,128 by=scroll heard=Package in=5 limit=260,152,760,476 paints=#D4D4D4FF role=group
            9 TextBlock at=260,240,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=65.81,28.8 limit=260,240,756,128 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
            10 StackPanel at=260,273,756,95 by=clip heard=Package in=5 limit=260,240,756,128 paints=#D4D4D4FF role=none
              11 TextBlock at=260,273,756,87 box=756,87 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game." in=5 layout=748.46,86.4 limit=260,240,756,128 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Packages/SNES/A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).sfc, self-contained: the game is kept in the package, copied from the library, which is only read. The folder alone is the whole game."
          12 FieldRow at=260,384,756,149 by=scroll heard="Soft patches" in=5 limit=260,152,760,476 paints=#D4D4D4FF role=group
            13 TextBlock at=260,384,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=97.3,28.8 limit=260,384,756,149 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Soft patches"
            14 HintText at=260,417,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=753.36,24 limit=260,384,756,149 paints=#8D8D91FF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            15 StackPanel at=260,445,756,88 by=clip heard="Soft patches" in=5 limit=260,384,756,149 role=none
              16 StackPanel at=260,445,756,58 by=clip in=5 limit=260,384,756,149 name=GamePackagePatches role=none
                17 DockPanel at=260,445,756,58 by=clip in=5 limit=260,384,756,149 role=none
                  18 Button at=915,452,101,44 by=clip drawn focus heard=REMOVE in=5 limit=260,384,756,149 name=GamePackagePatch.0.Remove op paints=#707070FF,#D4D4D4FF role=button text=Remove
                    19 TextBlock at=932,458,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE in=5 layout=66.33,31.2 limit=915,452,101,44 owner=18 paints=#EEEEF0FF role=text size=26 text=REMOVE
                  20 TextBlock at=260,445,655,58 box=655,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)" in=5 layout=636.98,57.6 limit=260,384,756,149 paints=#D4D4D4FF role=text size=24 text="1. A Game With A Very Long Name Indeed, Special Edition - The Director's Cut (USA, Europe) (Rev 1) (Beta) (Self-Contained).bps (BPS)"
              21 HintText at=260,509,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none." in=5 layout=740.88,24 limit=260,384,756,149 name=GamePackagePatchHint paints=#8D8D91FF role=text size=20 text="On this screen a patch is added from the IPS, UPS and BPS files beside the game in the library, and there are none."
          22 FieldRow at=260,549,756,166 by=scroll heard="Scraped as" in=5 limit=260,152,760,476 paints=#707070FF,#D4D4D4FF role=group seen=260,549,756,79
            23 TextBlock at=260,549,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Scraped as" in=5 layout=87.17,28.8 limit=260,549,756,166 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Scraped as"
            24 HintText at=260,582,756,48 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Which hashes the scraper sends: the dump's, or the patched game's first for a hack ScreenScraper lists in its own right. The saves stay the patch list's either way." in=5 layout=737.94,48 limit=260,549,756,166 paints=#8D8D91FF part role=text seen=260,582,756,46 size=20 text="Which hashes the scraper sends: the dump's, or the patched game's first for a hack ScreenScraper lists in its own right. The saves stay the patch list's either way."
focus 18
edge 3 up item:ButtonBar/0//
edge 3 down same
edge 3 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge 3 right same
edge 18 up same
edge 18 down item:ButtonBar/0//
edge 18 left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge 18 right same
edge item:ButtonBar/0// up 18
edge item:ButtonBar/0// down path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge item:ButtonBar/0// left path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0
edge item:ButtonBar/0// right same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 up item:ButtonBar/0//
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 down 3
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/1/0/0/0/5/0/2/0 right item:ButtonBar/0//
press pageup focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Remove'" title="Game Package" top=GamePackageWindow
press back top=none
