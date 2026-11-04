# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamePageDesktopEdit-1280x800
size 1280 800
kind window
root PageLayout#GamePage
unit 1
slack 1.5
floor 0
frame 0,0,1080,746
case GamePageDesktopEdit
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the page moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1080,746 by=panel limit=0,0,1080,746 role=none
    1 Panel at=0,0,1080,746 by=panel limit=0,0,1080,746 role=none
      2 Grid at=0,0,1080,746 by=panel limit=0,0,1080,746 paints=#1B1B1EFF role=none
        3 Panel at=0,0,1080,300 by=panel limit=0,0,1080,746 role=none
          4 PosterSlot at=0,0,1080,200 by=panel heard=Poster limit=0,0,1080,746 name=PagePoster nohit role=image
          5 Button at=24,16,80,28 by=panel drawn focus heard=Back limit=0,0,1080,746 name=PageBack op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="‹ Back"
            6 AccessText at=33,22,62,15 box=62,15 by=clip caps=9.0994 drawn face=Barlow layout=34.65,15.6 limit=24,16,80,28 owner=5 paints=#D4D4D4FF role=none size=13 text="‹ Back"
          7 Border at=24,93,150,200 by=panel heard="Cobalt Harbor (Synthetic), cover" limit=0,0,1080,746 name=PageCover paints=#26262BFF role=none
            8 Panel at=24,93,150,200 by=clip limit=24,93,150,200 role=none
              9 Image at=24,93,150,200 by=clip drawn limit=24,93,150,200 nohit role=image
          10 Viewbox at=208,104,201,29 by=panel limit=0,0,1080,746 role=none
            11 ViewboxContainer at=208,104,201,29 by=clip limit=208,104,201,29 role=none
              12 TextBlock at=208,104,201,29 box=201,29 by=clip caps=16.7988 drawn face="Barlow Condensed Medium" heard="Cobalt Harbor (Synthetic)" layout=200.21,28.8 limit=208,104,201,29 name=PageTitle paints=#FFFFFFFF role=text size=24 text="Cobalt Harbor (Synthetic)"
          13 Viewbox at=208,144,301,16 by=panel limit=0,0,1080,746 role=none
            14 ViewboxContainer at=208,144,301,16 by=clip limit=208,144,301,16 role=none
              15 TextBlock at=208,144,301,16 box=301,16 by=clip caps=9.0994 drawn face=Barlow heard="SNES · 1994 · Lantern Works · Publisher P · Never played" layout=300.25,15.6 limit=208,144,301,16 name=PageFacts paints=#FFFFFFFF role=text size=13 text="SNES · 1994 · Lantern Works · Publisher P · Never played"
          16 SelectionStack at=208,184,848,28 by=panel heard=Actions limit=0,0,1080,746 name=PageActions role=list
            17 Button at=208,184,96,28 by=panel drawn focus heard=Play limit=0,0,1080,746 name=PageAction.play op paints=#0E0E10FF,#4C9AE8FF,#707070FF role=button text=Play
              18 AccessText at=244,187,24,22 box=24,22 by=clip caps=9.0994 drawn face=Barlow layout=23.35,15.6 limit=208,184,96,28 owner=17 paints=#0E0E10FF role=none size=13 text=Play
            19 SplitButton at=320,184,201,28 by=panel focus heard="Resume: Auto · 12 min ago" limit=0,0,1080,746 name=PageAction.resume paints=#0E0E10FF,#D4D4D4FF role=splitbutton text="Resume: Auto · 12 min ago"
              20 Button at=320,184,168,28 drawn heard="Resume: Auto · 12 min ago" name=PART_PrimaryButton paints=#0E0E10FF,#D4D4D4FF part role=button text="Resume: Auto · 12 min ago"
                21 AccessText at=333,190,143,16 box=143,16 by=clip caps=9.0994 drawn face=Barlow layout=142.06,15.6 limit=320,184,168,28 owner=20 paints=#D4D4D4FF role=none size=13 text="Resume: Auto · 12 min ago"
              22 Button at=489,184,32,28 drawn heard=Avalonia.Controls.PathIcon name=PART_SecondaryButton paints=#0E0E10FF,#D4D4D4FF part role=button
                23 ViewboxContainer at=499,195,12,6.54 by=clip limit=499,195,12,6.54 role=none
            24 Button at=537,184,96,28 by=panel drawn focus heard=Avalonia.Controls.StackPanel limit=0,0,1080,746 name=PageAction.favourite op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button
              25 StackPanel at=550,187,70,22 by=clip limit=537,184,96,28 role=none
                26 Path at=550,192,12,12 by=clip limit=537,184,96,28 nohit role=none
                27 TextBlock at=568,190,52,16 box=52,16 by=clip caps=9.0994 drawn face=Barlow heard=Favourite layout=51.18,15.6 limit=537,184,96,28 owner=24 paints=#D4D4D4FF role=text size=13 text=Favourite
        28 Border at=24,300,1032,31 by=panel limit=0,0,1080,746 paints=#333337FF role=none
          29 SelectionStack at=36,300,1008,30 by=panel limit=0,0,1080,746 name=PageTabs role=tab
            30 SectionTab at=36,300,50,30 by=panel drawn focus heard=Details limit=0,0,1080,746 op paints=#D4D4D4FF role=tabitem text=DETAILS
              31 AccessText at=36,304,50,19 box=50,19 by=clip caps=9.7993 drawn face=Barlow layout=49.69,16.8 limit=36,300,50,30 owner=30 paints=#D4D4D4FF role=none size=14 text=DETAILS
            32 SectionTab at=110,300,80,30 by=panel drawn focus heard="Edit Details" limit=0,0,1080,746 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="EDIT DETAILS"
              33 AccessText at=110,304,80,19 box=80,19 by=clip caps=9.7993 drawn face=Barlow layout=79.81,16.8 limit=110,300,80,30 owner=32 paints=#EEEEF0FF role=none size=14 text="EDIT DETAILS"
        34 ScrollViewer at=24,349,1032,385 by=panel fades=b limit=0,0,1080,746 overflow=690 paints=#707070FF,#FFFFFFFF role=scrollviewer viewport=24,349,1032,385
          35 Border at=24,349,1032,1075 by=scroll in=34 limit=24,349,1032,385 name=PageContent role=none seen=24,349,1032,385
            36 StackPanel at=24,349,1032,1075 by=scroll in=34 limit=24,349,1032,385 name=PageRows paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF,#FFFFFFFF role=none seen=24,349,1032,385
              37 FieldRow at=24,349,1032,52 by=scroll heard=Name in=34 limit=24,349,1032,385 name=PageRow.name paints=#FFFFFFFF role=group
                38 TextBlock at=24,349,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard=Name in=34 layout=32.81,15.6 limit=24,349,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text=Name
                39 TextBox at=24,369,1032,32 by=clip drawn focus heard=Name in=34 limit=24,349,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Cobalt Harbor (Synthetic)"
                  40 ScrollViewer at=35,376,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              41 FieldRow at=24,405,1032,52 by=scroll heard="Sort name" in=34 limit=24,349,1032,385 name=PageRow.sortname paints=#FFFFFFFF role=group
                42 TextBlock at=24,405,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard="Sort name" in=34 layout=57.26,15.6 limit=24,405,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text="Sort name"
                43 TextBox at=24,425,1032,32 by=clip drawn focus heard="Sort name" in=34 limit=24,405,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  44 ScrollViewer at=35,432,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    45 TextBlock at=35,432,1014,19 by=fixed-scroll drawn in=34,44 limit=35,432,1014,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              46 FieldRow at=24,461,1032,52 by=scroll heard="Custom collections sortname" in=34 limit=24,349,1032,385 name=PageRow.collectionsortname paints=#FFFFFFFF role=group
                47 TextBlock at=24,461,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard="Custom collections sortname" in=34 layout=162.24,15.6 limit=24,461,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text="Custom collections sortname"
                48 TextBox at=24,481,1032,32 by=clip drawn focus heard="Custom collections sortname" in=34 limit=24,461,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  49 ScrollViewer at=35,488,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    50 TextBlock at=35,488,1014,19 by=fixed-scroll drawn in=34,49 limit=35,488,1014,19 name=PART_Placeholder paints=#8D8D91FF part role=text
              51 FieldRow at=24,517,1032,52 by=scroll heard=Description in=34 limit=24,349,1032,385 name=PageRow.description paints=#FFFFFFFF role=group
                52 TextBlock at=24,517,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard=Description in=34 layout=64.47,15.6 limit=24,517,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text=Description
                53 TextBox at=24,537,1032,32 by=clip drawn focus heard=Description in=34 limit=24,517,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text="Cobalt Harbor is a game…"
                  54 ScrollViewer at=35,544,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              55 FieldRow at=24,573,1032,52 by=scroll heard=Rating in=34 limit=24,349,1032,385 name=PageRow.rating paints=#FFFFFFFF role=group
                56 TextBlock at=24,573,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard=Rating in=34 layout=36.31,15.6 limit=24,573,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text=Rating
                57 ComboBox at=24,593,240,32 by=clip drawn focus heard=Rating in=34 limit=24,573,1032,52 op paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=combobox sideways
                  58 TextBlock at=36,600,196,16 box=196,16 by=clip caps=9.0994 drawn face=Barlow heard="No rating" in=34 layout=50.38,15.6 limit=36,598,196,20 owner=57 paints=#D4D4D4FF role=text size=13 text="No rating"
                  59 ViewboxContainer at=242,605.73,12,6.54 by=clip in=34 limit=242,605.73,12,6.54 nohit role=none
              60 FieldRow at=24,629,1032,52 by=scroll heard="Release date" in=34 limit=24,349,1032,385 name=PageRow.releasedate paints=#FFFFFFFF role=group
                61 TextBlock at=24,629,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard="Release date" in=34 layout=71.27,15.6 limit=24,629,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text="Release date"
                62 TextBox at=24,649,1032,32 by=clip drawn focus heard="Release date" in=34 limit=24,629,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=1994-03-01
                  63 ScrollViewer at=35,656,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              64 FieldRow at=24,685,1032,52 by=scroll heard=Developer in=34 limit=24,349,1032,385 name=PageRow.developer paints=#FFFFFFFF role=group seen=24,685,1032,49
                65 TextBlock at=24,685,1032,16 box=1032,16 by=clip caps=9.5056 drawn face="Barlow Medium" heard=Developer in=34 layout=56.1,15.6 limit=24,685,1032,52 name=PART_Label paints=#FFFFFFFF part role=text size=13 text=Developer
                66 TextBox at=24,705,1032,32 by=clip drawn focus heard=Developer in=34 limit=24,685,1032,52 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit seen=24,705,1032,29 text="Lantern Works"
                  67 ScrollViewer at=35,712,1014,19 in=34 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
focus none
