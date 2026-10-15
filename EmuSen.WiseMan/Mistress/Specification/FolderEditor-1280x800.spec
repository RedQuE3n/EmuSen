# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FolderEditor-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 220,206.5,840,387
title "Edit Folder's Metadata"
case FolderEditor
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,306,840,267 by=panel limit=220,306,840,267 paints=#9C9CA0FF role=none
    1 StackPanel at=260,334,760,223 by=panel limit=220,306,840,267 role=none
      2 StackPanel at=260,334,760,50 by=panel limit=220,306,840,267 role=none
        3 TextBlock at=260,334,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=Racing layout=46.06,24 limit=220,306,840,267 name=FolderTitle paints=#9C9CA0FF role=text size=20 text=Racing
        4 HintText at=260,360,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=SNES/Racing layout=87.4,24 limit=220,306,840,267 paints=#86868AFF role=text size=20 text=SNES/Racing
      5 FieldRow at=260,396,760,105 by=panel heard="Folder link" limit=220,306,840,267 paints=#9C9CA0FF role=group
        6 TextBlock at=260,396,760,29 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Folder link" layout=84.34,28.8 limit=260,396,760,105 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Folder link"
        7 HintText at=260,429,760,24 box=760,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it." layout=712.98,24 limit=260,396,760,105 paints=#86868AFF part role=text size=20 text="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it."
        8 Dropdown at=260,457,760,44 by=clip drawn focus heard="Folder link" limit=260,396,760,105 name=FolderLink op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
          9 TextBlock at=272,462,716,32 box=716,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=(none) layout=54.96,31.2 limit=272,462,716,32 owner=8 paints=#9C9CA0FF role=text size=26 text=(none)
          10 ViewboxContainer at=998,475.73,12,6.54 by=clip limit=998,475.73,12,6.54 nohit role=none
      11 ButtonBar at=832,513,188,44 by=panel limit=220,306,840,267 paints=#9C9CA0FF role=toolbar
        12 Button at=832,513,77,44 by=clip drawn focus heard=SAVE limit=832,513,188,44 name=FolderSave op paints=#55555AFF,#9C9CA0FF role=button text=Save
          13 TextBlock at=849,519,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=832,513,77,44 owner=12 paints=#EEEEF0FF role=text size=26 text=SAVE
        14 Button at=917,513,103,44 by=clip drawn focus heard=CANCEL limit=832,513,188,44 name=FolderCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          15 TextBlock at=934,519,69,32 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=917,513,103,44 owner=14 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 12
edge 8 up same
edge 8 down 12
edge 12 up 8
edge 12 down same
edge 12 left same
edge 12 right 14
edge 14 up 8
edge 14 down same
edge 14 left 12
edge 14 right same
press pageup focus="Button 'Save'" title="Edit Folder's Metadata" top=FolderEditorWindow
press pagedown focus="Button 'Save'" title="Edit Folder's Metadata" top=FolderEditorWindow
press first focus="Button 'Save'" title="Edit Folder's Metadata" top=FolderEditorWindow
press last focus="Button 'Save'" title="Edit Folder's Metadata" top=FolderEditorWindow
press back top=none
