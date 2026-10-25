# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePackageNew-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,108.5,840,583
title "Game Package"
case GamePackageNew
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,208,840,463 by=panel limit=220,208,840,463 paints=#9C9CA0FF role=none
    1 DockPanel at=260,236,760,419 by=panel limit=220,208,840,463 role=none
      2 StackPanel at=260,579,760,76 by=panel limit=220,208,840,463 role=none
        3 HintText at=260,579,760,24 by=panel drawn limit=220,208,840,463 name=GamePackageStatus paints=#86868AFF role=text
        4 ButtonBar at=930,611,90,44 by=panel limit=220,208,840,463 paints=#9C9CA0FF role=toolbar
          5 Button at=930,611,90,44 by=clip drawn focus heard=CLOSE limit=930,611,90,44 name=GamePackageClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            6 TextBlock at=947,617,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=930,611,90,44 owner=5 paints=#9C9CA0FF role=text size=26 text=CLOSE
      7 ScrollViewer at=260,236,760,333 by=panel limit=220,208,840,463 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=260,236,760,333
        8 StackPanel at=260,236,756,333 by=fixed-scroll in=7 limit=260,236,760,333 role=none
          9 HintText name=GamePackageAbout paints=#86868AFF role=text volatile="its words name the run's own folders"
          10 FieldRow at=260,324,756,114 by=fixed-scroll heard=Package in=7 limit=260,236,760,333 paints=#9C9CA0FF role=group
            11 TextBlock at=260,324,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=7 layout=69.48,28.8 limit=260,324,756,114 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
            12 StackPanel at=260,357,756,81 by=clip heard=Package in=7 limit=260,324,756,114 role=none
              13 TextBlock at=260,357,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=7 layout=150.96,28.8 limit=260,324,756,114 name=GamePackagePlace paints=#9C9CA0FF role=text size=24 text="Not a package yet."
              14 ButtonBar at=851,394,165,44 by=clip in=7 limit=260,324,756,114 paints=#9C9CA0FF role=toolbar
                15 Button at=851,394,165,44 by=clip drawn focus heard="MAKE PACKAGE" in=7 limit=851,394,165,44 name=GamePackageMake op paints=#55555AFF,#9C9CA0FF role=button text="Make Package"
                  16 TextBlock at=868,400,131,32 box=131,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=7 layout=130.65,31.2 limit=851,394,165,44 owner=15 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
          17 FieldRow at=260,454,756,115 by=fixed-scroll heard="Soft patches" in=7 limit=260,236,760,333 paints=#9C9CA0FF role=group
            18 TextBlock at=260,454,756,29 box=756,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Soft patches" in=7 layout=102.91,28.8 limit=260,454,756,115 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Soft patches"
            19 HintText at=260,487,756,48 box=756,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original." in=7 layout=727.86,48 limit=260,454,756,115 paints=#86868AFF part role=text size=20 text="Applied in order each time the game is loaded; the game's file keeps every byte, so its hashes still name the original."
            20 StackPanel at=260,539,756,30 by=clip heard="Soft patches" in=7 limit=260,454,756,115 role=none
              21 HintText at=260,545,756,24 box=756,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Make the package first; its patches live in it." in=7 layout=299.04,24 limit=260,454,756,115 name=GamePackagePatchHint paints=#86868AFF role=text size=20 text="Make the package first; its patches live in it."
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
