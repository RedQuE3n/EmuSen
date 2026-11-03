# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsEngine-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsEngine
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONSOLES
            5 AccessText at=80,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=64,40,125,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          6 SectionTab at=197,40,186,44 by=panel drawn focus heard="Super Nintendo" limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text="SUPER NINTENDO"
            7 AccessText at=213,48,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=197,40,186,44 owner=6 paints=#D4D4D4FF role=none size=20 text="SUPER NINTENDO"
          8 SectionTab at=391,40,98,44 by=panel drawn focus heard=Engine limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text=ENGINE
            9 AccessText at=407,48,66,25 box=66,25 by=clip caps=13.999 drawn face=Barlow layout=65.94,24 limit=391,40,98,44 owner=8 paints=#EEEEF0FF role=none size=20 text=ENGINE
        10 RowList at=64,108,704,54 by=panel heard=Engine limit=0,0,1280,800 name=SettingsRows role=list
          11 MenuRow at=64,108,704,54 by=panel focus heard="VenusRT (Rust), ● Default" limit=0,0,1280,800 name="SettingsRow.engine:VenusRT (Rust)" role=listitem
        12 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          13 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            14 TextBlock at=824,132,368,53 box=368,53 by=clip caps=15.3989 drawn face=Barlow heard="VenusRT (Rust) runs every Super Nintendo game." layout=298.87,52.8 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="VenusRT (Rust) runs every Super Nintendo game."
            15 TextBlock at=824,193,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="A chooses; B keeps what was chosen." layout=346.37,26.4 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="A chooses; B keeps what was chosen."
        16 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          17 HintBar at=64,716,1152,30 by=panel heard="Choose, Back" limit=0,0,1280,800 role=group
focus none
press down focus="MenuRow SettingsRow.engine:VenusRT (Rust)" settings="consoles Consoles › Super Nintendo › Engine engine:VenusRT (Rust)" top=none
press up focus="MenuRow SettingsRow.engine:VenusRT (Rust)" settings="consoles Consoles › Super Nintendo › Engine engine:VenusRT (Rust)" top=none
press back focus="MenuRow SettingsRow.engine" settings="consoles Consoles › Super Nintendo engine" top=none
