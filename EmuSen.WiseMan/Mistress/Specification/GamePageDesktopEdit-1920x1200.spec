# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageDesktopEdit-1920x1200
size 1920 1200
kind window
root PageLayout#GamePage
unit 1.5
slack 2
floor 0
frame 0,0,1720,1146
case GamePageDesktopEdit
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 role=none
    1 Panel at=0,0,1720,1146 by=panel limit=0,0,1720,1146 role=none
      2 Grid at=0,0,1720,1146 by=panel limit=0,0,1720,1146 paints=#1B1B1EFF role=none
        3 Panel at=0,0,1720,300 by=panel limit=0,0,1720,1146 role=none
          4 PosterSlot at=0,0,1720,200 by=panel heard=Poster limit=0,0,1720,1146 name=PagePoster nohit role=image
          5 Button at=24,16,80,28 by=panel drawn focus heard=Back limit=0,0,1720,1146 name=PageBack op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="‹ Back"
            6 AccessText at=33,22,62,15 box=62,15 by=clip caps=6.0662 drawn face=Barlow layout=34.65,15.6 limit=24,16,80,28 owner=5 paints=#D4D4D4FF role=none size=8.67 text="‹ Back"
          7 Border at=24,93,146,200 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1720,1146 name=PageCover paints=#26262BFF role=none
            8 Panel at=24,93,146,200 by=clip limit=24,93,146,200 role=none
              9 Image at=24,93,146,200 by=clip drawn limit=24,93,146,200 nohit role=image
          10 Viewbox at=208,104,201,29 by=panel limit=0,0,1720,1146 role=none
            11 ViewboxContainer at=208,104,201,29 by=clip limit=208,104,201,29 role=none
              12 TextBlock at=208,104,201,29 box=201,29 by=clip caps=11.1992 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=200.21,28.8 limit=208,104,201,29 name=PageTitle paints=#FFFFFFFF role=text size=16 text="Cobalt Harbor (Synthetic)"
          13 Viewbox at=208,144,301,16 by=panel limit=0,0,1720,1146 role=none
            14 ViewboxContainer at=208,144,301,16 by=clip limit=208,144,301,16 role=none
              15 TextBlock at=208,144,301,16 box=301,16 by=clip caps=6.0662 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=300.25,15.6 limit=208,144,301,16 name=PageFacts paints=#FFFFFFFF role=text size=8.67 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
          16 StackPanel at=208,184,1488,28 by=panel heard=Actions limit=0,0,1720,1146 name=PageActions role=none
            17 Button at=208,184,96,28 by=panel drawn focus heard=Play limit=0,0,1720,1146 name=PageAction.play op paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=Play
              18 AccessText at=244,187,24,22 box=24,22 by=clip caps=6.0662 drawn face=Barlow layout=23.35,15.6 limit=208,184,96,28 owner=17 paints=#0E0E10FF role=none size=8.67 text=Play
            19 SplitButton at=320,184,201,28 by=panel focus heard="Resume: Auto · 12 min ago" limit=0,0,1720,1146 name=PageAction.resume paints=#0E0E10FF,#D4D4D4FF role=splitbutton text="Resume: Auto · 12 min ago"
              20 Button at=320,184,168,28 drawn heard="Resume: Auto · 12 min ago" name=PART_PrimaryButton paints=#0E0E10FF,#D4D4D4FF part role=button text="Resume: Auto · 12 min ago"
                21 AccessText at=333,190,143,16 box=143,16 by=clip caps=6.0662 drawn face=Barlow layout=142.06,15.6 limit=320,184,168,28 owner=20 paints=#D4D4D4FF role=none size=8.67 text="Resume: Auto · 12 min ago"
              22 Button at=489,184,32,28 drawn heard=Avalonia.Controls.PathIcon name=PART_SecondaryButton paints=#0E0E10FF,#D4D4D4FF part role=button
                23 ViewboxContainer at=499,195,12,6.54 by=clip limit=499,195,12,6.54 role=none
            24 Button at=537,184,96,28 by=panel drawn focus heard=Avalonia.Controls.StackPanel limit=0,0,1720,1146 name=PageAction.favourite op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button
              25 StackPanel at=550,187,70,22 by=clip limit=537,184,96,28 role=none
                26 Path at=550,192,12,12 by=clip limit=537,184,96,28 nohit role=none
                27 TextBlock at=568,190,52,16 box=52,16 by=clip caps=6.0662 drawn face=Barlow heard=Favourite layout=51.18,15.6 limit=537,184,96,28 owner=24 paints=#D4D4D4FF role=text size=8.67 text=Favourite
        28 Border at=24,300,1672,31 by=panel limit=0,0,1720,1146 paints=#333337FF role=none
          29 StackPanel at=36,300,1648,30 by=panel limit=0,0,1720,1146 name=PageTabs role=none
            30 ToggleButton at=36,300,50,30 by=panel drawn focus heard=Details limit=0,0,1720,1146 op paints=#D4D4D4FF role=button text=DETAILS
              31 AccessText at=36,304,50,19 box=50,19 by=clip caps=6.5329 drawn face=Barlow layout=49.69,16.8 limit=36,300,50,30 owner=30 paints=#D4D4D4FF role=none size=9.33 text=DETAILS
            32 ToggleButton at=110,300,80,30 by=panel drawn focus heard="Edit Details" limit=0,0,1720,1146 op paints=#4C9AE8FF,#EEEEF0FF role=button text="EDIT DETAILS"
              33 AccessText at=110,304,80,19 box=80,19 by=clip caps=6.5329 drawn face=Barlow layout=79.81,16.8 limit=110,300,80,30 owner=32 paints=#EEEEF0FF role=none size=9.33 text="EDIT DETAILS"
        34 ScrollViewer at=24,349,1672,785 by=panel fades=b limit=0,0,1720,1146 overflow=290 paints=#707070FF,#FFFFFFFF role=scrollviewer viewport=24,349,1672,785
          35 Border at=24,349,1672,1075 by=scroll in=34 limit=24,349,1672,785 name=PageContent role=none seen=24,349,1672,785
            36 StackPanel at=24,349,1672,1075 by=scroll in=34 limit=24,349,1672,785 name=PageRows paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF,#FFFFFFFF role=none seen=24,349,1672,785
              37 FieldRow at=24,349,1672,52 by=scroll heard=Name in=34 limit=24,349,1672,785 name=PageRow.name paints=#FFFFFFFF role=group
                38 TextBlock at=24,349,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Name in=34 layout=32.81,15.6 limit=24,349,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Name
                39 TextBox at=24,369,1672,32 by=clip drawn focus heard=Name in=34 limit=24,349,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Cobalt Harbor (Synthetic)"
                  40 ScrollViewer at=35,376,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              41 FieldRow at=24,405,1672,52 by=scroll heard="Sort name" in=34 limit=24,349,1672,785 name=PageRow.sortname paints=#FFFFFFFF role=group
                42 TextBlock at=24,405,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Sort name" in=34 layout=57.26,15.6 limit=24,405,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Sort name"
                43 TextBox at=24,425,1672,32 by=clip drawn focus heard="Sort name" in=34 limit=24,405,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  44 ScrollViewer at=35,432,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    45 TextBlock at=35,432,1654,19 by=fixed-scroll drawn in=34,44 limit=35,432,1654,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              46 FieldRow at=24,461,1672,52 by=scroll heard="Custom collections sortname" in=34 limit=24,349,1672,785 name=PageRow.collectionsortname paints=#FFFFFFFF role=group
                47 TextBlock at=24,461,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Custom collections sortname" in=34 layout=162.24,15.6 limit=24,461,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Custom collections sortname"
                48 TextBox at=24,481,1672,32 by=clip drawn focus heard="Custom collections sortname" in=34 limit=24,461,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  49 ScrollViewer at=35,488,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    50 TextBlock at=35,488,1654,19 by=fixed-scroll drawn in=34,49 limit=35,488,1654,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              51 FieldRow at=24,517,1672,52 by=scroll heard=Description in=34 limit=24,349,1672,785 name=PageRow.description paints=#FFFFFFFF role=group
                52 TextBlock at=24,517,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Description in=34 layout=64.47,15.6 limit=24,517,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Description
                53 TextBox at=24,537,1672,32 by=clip drawn focus heard=Description in=34 limit=24,517,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Cobalt Harbor is a game…"
                  54 ScrollViewer at=35,544,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              55 FieldRow at=24,573,1672,52 by=scroll heard=Rating in=34 limit=24,349,1672,785 name=PageRow.rating paints=#FFFFFFFF role=group
                56 TextBlock at=24,573,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Rating in=34 layout=36.31,15.6 limit=24,573,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Rating
                57 ComboBox at=24,593,240,32 by=clip drawn focus heard=Rating in=34 limit=24,573,1672,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  58 TextBlock at=36,600,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard="No rating" in=34 layout=50.38,15.6 limit=36,598,196,20 owner=57 paints=#D4D4D4FF role=text size=8.67 text="No rating"
                  59 ViewboxContainer at=242,605.73,12,6.54 by=clip in=34 limit=242,605.73,12,6.54 nohit role=none
              60 FieldRow at=24,629,1672,52 by=scroll heard="Release date" in=34 limit=24,349,1672,785 name=PageRow.releasedate paints=#FFFFFFFF role=group
                61 TextBlock at=24,629,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Release date" in=34 layout=71.27,15.6 limit=24,629,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Release date"
                62 TextBox at=24,649,1672,32 by=clip drawn focus heard="Release date" in=34 limit=24,629,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=1994-03-01
                  63 ScrollViewer at=35,656,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              64 FieldRow at=24,685,1672,52 by=scroll heard=Developer in=34 limit=24,349,1672,785 name=PageRow.developer paints=#FFFFFFFF role=group
                65 TextBlock at=24,685,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Developer in=34 layout=56.1,15.6 limit=24,685,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Developer
                66 TextBox at=24,705,1672,32 by=clip drawn focus heard=Developer in=34 limit=24,685,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Lantern Works"
                  67 ScrollViewer at=35,712,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              68 FieldRow at=24,741,1672,52 by=scroll heard=Publisher in=34 limit=24,349,1672,785 name=PageRow.publisher paints=#FFFFFFFF role=group
                69 TextBlock at=24,741,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Publisher in=34 layout=52.79,15.6 limit=24,741,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Publisher
                70 TextBox at=24,761,1672,32 by=clip drawn focus heard=Publisher in=34 limit=24,741,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Publisher P"
                  71 ScrollViewer at=35,768,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              72 FieldRow at=24,797,1672,52 by=scroll heard=Genre in=34 limit=24,349,1672,785 name=PageRow.genre paints=#FFFFFFFF role=group
                73 TextBlock at=24,797,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Genre in=34 layout=32.82,15.6 limit=24,797,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Genre
                74 TextBox at=24,817,1672,32 by=clip drawn focus heard=Genre in=34 limit=24,797,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  75 ScrollViewer at=35,824,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    76 TextBlock at=35,824,1654,19 by=fixed-scroll drawn in=34,75 limit=35,824,1654,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              77 FieldRow at=24,853,1672,52 by=scroll heard=Players in=34 limit=24,349,1672,785 name=PageRow.players paints=#FFFFFFFF role=group
                78 TextBlock at=24,853,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Players in=34 layout=41.27,15.6 limit=24,853,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Players
                79 TextBox at=24,873,1672,32 by=clip drawn focus heard=Players in=34 limit=24,853,1672,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  80 ScrollViewer at=35,880,1654,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    81 TextBlock at=35,880,1654,19 by=fixed-scroll drawn in=34,80 limit=35,880,1654,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              82 FieldRow at=24,909,1672,52 by=scroll heard=Completed in=34 limit=24,349,1672,785 name=PageRow.completed paints=#FFFFFFFF role=group
                83 TextBlock at=24,909,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Completed in=34 layout=60.41,15.6 limit=24,909,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Completed
                84 ComboBox at=24,929,240,32 by=clip drawn focus heard=Completed in=34 limit=24,909,1672,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  85 TextBlock at=36,936,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard=Off in=34 layout=16.16,15.6 limit=36,934,196,20 owner=84 paints=#D4D4D4FF role=text size=8.67 text=Off
                  86 ViewboxContainer at=242,941.73,12,6.54 by=clip in=34 limit=242,941.73,12,6.54 nohit role=none
              87 FieldRow at=24,965,1672,52 by=scroll heard="Kid game" in=34 limit=24,349,1672,785 name=PageRow.kidgame paints=#FFFFFFFF role=group
                88 TextBlock at=24,965,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Kid game" in=34 layout=52.1,15.6 limit=24,965,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Kid game"
                89 ComboBox at=24,985,240,32 by=clip drawn focus heard="Kid game" in=34 limit=24,965,1672,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  90 TextBlock at=36,992,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard=Off in=34 layout=16.16,15.6 limit=36,990,196,20 owner=89 paints=#D4D4D4FF role=text size=8.67 text=Off
                  91 ViewboxContainer at=242,997.73,12,6.54 by=clip in=34 limit=242,997.73,12,6.54 nohit role=none
              92 FieldRow at=24,1021,1672,52 by=scroll heard=Hidden in=34 limit=24,349,1672,785 name=PageRow.hidden paints=#FFFFFFFF role=group
                93 TextBlock at=24,1021,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard=Hidden in=34 layout=39.77,15.6 limit=24,1021,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text=Hidden
                94 ComboBox at=24,1041,240,32 by=clip drawn focus heard=Hidden in=34 limit=24,1021,1672,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  95 TextBlock at=36,1048,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard=Off in=34 layout=16.16,15.6 limit=36,1046,196,20 owner=94 paints=#D4D4D4FF role=text size=8.67 text=Off
                  96 ViewboxContainer at=242,1053.73,12,6.54 by=clip in=34 limit=242,1053.73,12,6.54 nohit role=none
              97 FieldRow at=24,1077,1672,52 by=scroll heard="Broken / not working" in=34 limit=24,349,1672,785 name=PageRow.broken paints=#FFFFFFFF role=group
                98 TextBlock at=24,1077,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Broken / not working" in=34 layout=116.05,15.6 limit=24,1077,1672,52 name=PART_Label paints=#FFFFFFFF part role=text size=8.67 text="Broken / not working"
                99 ComboBox at=24,1097,240,32 by=clip drawn focus heard="Broken / not working" in=34 limit=24,1077,1672,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  100 TextBlock at=36,1104,196,16 box=196,16 by=clip caps=6.0662 drawn face=Barlow heard=Off in=34 layout=16.16,15.6 limit=36,1102,196,20 owner=99 paints=#D4D4D4FF role=text size=8.67 text=Off
                  101 ViewboxContainer at=242,1109.73,12,6.54 by=clip in=34 limit=242,1109.73,12,6.54 nohit role=none
              102 FieldRow at=24,1133,1672,52 by=scroll heard="Exclude from game counter" in=34 limit=24,349,1672,785 name=PageRow.nogamecount paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF,#FFFFFFFF role=group seen=24,1133,1672,1
                103 TextBlock at=24,1133,1672,16 box=1672,16 by=clip caps=6.3371 drawn face="Barlow Medium" heard="Exclude from game counter" in=34 layout=153.69,15.6 limit=24,1133,1672,52 name=PART_Label paints=#FFFFFFFF part role=text seen=24,1133,1672,1 size=8.67 text="Exclude from game counter"
focus none
