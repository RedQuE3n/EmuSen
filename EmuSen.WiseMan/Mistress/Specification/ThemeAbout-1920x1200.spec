# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ThemeAbout-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,37,1260,1041
title "About Plain Shelf"
case ThemeAbout
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,187,1260,861 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=none
    1 DockPanel at=390,229,1140,795 by=panel limit=330,187,1260,861 role=none
      2 ButtonBar at=1399.5,958,130.5,66 by=panel limit=330,187,1260,861 paints=#D4D4D4FF role=toolbar
        3 Button at=1399.5,958,130.5,66 by=clip drawn focus heard=CLOSE limit=1399.5,958,130.5,66 op paints=#707070FF,#D4D4D4FF role=button text=Close
          4 TextBlock at=1425,967,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1399.5,958,130.5,66 owner=3 paints=#EEEEF0FF role=text size=26 text=CLOSE
      5 ScrollViewer at=390,229,1140,711 by=panel fades=b focus limit=330,187,1260,861 name=AboutScroll overflow=82.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=390,229,1140,711
        6 StackPanel at=396,247,1128,769.5 by=scroll in=5 limit=390,229,1140,711 role=none seen=396,247,1128,693
          7 FieldRow at=396,247,1128,93 by=scroll heard=Theme in=5 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            8 TextBlock at=396,247,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Theme in=5 layout=52.61,28.8 limit=396,247,1128,93 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Theme
            9 TextBlock at=396,296.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Plain Shelf" in=5 layout=80.78,28.8 limit=396,247,1128,93 name=AboutName paints=#D4D4D4FF role=text size=24 text="Plain Shelf"
          10 FieldRow at=396,358,1128,93 by=scroll heard=Author in=5 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            11 TextBlock at=396,358,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Author in=5 layout=51.46,28.8 limit=396,358,1128,93 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Author
            12 TextBlock at=396,407.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="bo, the owner of its repository on GitHub" in=5 layout=310.78,28.8 limit=396,358,1128,93 name=AboutAuthor paints=#D4D4D4FF role=text size=24 text="bo, the owner of its repository on GitHub"
          13 FieldRow at=396,469,1128,135 by=scroll heard=Licence in=5 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            14 TextBlock at=396,469,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Licence in=5 layout=59.88,28.8 limit=396,469,1128,135 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Licence
            15 HintText at=396,518.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README or LICENSE file." in=5 layout=301.16,24 limit=396,469,1128,135 paints=#8D8D91FF part role=text size=20 text="From the theme's own README or LICENSE file."
            16 StackPanel at=396,560.5,1128,43.5 by=clip heard=Licence in=5 limit=396,469,1128,135 role=none
              17 TextBlock at=396,560.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The theme states no licence in a README or LICENSE file." in=5 layout=440.42,28.8 limit=396,469,1128,135 name=AboutLicence0 paints=#D4D4D4FF role=text size=24 text="The theme states no licence in a README or LICENSE file."
          18 FieldRow at=396,622,1128,135 by=scroll heard=Credits in=5 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            19 TextBlock at=396,622,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Credits in=5 layout=54.41,28.8 limit=396,622,1128,135 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Credits
            20 HintText at=396,671.5,1128,36 box=752,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="From the theme's own README." in=5 layout=202.1,24 limit=396,622,1128,135 paints=#8D8D91FF part role=text size=20 text="From the theme's own README."
            21 TextBlock at=396,713.5,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="The README names no credits." in=5 layout=238.66,28.8 limit=396,622,1128,135 name=AboutCredits paints=#D4D4D4FF role=text size=24 text="The README names no credits."
          22 FieldRow at=396,775,1128,136.5 by=scroll heard=Source in=5 limit=390,229,1140,711 paints=#D4D4D4FF role=group
            23 TextBlock at=396,775,1128,43.5 box=752,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Source in=5 layout=53.88,28.8 limit=396,775,1128,136.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Source
            24 TextBlock at=396,824.5,1128,87 box=752,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="https://github.com/bo/plain-shelf-es-de, branch master, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" in=5 layout=500.06,57.6 limit=396,775,1128,136.5 name=AboutSource paints=#D4D4D4FF role=text size=24 text="https://github.com/bo/plain-shelf-es-de, branch master, commit 1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
          25 TextBlock name=AboutStatement paints=#D4D4D4FF role=text volatile="says on which day the theme was downloaded, by the wall clock"
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
