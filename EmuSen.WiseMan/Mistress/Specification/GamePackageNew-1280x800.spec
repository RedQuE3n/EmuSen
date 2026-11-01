# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageNew-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,136.5,840,527
title "Game Package"
case GamePackageNew
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,236,840,407 by=panel limit=220,236,840,407 paints=#D4D4D4FF role=none
    1 DockPanel at=260,264,760,363 by=panel limit=220,236,840,407 role=none
      2 ButtonBar at=933,583,87,44 by=panel limit=220,236,840,407 paints=#D4D4D4FF role=toolbar
        3 Button at=933,583,87,44 by=clip drawn focus heard=CLOSE limit=933,583,87,44 name=GamePackageClose op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=950,589,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=933,583,87,44 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 ScrollViewer at=260,264,760,309 by=panel limit=220,236,840,407 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=260,264,760,309
        6 GamePackageDetail at=260,264,756,309 by=fixed-scroll in=5 limit=260,264,760,309 name=GamePackageDetail role=none
          7 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          8 FieldRow at=260,352,756,114 by=fixed-scroll heard=Package in=5 limit=260,264,760,309 paints=#D4D4D4FF role=group
            9 TextBlock at=260,352,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=65.81,28.8 limit=260,352,756,114 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
            10 StackPanel at=260,385,756,81 by=clip heard=Package in=5 limit=260,352,756,114 role=none
              11 TextBlock at=260,385,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=5 layout=144.72,28.8 limit=260,352,756,114 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
              12 ButtonBar at=856,422,160,44 by=clip in=5 limit=260,352,756,114 paints=#D4D4D4FF role=toolbar
                13 Button at=856,422,160,44 by=clip drawn focus heard="MAKE PACKAGE" in=5 limit=856,422,160,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                  14 TextBlock at=873,428,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=5 layout=125.29,31.2 limit=856,422,160,44 owner=13 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
          15 FieldRow at=260,482,756,91 by=fixed-scroll heard="Soft patches" in=5 limit=260,264,760,309 paints=#D4D4D4FF role=group
            16 TextBlock at=260,482,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=97.3,28.8 limit=260,482,756,91 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Soft patches"
            17 HintText at=260,515,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=753.36,24 limit=260,482,756,91 paints=#8D8D91FF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            18 StackPanel at=260,543,756,30 by=clip heard="Soft patches" in=5 limit=260,482,756,91 role=none
              19 HintText at=260,549,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Make the package first; its patches live in it." in=5 layout=283.34,24 limit=260,482,756,91 name=GamePackagePatchHint paints=#8D8D91FF role=text size=20 text="Make the package first; its patches live in it."
focus 13
edge 3 up 13
edge 3 down same
edge 3 left same
edge 3 right same
edge 13 up same
edge 13 down 3
edge 13 left same
edge 13 right same
press pageup focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press back top=none
