# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageNew-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,204.5,1260,791
title "Game Package"
case GamePackageNew
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,354,1260,612 by=panel limit=330,354,1260,611 paints=#D4D4D4FF role=none seen=330,354,1260,611
    1 DockPanel at=390,396,1140,546 by=panel limit=330,354,1260,611 role=none
      2 ButtonBar at=1399.5,876,130.5,66 by=panel limit=330,354,1260,611 paints=#D4D4D4FF role=toolbar
        3 Button at=1399.5,876,130.5,66 by=clip drawn focus heard=CLOSE limit=1399.5,876,130.5,66 name=GamePackageClose op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=1425,885,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1399.5,876,130.5,66 owner=3 paints=#D4D4D4FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,396,1140,465 by=panel limit=330,354,1260,611 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=390,396,1140,465
        6 GamePackageDetail at=390,396,1134,465 by=fixed-scroll in=5 limit=390,396,1140,465 name=GamePackageDetail role=none
          7 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
          8 FieldRow at=390,528,1134,171 by=fixed-scroll heard=Package in=5 limit=390,396,1140,465 paints=#D4D4D4FF role=group
            9 TextBlock at=390,528,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=65.81,28.8 limit=390,528,1134,171 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
            10 StackPanel at=390,577.5,1134,121.5 by=clip heard=Package in=5 limit=390,528,1134,171 role=none
              11 TextBlock at=390,577.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=5 layout=144.72,28.8 limit=390,528,1134,171 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
              12 ButtonBar at=1284,633,240,66 by=clip in=5 limit=390,528,1134,171 paints=#D4D4D4FF role=toolbar
                13 Button at=1284,633,240,66 by=clip drawn focus heard="MAKE PACKAGE" in=5 limit=1284,633,240,66 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                  14 TextBlock at=1309.5,642,189,48 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=5 layout=125.29,31.2 limit=1284,633,240,66 owner=13 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
          15 FieldRow at=390,723,1134,136.5 by=fixed-scroll heard="Soft patches" in=5 limit=390,396,1140,465 paints=#D4D4D4FF role=group
            16 TextBlock at=390,723,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=97.3,28.8 limit=390,723,1134,136.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Soft patches"
            17 HintText at=390,772.5,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=753.36,24 limit=390,723,1134,136.5 paints=#8D8D91FF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            18 StackPanel at=390,814.5,1134,45 by=clip heard="Soft patches" in=5 limit=390,723,1134,136.5 role=none
              19 HintText at=390,823.5,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Make the package first; its patches live in it." in=5 layout=283.34,24 limit=390,723,1134,136.5 name=GamePackagePatchHint paints=#8D8D91FF role=text size=20 text="Make the package first; its patches live in it."
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
