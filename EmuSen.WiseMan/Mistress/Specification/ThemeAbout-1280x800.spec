# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeAbout-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,24,840,694
title "About Plain Shelf"
case ThemeAbout
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,124,840,574 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=none
    1 DockPanel at=260,152,760,530 by=panel limit=220,124,840,574 role=none
      2 ButtonBar at=930,638,90,44 by=panel limit=220,124,840,574 paints=#9C9CA0FF role=toolbar
        3 Button at=930,638,90,44 by=clip drawn focus heard=CLOSE limit=930,638,90,44 op paints=#55555AFF,#9C9CA0FF role=button text=Close
          4 TextBlock at=947,644,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=930,638,90,44 owner=3 paints=#EEEEF0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=260,152,760,474 by=panel fades=b focus limit=220,124,840,574 name=AboutScroll overflow=55 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=260,152,760,474
        6 StackPanel at=264,164,752,513 by=scroll in=5 limit=260,152,760,474 role=none seen=264,164,752,462
          7 FieldRow at=264,164,752,62 by=scroll heard=Theme in=5 limit=260,152,760,474 paints=#9C9CA0FF role=group
            8 TextBlock at=264,164,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Theme in=5 layout=55.42,28.8 limit=264,164,752,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Theme
            9 TextBlock at=264,197,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf" in=5 layout=85.75,28.8 limit=264,164,752,62 name=AboutName paints=#9C9CA0FF role=text size=24 text="Plain Shelf"
          10 FieldRow at=264,238,752,62 by=scroll heard=Author in=5 limit=260,152,760,474 paints=#9C9CA0FF role=group
            11 TextBlock at=264,238,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Author in=5 layout=53.42,28.8 limit=264,238,752,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Author
            12 TextBlock at=264,271,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="bo, the owner of its repository on GitHub" in=5 layout=328.42,28.8 limit=264,238,752,62 name=AboutAuthor paints=#9C9CA0FF role=text size=24 text="bo, the owner of its repository on GitHub"
          13 FieldRow at=264,312,752,90 by=scroll heard=Licence in=5 limit=260,152,760,474 paints=#9C9CA0FF role=group
            14 TextBlock at=264,312,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence in=5 layout=63.5,28.8 limit=264,312,752,90 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Licence
            15 HintText at=264,345,752,24 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README or LICENSE file." in=5 layout=314.44,24 limit=264,312,752,90 paints=#86868AFF part role=text size=20 text="From the theme's own README or LICENSE file."
            16 StackPanel at=264,373,752,29 by=clip heard=Licence in=5 limit=264,312,752,90 role=none
              17 TextBlock at=264,373,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The theme states no licence in a README or LICENSE file." in=5 layout=463.73,28.8 limit=264,312,752,90 name=AboutLicence0 paints=#9C9CA0FF role=text size=24 text="The theme states no licence in a README or LICENSE file."
          18 FieldRow at=264,414,752,90 by=scroll heard=Credits in=5 limit=260,152,760,474 paints=#9C9CA0FF role=group
            19 TextBlock at=264,414,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Credits in=5 layout=57.91,28.8 limit=264,414,752,90 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Credits
            20 HintText at=264,447,752,24 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README." in=5 layout=210.66,24 limit=264,414,752,90 paints=#86868AFF part role=text size=20 text="From the theme's own README."
            21 TextBlock at=264,475,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The README names no credits." in=5 layout=250.49,28.8 limit=264,414,752,90 name=AboutCredits paints=#9C9CA0FF role=text size=24 text="The README names no credits."
          22 FieldRow at=264,516,752,91 by=scroll heard=Source in=5 limit=260,152,760,474 paints=#9C9CA0FF role=group
            23 TextBlock at=264,516,752,29 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Source in=5 layout=56.45,28.8 limit=264,516,752,91 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Source
            24 TextBlock at=264,549,752,58 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="https://github.com/bo/plain-shelf-es-de, branch master, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in=5 layout=530.45,57.6 limit=264,516,752,91 name=AboutSource paints=#9C9CA0FF role=text size=24 text="https://github.com/bo/plain-shelf-es-de, branch master, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
          25 TextBlock name=AboutStatement paints=#9C9CA0FF role=text volatile="says on which day the theme was downloaded, by the wall clock"
focus 3
edge 3 up same
edge 3 down same
edge 3 left same
edge 3 right same
press pageup focus="Button 'Close'" title="About Plain Shelf" top=ThemeAboutWindow
press pagedown focus="Button 'Close'" title="About Plain Shelf" top=ThemeAboutWindow
press first focus="Button 'Close'" title="About Plain Shelf" top=ThemeAboutWindow
press last focus="Button 'Close'" title="About Plain Shelf" top=ThemeAboutWindow
press back focus="Button 'Previous'" title="Plain Shelf" top=ThemeDetailWindow
