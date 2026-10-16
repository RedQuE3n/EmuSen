# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-CollectionSettings-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CollectionSettings
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#FFFFFFFF role=none
    1 Grid at=0,0,1280,800 by=clip limit=0,0,1280,800 role=none
      2 MenuPanel at=0,0,1280,800 by=clip heard="Game Collection Settings" limit=0,0,1280,800 name=BigMenu role=menu
        3 HintBar at=410,742,461,48 by=clip heard="A Select, B Back, D-pad left and right Change, D-pad up and down Choose" limit=0,0,1280,800 role=group
        4 ScrollViewer at=220,133,840,432 by=clip limit=0,0,1280,800 overflow=648 paints=#FFFFFFFF role=scrollviewer viewport=220,133,840,432
          5 StackPanel at=220,133,840,1080 by=scroll in=4 limit=220,133,840,432 name=BigMenuRows paints=#55555AFF,#9C9CA0FF role=none seen=220,133,840,432
            6 LunaSwitch at=220,133,840,54 by=scroll drawn focus heard="All Games" in=4 limit=220,133,840,432 name=Autoall op paints=#9C9CA0FF role=button
            7 LunaSwitch at=220,187,840,54 by=scroll drawn focus heard=Favorites in=4 limit=220,133,840,432 name=Autofavorites op paints=#9C9CA0FF role=button
            8 LunaSwitch at=220,241,840,54 by=scroll drawn focus heard="Last Played" in=4 limit=220,133,840,432 name=Autorecent op paints=#9C9CA0FF role=button
            9 MenuRow at=220,295,840,54 by=scroll heard="No custom collections yet. Create one below." in=4 limit=220,133,840,432 role=listitem
            10 Button at=220,349,840,54 by=scroll drawn focus heard="Create New Custom Collection" in=4 limit=220,133,840,432 name=CollectionCreate op paints=#55555AFF,#9C9CA0FF role=button text="Create New Custom Collection..."
            11 Dropdown at=220,403,840,54 by=scroll drawn focus heard="Group Custom Collections" in=4 limit=220,133,840,432 name=CollectionGrouping op paints=#55555AFF,#9C9CA0FF role=combobox sideways
            12 LunaSwitch at=220,457,840,54 by=scroll drawn focus heard="Sort favorites on top for custom collections" in=4 limit=220,133,840,432 name=FavoritesFirstCustom op paints=#9C9CA0FF role=button
            13 LunaSwitch at=220,511,840,54 by=scroll drawn focus heard="Display star markings for custom collections" in=4 limit=220,133,840,432 name=StarsCustom op paints=#9C9CA0FF role=button
        14 StackPanel at=598,577,85,46 by=clip limit=0,0,1280,800 role=none
          15 Button at=598,577,85,46 by=clip drawn focus heard=Back limit=0,0,1280,800 name=BigMenuBack op paints=#55555AFF,#9C9CA0FF role=button text=Back
focus 6
edge 6 up same
edge 6 down 7
edge 6 left same
edge 6 right same
edge 7 up 6
edge 7 down 8
edge 7 left same
edge 7 right same
edge 8 up 7
edge 8 down 10
edge 8 left same
edge 8 right same
edge 10 up 8
edge 10 down 11
edge 10 left same
edge 10 right same
edge 11 up 10
edge 11 down 12
edge 12 up 11
edge 12 down 13
edge 12 left same
edge 12 right same
edge 13 up 12
edge 13 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8
edge 13 left same
edge 13 right same
edge 15 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/19
edge 15 down same
edge 15 left same
edge 15 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/11
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/12
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/13
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/14
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/15
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/16
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/17
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/19
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/19 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/19 down 15
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8 up 13
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 right same
press pageup focus="LunaSwitch Autoall (All Games)" top=CollectionSettingsWindow
press pagedown focus="LunaSwitch Autoall (All Games)" top=CollectionSettingsWindow
press first focus="LunaSwitch Autoall (All Games)" top=CollectionSettingsWindow
press last focus="LunaSwitch Autoall (All Games)" top=CollectionSettingsWindow
press back top=none
