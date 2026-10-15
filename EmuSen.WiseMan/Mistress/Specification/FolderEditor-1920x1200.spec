# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state FolderEditor-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 330,309.5,1260,581
title "Edit Folder's Metadata"
case FolderEditor
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=330,460,1260,402 by=panel limit=330,460,1260,401 paints=#9C9CA0FF role=none seen=330,460,1260,401
    1 StackPanel at=390,502,1140,336 by=panel limit=330,460,1260,401 role=none
      2 StackPanel at=390,502,1140,75 by=panel limit=330,460,1260,401 role=none
        3 TextBlock at=390,502,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=Racing layout=46.06,24 limit=330,460,1260,401 name=FolderTitle paints=#9C9CA0FF role=text size=20 text=Racing
        4 HintText at=390,541,1140,36 box=760,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard=SNES/Racing layout=87.4,24 limit=330,460,1260,401 paints=#86868AFF role=text size=20 text=SNES/Racing
      5 FieldRow at=390,595,1140,157.5 by=panel heard="Folder link" limit=330,460,1260,401 paints=#9C9CA0FF role=group
        6 TextBlock at=390,595,1140,43.5 box=760,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Folder link" layout=84.34,28.8 limit=390,595,1140,157.5 name=PART_Label paints=#9C9CA0FF part role=text size=24 text="Folder link"
        7 HintText at=390,644.5,1140,36 box=760,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it." layout=712.98,24 limit=390,595,1140,157.5 paints=#86868AFF part role=text size=20 text="A on the folder launches this game instead of opening the folder. Enter Folder in the options still opens it."
        8 Dropdown at=390,686.5,1140,66 by=clip drawn focus heard="Folder link" limit=390,595,1140,157.5 name=FolderLink op paints=#0E0E10FF,#55555AFF,#9C9CA0FF role=combobox sideways
          9 TextBlock at=408,694,1074,48 box=716,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=(none) layout=54.96,31.2 limit=408,694,1074,48 owner=8 paints=#9C9CA0FF role=text size=26 text=(none)
          10 ViewboxContainer at=1497,714.6,18,9.81 by=clip limit=1497,714.6,18,9.81 nohit role=none
      11 ButtonBar at=1248,770.5,282,66 by=panel limit=330,460,1260,401 paints=#9C9CA0FF role=toolbar
        12 Button at=1248,770.5,115.5,66 by=clip drawn focus heard=SAVE limit=1248,770.5,282,66 name=FolderSave op paints=#55555AFF,#9C9CA0FF role=button text=Save
          13 TextBlock at=1273.5,779.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=1248,770.5,115.5,66 owner=12 paints=#EEEEF0FF role=text size=26 text=SAVE
        14 Button at=1375.5,770.5,154.5,66 by=clip drawn focus heard=CANCEL limit=1248,770.5,282,66 name=FolderCancel op paints=#55555AFF,#9C9CA0FF role=button text=Cancel
          15 TextBlock at=1401,779.5,103.5,48 box=69,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL layout=68.02,31.2 limit=1375.5,770.5,154.5,66 owner=14 paints=#9C9CA0FF role=text size=26 text=CANCEL
focus 12
edge 8 up same
edge 8 down 12
edge 12 up 8
edge 12 down same
edge 12 left 8
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
