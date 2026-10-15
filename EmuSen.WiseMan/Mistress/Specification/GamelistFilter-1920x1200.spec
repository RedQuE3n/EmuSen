# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamelistFilter-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,37,1260,1041
title "Filter Gamelist"
case GamelistFilter
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 ButtonBar at=1093.5,958,436.5,66 by=panel limit=330,187,1260,861 paints=#9C9CA0FF role=toolbar
        3 Button at=1093.5,958,301.5,66 by=clip drawn focus heard="RESET ALL FILTERS" limit=1093.5,958,436.5,66 name=FilterReset op paints=#55555AFF,#9C9CA0FF role=button text="Reset All Filters"
          4 TextBlock at=1119,967,250.5,48 box=167,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL FILTERS" layout=166.53,31.2 limit=1093.5,958,301.5,66 owner=3 paints=#9C9CA0FF role=text size=26 text="RESET ALL FILTERS"
        5 Button at=1407,958,123,66 by=clip drawn focus heard=DONE limit=1093.5,958,436.5,66 name=FilterDone op paints=#55555AFF,#9C9CA0FF role=button text=Done
          6 TextBlock at=1432.5,967,72,48 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=DONE layout=47.92,31.2 limit=1407,958,123,66 owner=5 paints=#9C9CA0FF role=text size=26 text=DONE
      7 ScrollViewer at=390,229,1140,711 by=panel fades=b limit=330,187,1260,861 overflow=393 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,711
        8 StackPanel at=396,235,1128,1092 by=scroll in=7 limit=390,229,1140,711 role=none seen=396,235,1128,705
          9 FieldRow at=396,235,1128,160.5 by=scroll heard="Game Name" in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
            10 TextBlock at=396,235,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Game Name" in=7 layout=97.92,28.8 limit=396,235,1128,160.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Game Name"
            11 HintText at=396,284.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Any part of the name, in any case." in=7 layout=229.28,24 limit=396,235,1128,160.5 paints=#86868AFF part role=text size=20 text="Any part of the name, in any case."
            12 TextBox at=396,326.5,1128,69 by=clip drawn focus heard="Game Name" in=7 limit=396,235,1128,160.5 name=FilterGameName op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              13 ScrollViewer allow="a text box scrolls its own text" at=412.5,337,1095,48 in=7 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                14 TextBlock allow="a text box scrolls its own text" at=412.5,337,1095,48 box=730,32 caps=18.1987 drawn face="Barlow Condensed" heard="Any name" in=7,13 layout=85.57,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Any name"
          15 StackPanel at=396,413.5,1128,913.5 by=scroll in=7 limit=390,229,1140,711 paints=#86868AFF,#9C9CA0FF role=none seen=396,413.5,1128,526.5
            16 FieldRow at=396,413.5,1128,85.5 by=scroll heard=Rating in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
              17 TextBlock at=396,413.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Rating in=7 layout=52.03,28.8 limit=396,413.5,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Rating
              18 HintText at=396,463,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=107.14,24 limit=396,413.5,1128,85.5 name=FilterRatingNothing paints=#86868AFF role=text size=20 text="Nothing to filter"
            19 FieldRow at=396,517,1128,85.5 by=scroll heard=Developer in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
              20 TextBlock at=396,517,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Developer in=7 layout=80.83,28.8 limit=396,517,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Developer
              21 HintText at=396,566.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=107.14,24 limit=396,517,1128,85.5 name=FilterDeveloperNothing paints=#86868AFF role=text size=20 text="Nothing to filter"
            22 FieldRow at=396,620.5,1128,85.5 by=scroll heard=Publisher in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
              23 TextBlock at=396,620.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Publisher in=7 layout=76.2,28.8 limit=396,620.5,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Publisher
              24 HintText at=396,670,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=107.14,24 limit=396,620.5,1128,85.5 name=FilterPublisherNothing paints=#86868AFF role=text size=20 text="Nothing to filter"
            25 FieldRow at=396,724,1128,85.5 by=scroll heard=Genre in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
              26 TextBlock at=396,724,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Genre in=7 layout=47.16,28.8 limit=396,724,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Genre
              27 HintText at=396,773.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=107.14,24 limit=396,724,1128,85.5 name=FilterGenreNothing paints=#86868AFF role=text size=20 text="Nothing to filter"
            28 FieldRow at=396,827.5,1128,85.5 by=scroll heard=Players in=7 limit=390,229,1140,711 paints=#9C9CA0FF role=group
              29 TextBlock at=396,827.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Players in=7 layout=59.14,28.8 limit=396,827.5,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Players
              30 HintText at=396,877,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=107.14,24 limit=396,827.5,1128,85.5 name=FilterPlayersNothing paints=#86868AFF role=text size=20 text="Nothing to filter"
            31 FieldRow at=396,931,1128,85.5 by=scroll heard=Favorite in=7 limit=390,229,1140,711 paints=#86868AFF,#9C9CA0FF role=group seen=396,931,1128,9
              32 TextBlock at=396,931,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Favorite in=7 layout=66.24,28.8 limit=396,931,1128,85.5 name=PART_Label paints=#9C9CA0FF part role=text seen=396,931,1128,9 size=24 text=Favorite
focus 12
edge 3 up 12
edge 3 down same
edge 3 left same
edge 3 right 5
edge 5 up 12
edge 5 down same
edge 5 left 3
edge 5 right same
edge 12 up same
edge 12 down 3
edge 12 left same
edge 12 right 5
press pageup focus="TextBox FilterGameName" title="Filter Gamelist" top=GamelistFilterWindow
press pagedown focus="TextBox FilterGameName" title="Filter Gamelist" top=GamelistFilterWindow
press first focus="TextBox FilterGameName" title="Filter Gamelist" top=GamelistFilterWindow
press last focus="TextBox FilterGameName" title="Filter Gamelist" top=GamelistFilterWindow
press back focus="Dropdown GamelistJumpTo" top=GameOptionsWindow
