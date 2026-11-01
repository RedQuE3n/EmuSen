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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=220,306,840,267 by=panel limit=220,306,840,267 paints=#D4D4D4FF role=none
    1 StackPanel at=260,334,760,223 by=panel limit=220,306,840,267 role=none
      2 StackPanel at=260,334,760,50 by=panel limit=220,306,840,267 role=none
        3 TextBlock at=260,334,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=Racing layout=43.62,24 limit=220,306,840,267 name=FolderTitle paints=#D4D4D4FF role=text size=20 text=Racing
        4 HintText at=260,360,760,24 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=SNES/Racing layout=82.46,24 limit=220,306,840,267 paints=#8D8D91FF role=text size=20 text=SNES/Racing
      5 FieldRow at=260,396,760,105 by=panel heard="Folder link" limit=220,306,840,267 paints=#D4D4D4FF role=group
        6 TextBlock at=260,396,760,29 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Folder link" layout=79.37,28.8 limit=260,396,760,105 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Folder link"
        7 HintText at=260,429,760,24 box=760,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it." layout=677.06,24 limit=260,396,760,105 paints=#8D8D91FF part role=text size=20 text="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it."
        8 Dropdown at=260,457,760,44 by=clip drawn focus heard="Folder link" limit=260,396,760,105 name=FolderLink op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=combobox sideways
          9 TextBlock at=272,462,716,32 box=716,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=(none) layout=52.42,31.2 limit=272,462,716,32 owner=8 paints=#D4D4D4FF role=text size=26 text=(none)
          10 ViewboxContainer at=998,475.73,12,6.54 by=clip limit=998,475.73,12,6.54 nohit role=none
      11 ButtonBar at=838,513,182,44 by=panel limit=220,306,840,267 paints=#D4D4D4FF role=toolbar
        12 Button at=838,513,75,44 by=clip drawn focus heard=SAVE limit=838,513,182,44 name=FolderSave op paints=#707070FF,#D4D4D4FF role=button text=Save
          13 TextBlock at=855,519,41,32 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=40.87,31.2 limit=838,513,75,44 owner=12 paints=#EEEEF0FF role=text size=26 text=SAVE
        14 Button at=921,513,99,44 by=clip drawn focus heard=CANCEL limit=838,513,182,44 name=FolderCancel op paints=#707070FF,#D4D4D4FF role=button text=Cancel
          15 TextBlock at=938,519,65,32 box=65,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=64.74,31.2 limit=921,513,99,44 owner=14 paints=#D4D4D4FF role=text size=26 text=CANCEL
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
