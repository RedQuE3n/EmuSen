# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageNew-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,186.5,1260,827
title "Game Package"
case GamePackageNew
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,336,1260,648 by=panel limit=330,336,1260,647 paints=#9C9CA0FF role=none seen=330,336,1260,647
    1 DockPanel at=390,378,1140,582 by=panel limit=330,336,1260,647 role=none
      2 ButtonBar at=1395,894,135,66 by=panel limit=330,336,1260,647 paints=#9C9CA0FF role=toolbar
        3 Button at=1395,894,135,66 by=clip drawn focus heard=CLOSE limit=1395,894,135,66 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
          4 TextBlock at=1420.5,903,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,894,135,66 owner=3 paints=#9C9CA0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,378,1140,501 by=panel limit=330,336,1260,647 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=390,378,1140,501
        6 GamePackageDetail at=390,378,1134,501 by=fixed-scroll in=5 limit=390,378,1140,501 name=GamePackageDetail role=none
          7 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          8 FieldRow at=390,510,1134,171 by=fixed-scroll heard=Package in=5 limit=390,378,1140,501 paints=#9C9CA0FF role=group
            9 TextBlock at=390,510,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=5 layout=69.48,28.8 limit=390,510,1134,171 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            10 StackPanel at=390,559.5,1134,121.5 by=clip heard=Package in=5 limit=390,510,1134,171 role=none
              11 TextBlock at=390,559.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=5 layout=150.96,28.8 limit=390,510,1134,171 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Not a package yet."
              12 ButtonBar at=1276.5,615,247.5,66 by=clip in=5 limit=390,510,1134,171 paints=#9C9CA0FF role=toolbar
                13 Button at=1276.5,615,247.5,66 by=clip drawn focus heard="MAKE PACKAGE" in=5 limit=1276.5,615,247.5,66 name=GamePackageMake op paints=#55555AFF,#9C9CA0FF role=button text="Make Package"
                  14 TextBlock at=1302,624,196.5,48 box=131,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=5 layout=130.65,31.2 limit=1276.5,615,247.5,66 owner=13 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
          15 FieldRow at=390,705,1134,172.5 by=fixed-scroll heard="Soft patches" in=5 limit=390,378,1140,501 paints=#9C9CA0FF role=group
            16 TextBlock at=390,705,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=5 layout=102.91,28.8 limit=390,705,1134,172.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            17 HintText at=390,754.5,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=5 layout=727.86,48 limit=390,705,1134,172.5 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            18 StackPanel at=390,832.5,1134,45 by=clip heard="Soft patches" in=5 limit=390,705,1134,172.5 role=none
              19 HintText at=390,841.5,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Make the package first; its patches live in it." in=5 layout=299.04,24 limit=390,705,1134,172.5 name=GamePackagePatchHint paints=#86868AFF role=text size=20 text="Make the package first; its patches live in it."
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
