# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-MetadataEditor-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case MetadataEditor
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#FFFFFFFF role=none
    1 MenuPanel at=0,0,1280,800 by=clip heard="Edit Metadata" limit=0,0,1280,800 name=MetadataMenu role=menu
      2 HintBar at=411,742,458,48 by=clip heard="A Select, B Back, Y Scrape, D-pad up and down Choose" limit=0,0,1280,800 role=group
      3 ScrollViewer at=220,184,840,378 by=clip limit=0,0,1280,800 overflow=504 paints=#FFFFFFFF role=scrollviewer viewport=220,184,840,378
        4 StackPanel at=220,184,840,882 by=scroll in=3 limit=220,184,840,378 name=MetadataRows paints=#55555AFF,#9C9CA0FF,#D2D2D6FF role=none seen=220,184,840,378
          5 TextBox at=220,184,840,42 by=scroll drawn focus heard=Name in=3 limit=220,184,840,378 name=Meta_name op paints=#55555AFF,#EEEEF0FF role=edit text="Aurora Drift (Synthetic)"
          6 TextBox at=220,226,840,42 by=scroll drawn focus heard="Sort name" in=3 limit=220,184,840,378 name=Meta_sortname op paints=#55555AFF,#D2D2D6FF role=edit
          7 TextBox at=220,268,840,42 by=scroll drawn focus heard=Description in=3 limit=220,184,840,378 name=Meta_description op paints=#55555AFF,#D2D2D6FF role=edit
          8 MenuFieldRow at=220,310,840,42 by=scroll heard=Rating in=3 limit=220,184,840,378 role=listitem
            9 MenuRow at=220,310,840,42 by=scroll heard=Rating in=3 limit=220,184,840,378 nohit role=listitem
            10 RatingPicker at=947,320,105,21 by=scroll focus heard="0 of 5" in=3 limit=220,184,840,378 name=Meta_rating op role=slider sideways
          11 MenuFieldRow at=220,352,840,42 by=scroll heard="Release date" in=3 limit=220,184,840,378 role=listitem
            12 MenuRow at=220,352,840,42 by=scroll heard="Release date" in=3 limit=220,184,840,378 nohit role=listitem
            13 DateStepper at=931,355,121,35 by=scroll focus heard=unknown in=3 limit=220,184,840,378 name=Meta_releasedate op role=spinner sideways
          14 TextBox at=220,394,840,42 by=scroll drawn focus heard=Developer in=3 limit=220,184,840,378 name=Meta_developer op paints=#55555AFF,#D2D2D6FF role=edit
          15 TextBox at=220,436,840,42 by=scroll drawn focus heard=Publisher in=3 limit=220,184,840,378 name=Meta_publisher op paints=#55555AFF,#D2D2D6FF role=edit
          16 TextBox at=220,478,840,42 by=scroll drawn focus heard=Genre in=3 limit=220,184,840,378 name=Meta_genre op paints=#55555AFF,#D2D2D6FF role=edit
          17 TextBox at=220,520,840,42 by=scroll drawn focus heard=Players in=3 limit=220,184,840,378 name=Meta_players op paints=#55555AFF,#D2D2D6FF role=edit
      18 StackPanel at=255,574,770,49 by=clip limit=0,0,1280,800 name=MetadataButtons role=none
        19 Button at=255,574,125,49 by=clip drawn focus heard=Scrape limit=0,0,1280,800 name=MetadataScrape op paints=#55555AFF,#9C9CA0FF role=button text=Scrape
        20 Button at=388,574,89,49 by=clip drawn focus heard=Save limit=0,0,1280,800 name=MetadataSave op paints=#55555AFF,#9C9CA0FF role=button text=Save
        21 Button at=485,574,125,49 by=clip drawn focus heard=Cancel limit=0,0,1280,800 name=MetadataCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
        22 Button at=618,574,108,49 by=clip drawn focus heard=Clear limit=0,0,1280,800 name=MetadataClear op paints=#55555AFF,#9C9CA0FF role=button text=Clear
        23 Button at=734,574,291,49 by=clip drawn focus heard="Hide from Library..." limit=0,0,1280,800 name=MetadataHide op paints=#55555AFF,#9C9CA0FF role=button text="Hide from Library..."
focus 5
edge 5 up same
edge 5 down 6
edge 5 left same
edge 5 right same
edge 6 up 5
edge 6 down 7
edge 6 left same
edge 6 right same
edge 7 up 6
edge 7 down 10
edge 7 left same
edge 7 right same
edge 10 up 7
edge 10 down 13
edge 13 up 10
edge 13 down 14
edge 14 up 13
edge 14 down 15
edge 14 left same
edge 14 right same
edge 15 up 14
edge 15 down 16
edge 15 left same
edge 15 right same
edge 16 up 15
edge 16 down 17
edge 16 left same
edge 16 right same
edge 17 up 16
edge 17 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10
edge 17 left same
edge 17 right same
edge 19 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge 19 down same
edge 19 left same
edge 19 right 20
edge 20 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge 20 down same
edge 20 left 19
edge 20 right 21
edge 21 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge 21 down same
edge 21 left 20
edge 21 right 22
edge 22 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge 22 down same
edge 22 left 21
edge 22 right 23
edge 23 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge 23 down same
edge 23 left 22
edge 23 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10 up 17
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/10
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/11
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/12
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/13
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/14
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/15
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/16
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/17
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/18
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/20
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19 left same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19 right same
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/20 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/19
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/20 down path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21 up path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/20
edge path:0/3/0/0/2/3/0/0/0/0/1/0/0/0/21 down 19
press pageup focus="TextBox Meta_name (Name)" top=MetadataEditorWindow
press pagedown focus="TextBox Meta_name (Name)" top=MetadataEditorWindow
press first focus="TextBox Meta_name (Name)" top=MetadataEditorWindow
press last focus="TextBox Meta_name (Name)" top=MetadataEditorWindow
press back top=none
