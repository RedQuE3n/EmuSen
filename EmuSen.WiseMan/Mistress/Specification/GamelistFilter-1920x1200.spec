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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 ButtonBar at=1108.5,958,421.5,66 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=toolbar
        3 Button at=1108.5,958,289.5,66 by=clip drawn focus heard="RESET ALL FILTERS" limit=1108.5,958,421.5,66 name=FilterReset op paints=#707070FF,#D4D4D4FF role=button text="Reset All Filters"
          4 TextBlock at=1134,967,238.5,48 box=159,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET ALL FILTERS" layout=158.03,31.2 limit=1108.5,958,289.5,66 owner=3 paints=#D4D4D4FF role=text size=26 text="RESET ALL FILTERS"
        5 Button at=1410,958,120,66 by=clip drawn focus heard=DONE limit=1108.5,958,421.5,66 name=FilterDone op paints=#707070FF,#D4D4D4FF role=button text=Done
          6 TextBlock at=1435.5,967,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=DONE layout=45.08,31.2 limit=1410,958,120,66 owner=5 paints=#D4D4D4FF role=text size=26 text=DONE
      7 ScrollViewer at=390,229,1140,711 by=panel fades=b limit=330,187,1260,861 overflow=393 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,711
        8 StackPanel at=396,235,1128,1092 by=scroll in=7 limit=390,229,1140,711 role=none seen=396,235,1128,705
          9 FieldRow at=396,235,1128,160.5 by=scroll heard="Game Name" in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            10 TextBlock at=396,235,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Game Name" in=7 layout=94.1,28.8 limit=396,235,1128,160.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Game Name"
            11 HintText at=396,284.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Any part of the name, in any case." in=7 layout=220.36,24 limit=396,235,1128,160.5 paints=#8D8D91FF part role=text size=20 text="Any part of the name, in any case."
            12 TextBox at=396,326.5,1128,69 by=clip drawn focus heard="Game Name" in=7 limit=396,235,1128,160.5 name=FilterGameName op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              13 ScrollViewer allow="a text box scrolls its own text" at=412.5,337,1095,48 in=7 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                14 TextBlock allow="a text box scrolls its own text" at=412.5,337,1095,48 box=730,32 caps=18.1987 drawn face="Barlow Condensed" heard="Any name" in=7,13 layout=83.54,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Any name"
          15 StackPanel at=396,413.5,1128,913.5 by=scroll in=7 limit=390,229,1140,711 paints=#8D8D91FF,#D4D4D4FF role=none seen=396,413.5,1128,526.5
            16 FieldRow at=396,413.5,1128,85.5 by=scroll heard=Rating in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
              17 TextBlock at=396,413.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Rating in=7 layout=48.82,28.8 limit=396,413.5,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Rating
              18 HintText at=396,463,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=101.2,24 limit=396,413.5,1128,85.5 name=FilterRatingNothing paints=#8D8D91FF role=text size=20 text="Nothing to filter"
            19 FieldRow at=396,517,1128,85.5 by=scroll heard=Developer in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
              20 TextBlock at=396,517,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Developer in=7 layout=77.28,28.8 limit=396,517,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Developer
              21 HintText at=396,566.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=101.2,24 limit=396,517,1128,85.5 name=FilterDeveloperNothing paints=#8D8D91FF role=text size=20 text="Nothing to filter"
            22 FieldRow at=396,620.5,1128,85.5 by=scroll heard=Publisher in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
              23 TextBlock at=396,620.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Publisher in=7 layout=72.14,28.8 limit=396,620.5,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Publisher
              24 HintText at=396,670,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=101.2,24 limit=396,620.5,1128,85.5 name=FilterPublisherNothing paints=#8D8D91FF role=text size=20 text="Nothing to filter"
            25 FieldRow at=396,724,1128,85.5 by=scroll heard=Genre in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
              26 TextBlock at=396,724,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Genre in=7 layout=45.29,28.8 limit=396,724,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Genre
              27 HintText at=396,773.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=101.2,24 limit=396,724,1128,85.5 name=FilterGenreNothing paints=#8D8D91FF role=text size=20 text="Nothing to filter"
            28 FieldRow at=396,827.5,1128,85.5 by=scroll heard=Players in=7 limit=390,229,1140,711 paints=#D4D4D4FF role=group
              29 TextBlock at=396,827.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Players in=7 layout=56.98,28.8 limit=396,827.5,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Players
              30 HintText at=396,877,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Nothing to filter" in=7 layout=101.2,24 limit=396,827.5,1128,85.5 name=FilterPlayersNothing paints=#8D8D91FF role=text size=20 text="Nothing to filter"
            31 FieldRow at=396,931,1128,85.5 by=scroll heard=Favorite in=7 limit=390,229,1140,711 paints=#8D8D91FF,#D4D4D4FF role=group seen=396,931,1128,9
              32 TextBlock at=396,931,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Favorite in=7 layout=61.06,28.8 limit=396,931,1128,85.5 name=PART_Label paints=#D4D4D4FF part role=text seen=396,931,1128,9 size=24 text=Favorite
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
press back focus="Button 'Filter Gamelist'" top=GamelistOptionsWindow
