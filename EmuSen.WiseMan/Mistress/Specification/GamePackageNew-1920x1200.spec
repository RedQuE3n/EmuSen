# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageNew-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,162.5,1260,875
title "Game Package"
case GamePackageNew
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,312,1260,696 by=panel limit=330,312,1260,695 paints=#9C9CA0FF role=none seen=330,312,1260,695
    1 DockPanel at=390,354,1140,630 by=panel limit=330,312,1260,695 role=none
      2 StackPanel at=390,870,1140,114 by=panel limit=330,312,1260,695 role=none
        3 HintText at=390,870,1140,36 by=panel drawn limit=330,312,1260,695 name=GamePackageStatus paints=#86868AFF role=text
        4 ButtonBar at=1395,918,135,66 by=panel limit=330,312,1260,695 paints=#9C9CA0FF role=toolbar
          5 Button at=1395,918,135,66 by=clip drawn focus heard=CLOSE limit=1395,918,135,66 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=1420.5,927,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=1395,918,135,66 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=390,354,1140,501 by=panel limit=330,312,1260,695 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=390,354,1140,501
        8 StackPanel at=390,354,1134,501 by=fixed-scroll in=7 limit=390,354,1140,501 role=none
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=390,486,1134,171 by=fixed-scroll heard=Package in=7 limit=390,354,1140,501 paints=#9C9CA0FF role=group
            11 TextBlock at=390,486,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=390,486,1134,171 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=390,535.5,1134,121.5 by=clip heard=Package in=7 limit=390,486,1134,171 role=none
              13 TextBlock at=390,535.5,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=7 layout=150.96,28.8 limit=390,486,1134,171 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Not a package yet."
              14 ButtonBar at=1276.5,591,247.5,66 by=clip in=7 limit=390,486,1134,171 paints=#9C9CA0FF role=toolbar
                15 Button at=1276.5,591,247.5,66 by=clip drawn focus heard="MAKE PACKAGE" in=7 limit=1276.5,591,247.5,66 name=GamePackageMake op paints=#55555AFF,#9C9CA0FF role=button text="Make Package"
                  16 TextBlock at=1302,600,196.5,48 box=131,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=7 layout=130.65,31.2 limit=1276.5,591,247.5,66 owner=15 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
          17 FieldRow at=390,681,1134,172.5 by=fixed-scroll heard="Soft patches" in=7 limit=390,354,1140,501 paints=#9C9CA0FF role=group
            18 TextBlock at=390,681,1134,43.5 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=390,681,1134,172.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            19 HintText at=390,730.5,1134,72 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=390,681,1134,172.5 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            20 StackPanel at=390,808.5,1134,45 by=clip heard="Soft patches" in=7 limit=390,681,1134,172.5 role=none
              21 HintText at=390,817.5,1134,36 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Make the package first; its patches live in it." in=7 layout=299.04,24 limit=390,681,1134,172.5 name=GamePackagePatchHint paints=#86868AFF role=text size=20 text="Make the package first; its patches live in it."
focus 15
edge 5 up 15
edge 5 down same
edge 5 left same
edge 5 right same
edge 15 up same
edge 15 down 5
edge 15 left same
edge 15 right same
press pageup focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press pagedown focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press first focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press last focus="Button 'Make Package'" title="Game Package" top=GamePackageWindow
press back top=none
