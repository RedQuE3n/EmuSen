# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CheatDatabase-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 128,24,1024,668
title "Cheat Database"
footer "EmuSen ships no cheat data. Point this at a cheat folder you already have - a RetroArch cheats folder works as-is - or download one. Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database), licensed CC BY-SA 4.0. The codes themselves were collected from community sources, substantially GameHacking.org - credit belongs to their original authors. EmuSen ships none of this data; it is downloaded to your machine at your request."
case CheatDatabase
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=128,124,1024,440 by=panel limit=128,124,1024,440 paints=#D4D4D4FF role=none
    1 DockPanel at=152,136,976,412 by=panel limit=128,124,1024,440 role=none
      2 Grid at=152,140,976,46 by=panel limit=128,124,1024,440 role=none
        3 TextBlock at=152,148,97,29 box=97,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Cheat Folder" layout=96.53,28.8 limit=128,124,1024,440 paints=#D4D4D4FF role=text size=24 text="Cheat Folder"
        4 PathPickerRow at=265,140,863,46 by=panel heard="Choose a cheat folder (a RetroArch cheats folder works as-is)" limit=128,124,1024,440 name=DirectoryPicker paints=#D4D4D4FF role=group
          5 TextBox heard="Choose a cheat folder (a RetroArch cheats folder works as-is)" name=PART_Path op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF part role=edit volatile="its words name the run's own folders"
      6 Border at=152,409,976,139 by=panel limit=128,124,1024,440 role=none
        7 Grid at=152,409,976,139 by=panel limit=128,124,1024,440 role=none
          8 TextBlock live=polite name=StatusText paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
          9 ButtonBar at=258,504,765,44 by=panel limit=128,124,1024,440 paints=#D4D4D4FF role=toolbar
            10 Button at=258,504,173,44 by=clip drawn focus heard="ACTIVE CHEATS..." limit=258,504,765,44 name=ActiveCheatsButton op paints=#707070FF,#D4D4D4FF role=button text="Active Cheats..."
              11 TextBlock at=275,510,139,32 box=139,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="ACTIVE CHEATS..." layout=138.94,31.2 limit=258,504,173,44 owner=10 paints=#D4D4D4FF role=text size=26 text="ACTIVE CHEATS..."
            12 Button at=439,504,217,44 by=clip drawn focus heard="PRUNE UNSUPPORTED" limit=258,504,765,44 name=PruneButton op paints=#707070FF,#D4D4D4FF role=button text="Prune Unsupported"
              13 TextBlock at=456,510,183,32 box=183,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PRUNE UNSUPPORTED" layout=182.65,31.2 limit=439,504,217,44 owner=12 paints=#D4D4D4FF role=text size=26 text="PRUNE UNSUPPORTED"
            14 Button at=664,504,264,44 by=clip drawn focus heard="DOWNLOAD FROM LIBRETRO" limit=258,504,765,44 name=DownloadButton op paints=#707070FF,#D4D4D4FF role=button text="Download from libretro"
              15 TextBlock at=681,510,230,32 box=230,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="DOWNLOAD FROM LIBRETRO" layout=229.42,31.2 limit=664,504,264,44 owner=14 paints=#D4D4D4FF role=text size=26 text="DOWNLOAD FROM LIBRETRO"
            16 Button at=936,504,87,44 by=clip drawn focus heard=CLOSE limit=258,504,765,44 op paints=#707070FF,#D4D4D4FF role=button text=Close
              17 TextBlock at=953,510,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=936,504,87,44 owner=16 paints=#D4D4D4FF role=text size=26 text=CLOSE
      18 Grid at=152,198,976,199 by=panel limit=128,124,1024,440 role=none
        19 DockPanel at=152,198,480,199 by=panel limit=128,124,1024,440 role=none
          20 TextBlock at=152,198,480,29 box=480,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard=Installed layout=65.52,28.8 limit=128,124,1024,440 paints=#D4D4D4FF role=text size=24 text=Installed
          21 LunaList at=152,231,480,166 by=panel heard="Systems with cheat files" limit=128,124,1024,440 name=SystemsList paints=#FFFFFFFF role=list
            22 ScrollViewer at=152,231,480,166 fades=b name=PART_ScrollViewer overflow=296 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=152,231,480,166
              23 ListBoxItem at=152,231,480,42 by=scroll drawn focus heard="Nintendo - Game Boy  (1)" in=22 limit=152,231,480,166 op paints=#333337FF,#D4D4D4FF role=listitem text="Nintendo - Game Boy  (1)"
                24 TextBlock at=162,237,460,29 box=460,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Nintendo - Game Boy  (1)" in=22 layout=190.54,28.8 limit=152,231,480,42 owner=23 paints=#D4D4D4FF role=text size=24 text="Nintendo - Game Boy  (1)"
              25 ListBoxItem at=152,273,480,42 by=scroll drawn focus heard="Nintendo - Nintendo Entertainment System  (2)" in=22 limit=152,231,480,166 op paints=#333337FF,#D4D4D4FF role=listitem text="Nintendo - Nintendo Entertainment System  (2)"
                26 TextBlock at=162,279,460,29 box=460,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Nintendo - Nintendo Entertainment System  (2)" in=22 layout=361.39,28.8 limit=152,273,480,42 owner=25 paints=#D4D4D4FF role=text size=24 text="Nintendo - Nintendo Entertainment System  (2)"
              27 ListBoxItem at=152,315,480,42 by=scroll drawn focus heard="Nintendo - Super Nintendo Entertainment System  (4)" in=22 limit=152,231,480,166 op paints=#333337FF,#D4D4D4FF role=listitem text="Nintendo - Super Nintendo Entertainment System  (4)"
                28 TextBlock at=162,321,460,29 box=460,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Nintendo - Super Nintendo Entertainment System  (4)" in=22 layout=411.89,28.8 limit=152,315,480,42 owner=27 paints=#EEEEF0FF role=text size=24 text="Nintendo - Super Nintendo Entertainment System  (4)"
              29 ListBoxItem at=152,357,480,42 by=scroll drawn focus heard="Nintendo - Game Boy Color  (not downloaded)" in=22 limit=152,231,480,166 op paints=#333337FF,#D4D4D4FF role=listitem seen=152,357,480,40 text="Nintendo - Game Boy Color  (not downloaded)"
                30 TextBlock at=162,363,460,29 box=460,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Nintendo - Game Boy Color  (not downloaded)" in=22 layout=353.26,28.8 limit=152,357,480,42 owner=29 paints=#D4D4D4FF role=text size=24 text="Nintendo - Game Boy Color  (not downloaded)"
        31 DockPanel at=648,198,480,199 by=panel limit=128,124,1024,440 role=none
          32 TextBlock at=648,198,480,29 box=480,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Games  (4)" layout=82.1,28.8 limit=128,124,1024,440 name=GamesHeaderText paints=#D4D4D4FF role=text size=24 text="Games  (4)"
          33 Grid at=648,231,480,46 by=panel limit=128,124,1024,440 role=none
            34 FilterBar at=648,231,221,46 by=panel limit=128,124,1024,440 name=GameFilter paints=#D4D4D4FF role=group
              35 TextBox at=648,231,221,46 drawn focus heard="Filter games..." name=PART_Search op paints=#0E0E10FF,#707070FF,#D4D4D4FF part role=edit
                36 ScrollViewer allow="a text box scrolls its own text" at=659,238,199,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                  37 TextBlock allow="a text box scrolls its own text" at=659,238,199,32 box=199,32 caps=18.1987 drawn face="Barlow Condensed" heard="Filter games..." in=36 layout=118.85,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Filter games..."
            38 Button at=877,232,251,44 by=panel drawn heard="LOAD INTO ACTIVE CHEATS" limit=128,124,1024,440 name=LoadGameButton off paints=#707070FF,#D4D4D4FF role=button text="Load into Active Cheats"
              39 TextBlock at=894,238,217,32 box=217,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="LOAD INTO ACTIVE CHEATS" layout=216.11,31.2 limit=877,232,251,44 off owner=38 paints=#707070FF role=text size=26 text="LOAD INTO ACTIVE CHEATS"
          40 ListBox at=648,283,480,114 by=panel heard="Games in the selected system" limit=128,124,1024,440 name=GamesList paints=#FFFFFFFF role=list
            41 ScrollViewer at=648,283,480,114 fades=b name=PART_ScrollViewer overflow=104 paints=#707070FF,#FFFFFFFF part role=scrollviewer viewport=648,283,480,114
              42 ListBoxItem at=648,283,480,67 by=scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)" in=41 limit=648,283,480,114 op paints=#333337FF,#D4D4D4FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)"
                43 TextBlock at=658,287,460,58 box=460,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)" in=41 layout=423.53,57.6 limit=648,283,480,67 owner=42 paints=#D4D4D4FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1)"
              44 ListBoxItem at=648,350,480,42 by=scroll drawn focus heard="Aurora Drift (USA)" in=41 limit=648,283,480,114 op paints=#333337FF,#D4D4D4FF role=listitem text="Aurora Drift (USA)"
                45 TextBlock at=658,356,460,29 box=460,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Aurora Drift (USA)" in=41 layout=135.12,28.8 limit=648,350,480,42 owner=44 paints=#D4D4D4FF role=text size=24 text="Aurora Drift (USA)"
              46 ListBoxItem at=648,392,480,42 by=scroll drawn focus heard="Brass Lantern (Europe)" in=41 limit=648,283,480,114 op paints=#333337FF,#D4D4D4FF role=listitem seen=648,392,480,5 text="Brass Lantern (Europe)"
focus 5
edge 5 up same
edge 5 down 23
edge 5 left same
edge 5 right same
edge 10 up 29
edge 10 down same
edge 10 left same
edge 10 right 12
edge 12 up 29
edge 12 down same
edge 12 left 10
edge 12 right 14
edge 14 up "row:40/0/Hollow Comet (World)"
edge 14 down same
edge 14 left 12
edge 14 right 16
edge 16 up "row:40/0/Hollow Comet (World)"
edge 16 down same
edge 16 left 14
edge 16 right same
edge 23 up 5
edge 23 down 25
edge 23 left same
edge 23 right 35
edge 25 up 23
edge 25 down 27
edge 25 left same
edge 25 right "row:40/0/Fable of Tiles (USA)"
edge 27 up 25
edge 27 down 29
edge 27 left same
edge 27 right 42
edge 29 up 27
edge 29 down "row:21/4/Nintendo - Nintendo 64  (not downloaded)"
edge 29 left same
edge 29 right 35
edge 35 up 5
edge 35 down "row:40/0/Hollow Comet (World)"
edge 35 left 23
edge 35 right 16
edge 42 up 35
edge 42 down 44
edge 42 left 25
edge 42 right same
edge 44 up 42
edge 44 down 46
edge 44 left 29
edge 44 right same
edge 46 up 44
edge 46 down "row:40/3/Cobalt Harbor (USA)"
edge 46 left 27
edge 46 right same
edge "row:21/4/Nintendo - Nintendo 64  (not downloaded)" up 29
edge "row:21/4/Nintendo - Nintendo 64  (not downloaded)" down "row:21/5/Nintendo - Satellaview  (not downloaded)"
edge "row:21/4/Nintendo - Nintendo 64  (not downloaded)" left same
edge "row:21/4/Nintendo - Nintendo 64  (not downloaded)" right 35
edge "row:21/5/Nintendo - Satellaview  (not downloaded)" up "row:21/4/Nintendo - Nintendo 64  (not downloaded)"
edge "row:21/5/Nintendo - Satellaview  (not downloaded)" down "row:21/6/Sega - Master System - Mark III  (not downloaded)"
edge "row:21/5/Nintendo - Satellaview  (not downloaded)" left same
edge "row:21/5/Nintendo - Satellaview  (not downloaded)" right 35
edge "row:40/0/Fable of Tiles (USA)" up 35
edge "row:40/0/Fable of Tiles (USA)" down "row:40/1/Granite Choir (Japan)"
edge "row:40/0/Fable of Tiles (USA)" left 25
edge "row:40/0/Fable of Tiles (USA)" right same
edge "row:40/0/Hollow Comet (World)" up 35
edge "row:40/0/Hollow Comet (World)" down 14
edge "row:40/0/Hollow Comet (World)" left 25
edge "row:40/0/Hollow Comet (World)" right same
edge "row:40/1/Granite Choir (Japan)" up "row:40/0/Fable of Tiles (USA)"
edge "row:40/1/Granite Choir (Japan)" down 14
edge "row:40/1/Granite Choir (Japan)" left 27
edge "row:40/1/Granite Choir (Japan)" right same
press pageup focus="TextBox PART_Path (Choose a cheat folder (a RetroArch cheats folder works as-is))" title="Cheat Database" top=CheatDatabaseWindow
press pagedown focus="TextBox PART_Path (Choose a cheat folder (a RetroArch cheats folder works as-is))" title="Cheat Database" top=CheatDatabaseWindow
press first focus="TextBox PART_Path (Choose a cheat folder (a RetroArch cheats folder works as-is))" title="Cheat Database" top=CheatDatabaseWindow
press last focus="TextBox PART_Path (Choose a cheat folder (a RetroArch cheats folder works as-is))" title="Cheat Database" top=CheatDatabaseWindow
press back top=none
