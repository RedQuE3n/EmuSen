# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 128,82.5,1024,635
title "Plain Shelf"
case ThemeDetail
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=128,182,1024,515 by=panel limit=128,182,1024,515 paints=#9C9CA0FF role=none
    1 DockPanel at=152,194,976,491 by=panel limit=128,182,1024,515 role=none
      2 Grid at=152,194,976,300 by=panel limit=128,182,1024,515 role=none
        3 StackPanel at=152,194,529,300 by=panel limit=128,182,1024,515 role=none
          4 FittedImage at=152,194,529,210 by=panel drawn limit=128,182,1024,515 name=ThemeDetailScreenshot role=image
          5 TextBlock at=152,410,529,29 box=529,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf, screenshot 1" layout=196.51,28.8 limit=128,182,1024,515 name=ThemeDetailCaption paints=#9C9CA0FF role=text size=24 text="Plain Shelf, screenshot 1"
          6 StackPanel at=152,445,529,44 by=panel limit=128,182,1024,515 role=none
            7 Button at=152,445,121,44 by=panel drawn focus heard=PREVIOUS limit=128,182,1024,515 name=ThemeDetailPrevious op paints=#55555AFF,#9C9CA0FF role=button text=Previous
              8 TextBlock at=169,451,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PREVIOUS layout=86.22,31.2 limit=152,445,121,44 owner=7 paints=#EEEEF0FF role=text size=26 text=PREVIOUS
            9 TextBlock at=285,453,42,29 box=42,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="1 of 2" layout=42,28.8 limit=128,182,1024,515 name=ThemeDetailPosition paints=#9C9CA0FF role=text size=24 text="1 of 2"
            10 Button at=339,445,81,44 by=panel drawn focus heard=NEXT limit=128,182,1024,515 name=ThemeDetailNext op paints=#55555AFF,#9C9CA0FF role=button text=Next
              11 TextBlock at=356,451,47,32 box=47,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NEXT layout=46.25,31.2 limit=339,445,81,44 owner=10 paints=#9C9CA0FF role=text size=26 text=NEXT
        12 ScrollViewer at=705,194,423,300 by=panel fades=b focus limit=128,182,1024,515 name=ThemeDetailFacts overflow=166 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=705,194,423,300
          13 StackPanel at=705,194,415,466 by=scroll in=12 limit=705,194,423,300 paints=#9C9CA0FF role=none seen=705,194,415,300
            14 TextBlock at=705,194,415,29 box=415,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="by Bo Sample   ·   github.com/bo/plain-shelf-es-de" in=12 layout=410.52,28.8 limit=705,194,423,300 name=ThemeDetailAuthor paints=#9C9CA0FF role=text size=24 text="by Bo Sample   ·   github.com/bo/plain-shelf-es-de"
            15 FieldRow at=705,231,415,62 by=scroll heard="Last Updated" in=12 limit=705,194,423,300 paints=#9C9CA0FF role=group
              16 TextBlock at=705,231,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Last Updated" in=12 layout=107.69,28.8 limit=705,231,415,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Last Updated"
              17 TextBlock at=705,264,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="2026-09-20, commit 1111111, on branch master" in=12 layout=364.82,28.8 limit=705,231,415,62 name=ThemeDetailUpdated paints=#9C9CA0FF role=text size=24 text="2026-09-20, commit 1111111, on branch master"
            18 FieldRow at=705,301,415,149 by=scroll heard=Here in=12 limit=705,194,423,300 paints=#9C9CA0FF role=group
              19 TextBlock at=705,301,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Here in=12 layout=37.78,28.8 limit=705,301,415,149 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Here
              20 TextBlock name=ThemeDetailState paints=#9C9CA0FF role=text volatile="its words name the run's own folders"
            21 FieldRow at=705,458,415,62 by=scroll heard="Variants (2)" in=12 limit=705,194,423,300 paints=#9C9CA0FF role=group seen=705,458,415,36
              22 TextBlock at=705,458,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Variants (2)" in=12 layout=92.69,28.8 limit=705,458,415,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Variants (2)"
              23 TextBlock at=705,491,415,29 box=415,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="List, Grid" in=12 layout=71.9,28.8 limit=705,458,415,62 name=ThemeDetailVariants paints=#9C9CA0FF role=text seen=705,491,415,3 size=24 text="List, Grid"
      24 StackPanel at=152,506,976,179 by=panel limit=128,182,1024,515 role=none
        25 FieldRow at=152,506,976,90 by=panel heard=Licence limit=128,182,1024,515 name=ThemeDetailLicenceRow paints=#9C9CA0FF role=group
          26 TextBlock at=152,506,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence layout=63.5,28.8 limit=152,506,976,90 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Licence
          27 HintText at=152,539,976,24 box=976,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms." layout=808.76,24 limit=152,506,976,90 paints=#86868AFF part role=text size=20 text="From the theme's own README, else the licence file its host names. The theme is its author's work, under its own terms."
          28 StackPanel at=152,567,976,29 by=clip heard=Licence limit=152,506,976,90 role=none
            29 TextBlock at=152,567,976,29 box=976,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Reading the licence line from the theme's repository…" layout=439.63,28.8 limit=152,506,976,90 name=ThemeDetailLicence0 paints=#9C9CA0FF role=text size=24 text="Reading the licence line from the theme's repository…"
        30 WrapPanel at=152,604,976,44 by=panel limit=128,182,1024,515 role=none
          31 Button at=152,604,101,44 by=panel drawn heard=UPDATE limit=128,182,1024,515 name=ThemeDetailUpdate off paints=#55555AFF,#9C9CA0FF role=button text=Update
            32 TextBlock at=169,610,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=152,604,101,44 off owner=31 paints=#55555AFF role=text size=26 text=UPDATE
          33 Button at=261,604,69,44 by=panel drawn focus heard=USE limit=128,182,1024,515 name=ThemeDetailUse op paints=#55555AFF,#9C9CA0FF role=button text=Use
            34 TextBlock at=278,610,35,32 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=USE layout=34.16,31.2 limit=261,604,69,44 owner=33 paints=#9C9CA0FF role=text size=26 text=USE
          35 Button at=338,604,105,44 by=panel drawn focus heard=REMOVE limit=128,182,1024,515 name=ThemeDetailRemove op paints=#55555AFF,#9C9CA0FF role=button text=Remove
            36 TextBlock at=355,610,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=338,604,105,44 owner=35 paints=#9C9CA0FF role=text size=26 text=REMOVE
          37 Button at=451,604,92,44 by=panel drawn focus heard=ABOUT limit=128,182,1024,515 name=ThemeDetailAbout op paints=#55555AFF,#9C9CA0FF role=button text=About
            38 TextBlock at=468,610,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ABOUT layout=57.56,31.2 limit=451,604,92,44 owner=37 paints=#9C9CA0FF role=text size=26 text=ABOUT
          39 Button at=551,604,90,44 by=panel drawn focus heard=CLOSE limit=128,182,1024,515 name=ThemeDetailClose op paints=#55555AFF,#9C9CA0FF role=button text=Close
            40 TextBlock at=568,610,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=551,604,90,44 owner=39 paints=#9C9CA0FF role=text size=26 text=CLOSE
        41 TextBlock at=152,656,976,29 by=panel drawn limit=128,182,1024,515 name=ThemeDetailStatus paints=#9C9CA0FF role=text
focus 7
edge 7 up same
edge 7 down 35
edge 7 left same
edge 7 right 10
edge 10 up same
edge 10 down 35
edge 10 left 7
edge 10 right 37
edge 33 up 10
edge 33 down same
edge 33 left same
edge 33 right 35
edge 35 up 10
edge 35 down same
edge 35 left 33
edge 35 right 37
edge 37 up 10
edge 37 down same
edge 37 left 35
edge 37 right 39
edge 39 up 10
edge 39 down same
edge 39 left 37
edge 39 right same
press pageup focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press pagedown focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press first focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press last focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
press back focus="Button 'Refresh'" title="ES-DE Themes" top=ThemeBrowserWindow
