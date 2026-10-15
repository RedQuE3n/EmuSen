# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-GameOptions-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GameOptions
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#FFFFFFFF role=none
    1 MenuPanel at=0,0,1920,1200 by=clip heard="Gamelist Options" limit=0,0,1920,1200 name=GameOptionsMenu role=menu
      2 HintBar at=366,1114,1188,71 by=clip heard="View Close (Cancel), A Select, B Close (Apply), D-pad left and right Change Value, D-pad up and down Choose" limit=0,0,1920,1200 role=group
      3 ScrollViewer at=330,238,1260,810 by=clip limit=0,0,1920,1200 overflow=81 paints=#FFFFFFFF role=scrollviewer viewport=330,238,1260,810
        4 StackPanel at=330,238,1260,891 by=scroll in=3 limit=330,238,1260,810 paints=#55555AFF,#9C9CA0FF role=none seen=330,238,1260,810
          5 Dropdown at=330,238,1260,81 by=scroll drawn focus heard="Jump To..." in=3 limit=330,238,1260,810 name=GamelistJumpTo op paints=#55555AFF,#9C9CA0FF role=combobox sideways
          6 Dropdown at=330,319,1260,81 by=scroll drawn focus heard="Sort Games By" in=3 limit=330,238,1260,810 name=GamelistSortBy op paints=#55555AFF,#9C9CA0FF role=combobox sideways
          7 Button at=330,400,1260,81 by=scroll drawn focus heard="Filter Gamelist" in=3 limit=330,238,1260,810 name=GamelistFilterButton op paints=#55555AFF,#9C9CA0FF role=button text="Filter Gamelist"
          8 Button at=330,481,1260,81 by=scroll drawn focus heard=Search... in=3 limit=330,238,1260,810 name=GameOption_Search op paints=#55555AFF,#9C9CA0FF role=button text=Search...
          9 Button at=330,562,1260,81 by=scroll drawn focus heard="Add to Favourites" in=3 limit=330,238,1260,810 name=GameOption_AddtoFavourites op paints=#55555AFF,#9C9CA0FF role=button text="Add to Favourites"
          10 Button at=330,643,1260,81 by=scroll drawn focus heard="Edit This Game's Metadata" in=3 limit=330,238,1260,810 name=GameOption_EditThisGamesMetadata op paints=#55555AFF,#9C9CA0FF role=button text="Edit This Game's Metadata"
          11 Button at=330,724,1260,81 by=scroll drawn focus heard="Crop This Game..." in=3 limit=330,238,1260,810 name=GameOption_CropThisGame op paints=#55555AFF,#9C9CA0FF role=button text="Crop This Game..."
          12 Button at=330,805,1260,81 by=scroll drawn focus heard="Use Another Game's Cover..." in=3 limit=330,238,1260,810 name=GameOption_UseAnotherGamesCover op paints=#55555AFF,#9C9CA0FF role=button text="Use Another Game's Cover..."
          13 Button at=330,886,1260,81 by=scroll drawn focus heard="Scrape This Game..." in=3 limit=330,238,1260,810 name=GameOption_ScrapeThisGame op paints=#55555AFF,#9C9CA0FF role=button text="Scrape This Game..."
          14 Button at=330,967,1260,81 by=scroll drawn focus heard="Find by Name..." in=3 limit=330,238,1260,810 name=GameOption_FindbyName op paints=#55555AFF,#9C9CA0FF role=button text="Find by Name..."
focus 5
edge 5 up same
edge 5 down 6
edge 6 up 5
edge 6 down 7
edge 7 up 6
edge 7 down 8
edge 7 left same
edge 7 right same
edge 8 up 7
edge 8 down 9
edge 8 left same
edge 8 right same
edge 9 up 8
edge 9 down 10
edge 9 left same
edge 9 right same
edge 10 up 9
edge 10 down 11
edge 10 left same
edge 10 right same
edge 11 up 10
edge 11 down 12
edge 11 left same
edge 11 right same
edge 12 up 11
edge 12 down 13
edge 12 left same
edge 12 right same
edge 13 up 12
edge 13 down 14
edge 13 left same
edge 13 right same
edge 14 up 13
edge 14 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10/0
edge 14 left same
edge 14 right same
press pageup focus="Dropdown GamelistJumpTo (Jump To...)" top=GameOptionsWindow
press pagedown focus="Dropdown GamelistJumpTo (Jump To...)" top=GameOptionsWindow
press first focus="Dropdown GamelistJumpTo (Jump To...)" top=GameOptionsWindow
press last focus="Dropdown GamelistJumpTo (Jump To...)" top=GameOptionsWindow
press back top=none
