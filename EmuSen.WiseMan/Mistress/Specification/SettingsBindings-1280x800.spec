# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsBindings-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsBindings
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=tab
          4 SectionTab at=64,40,159,44 by=panel drawn focus heard=Controllers limit=0,0,1280,800 op paints=#D4D4D4FF role=tabitem text=CONTROLLERS
            5 AccessText at=80,48,127,25 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=64,40,159,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 SectionTab at=231,40,235,44 by=panel drawn focus heard="Controller bindings" limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="CONTROLLER BINDINGS"
            7 AccessText at=247,48,203,25 box=203,25 by=clip caps=13.999 drawn face=Barlow layout=202.66,24 limit=231,40,235,44 owner=6 paints=#EEEEF0FF role=none size=20 text="CONTROLLER BINDINGS"
        8 Border at=64,108,704,578 by=panel heard="Controller bindings" limit=0,0,1280,800 name=SettingsContent paints=#26262BFF role=none
          9 StackPanel at=88,132,656,530 by=clip limit=64,108,704,578 role=none
            10 WholeRow at=88,132,656,44 by=clip limit=64,108,704,578 name=SettingsContentTabs paints=#D4D4D4FF role=tab
              11 SectionTab at=88,132,110,44 by=clip drawn focus heard="Player 1" limit=64,108,704,578 op paints=#D4D4D4FF role=tabitem text="PLAYER 1"
                12 AccessText at=104,140,78,25 box=78,25 by=clip caps=13.999 drawn face=Barlow layout=77.74,24 limit=88,132,110,44 owner=11 paints=#D4D4D4FF role=none size=20 text="PLAYER 1"
              13 SectionTab at=206,132,186,44 by=clip drawn focus heard="Super Nintendo" limit=64,108,704,578 op paints=#4C9AE8FF,#EEEEF0FF role=tabitem text="SUPER NINTENDO"
                14 AccessText at=222,140,154,25 box=154,25 by=clip caps=13.999 drawn face=Barlow layout=153.46,24 limit=206,132,186,44 owner=13 paints=#EEEEF0FF role=none size=20 text="SUPER NINTENDO"
              15 SectionTab at=400,132,106,44 by=clip drawn focus heard="Sega CD" limit=64,108,704,578 op paints=#D4D4D4FF role=tabitem text="SEGA CD"
                16 AccessText at=416,140,74,25 box=74,25 by=clip caps=13.999 drawn face=Barlow layout=73.12,24 limit=400,132,106,44 owner=15 paints=#D4D4D4FF role=none size=20 text="SEGA CD"
              17 SectionTab at=514,132,146,44 by=clip drawn focus heard=PlayStation limit=64,108,704,578 op paints=#D4D4D4FF role=tabitem text=PLAYSTATION
                18 AccessText at=530,140,114,25 box=114,25 by=clip caps=13.999 drawn face=Barlow layout=113.56,24 limit=514,132,146,44 owner=17 paints=#D4D4D4FF role=none size=20 text=PLAYSTATION
            19 Border at=88,176,656,380 by=clip limit=64,108,704,578 name=SettingsContentBody role=none
              20 ControllerDiagram at=88,176,656,380 by=clip heard="SNES controller" limit=88,176,656,380 name=BindingsDiagram role=group
            21 TextBlock at=88,568,656,53 box=656,53 by=clip caps=15.3989 drawn face=Barlow heard="Left and Right move between the drawing's regions; A rebinds the focused one." layout=603.64,52.8 limit=64,108,704,578 name=SettingsContentCaption paints=#8D8D91FF role=text size=22 text="Left and Right move between the drawing's regions; A rebinds the focused one."
        22 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          23 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            24 TextBlock at=824,132,368,39 box=368,39 by=clip caps=22.3984 drawn face=Barlow heard=Y layout=17.63,38.4 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=32 text=Y
            25 TextBlock at=824,179,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Super Nintendo · Y" layout=170.28,26.4 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="Super Nintendo · Y"
            26 TextBlock at=824,214,368,27 box=368,27 by=clip caps=15.3989 drawn face=Barlow heard="Bound to the pad's West button." layout=294.58,26.4 limit=800,108,416,578 name=SettingsLine paints=#D4D4D4FF role=text size=22 text="Bound to the pad's West button."
            27 TextBlock at=824,249,368,80 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="A listens for a press and rebinds it; Y resets this button; the profile saves at once." layout=345.38,79.2 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="A listens for a press and rebinds it; Y resets this button; the profile saves at once."
        28 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          29 HintBar at=64,716,1152,30 by=panel heard="Rebind, Region, Reset, Console, Player, Test, Back" limit=0,0,1280,800 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="controllers Controllers › Controller bindings none" top=none
press up focus="SettingsLayout SettingsScreen" settings="controllers Controllers › Controller bindings none" top=none
press back focus="MenuRow SettingsRow.bindings" settings="controllers  bindings" top=none
