# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-CollectionSettings-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case CollectionSettings
palette #000000FF #007ACCFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #6E6E6EFF #808080FF #9CDCFEFF #CD5C5CFF #D08770FF #D4D4D4FF #DAA520FF #DCDCDCFF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#FFFFFFFF role=none
    1 Grid at=0,0,1920,1200 by=clip limit=0,0,1920,1200 role=none
      2 MenuPanel at=0,0,1920,1200 by=clip heard="Game Collection Settings" limit=0,0,1920,1200 name=BigMenu role=menu
        3 HintBar at=614,1114,691,71 by=clip heard="A Select, B Back, D-pad left and right Change, D-pad up and down Choose" limit=0,0,1920,1200 role=group
        4 ScrollViewer at=330,200,1260,648 by=clip limit=0,0,1920,1200 overflow=891 paints=#FFFFFFFF role=scrollviewer viewport=330,200,1260,648
          5 StackPanel at=330,200,1260,1539 by=scroll in=4 limit=330,200,1260,648 name=BigMenuRows paints=#6E6E6EFF,#D4D4D4FF,#FFFFFFFF role=none seen=330,200,1260,648
            6 LunaSwitch at=330,200,1260,81 by=scroll drawn focus heard="All Games" in=4 limit=330,200,1260,648 name=Autoall op paints=#FFFFFFFF role=button
            7 LunaSwitch at=330,281,1260,81 by=scroll drawn focus heard=Favorites in=4 limit=330,200,1260,648 name=Autofavorites op paints=#FFFFFFFF role=button
            8 LunaSwitch at=330,362,1260,81 by=scroll drawn focus heard="Last Played" in=4 limit=330,200,1260,648 name=Autorecent op paints=#FFFFFFFF role=button
            9 MenuRow at=330,443,1260,81 by=scroll heard="No custom collections yet. Create one below." in=4 limit=330,200,1260,648 role=listitem
            10 Button at=330,524,1260,81 by=scroll drawn focus heard="Create New Custom Collection..." in=4 limit=330,200,1260,648 name=CollectionCreate op paints=#6E6E6EFF,#D4D4D4FF role=button text="Create New Custom Collection..."
            11 Dropdown at=330,605,1260,81 by=scroll drawn focus in=4 limit=330,200,1260,648 name=CollectionGrouping op paints=#6E6E6EFF,#D4D4D4FF role=combobox sideways
            12 LunaSwitch at=330,686,1260,81 by=scroll drawn focus heard="Sort favorites on top for custom collections" in=4 limit=330,200,1260,648 name=FavoritesFirstCustom op paints=#FFFFFFFF role=button
            13 LunaSwitch at=330,767,1260,81 by=scroll drawn focus heard="Display star markings for custom collections" in=4 limit=330,200,1260,648 name=StarsCustom op paints=#FFFFFFFF role=button
        14 StackPanel at=896,866,128,69 by=clip limit=0,0,1920,1200 role=none
          15 Button at=896,866,128,69 by=clip drawn focus heard=Back limit=0,0,1920,1200 name=BigMenuBack op paints=#55555AFF,#6E6E6EFF,#D4D4D4FF role=button text=Back
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
edge 15 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18
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
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/18 down 15
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8 up 13
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 up path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/8
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 down path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/10
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/1/0/0/0/9 right same
press pageup focus="LunaSwitch Autoall" top=CollectionSettingsWindow
press pagedown focus="LunaSwitch Autoall" top=CollectionSettingsWindow
press first focus="LunaSwitch Autoall" top=CollectionSettingsWindow
press last focus="LunaSwitch Autoall" top=CollectionSettingsWindow
press back top=none
