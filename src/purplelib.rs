//! Purple-lib: the settings under Options > Mod Options > Better Monster Information.
//!
//! Optional. Purple-lib calls `purplelib_on_register` (found by name in the exported symbols) once from its first
//! update; without Purple-lib nothing here runs and the Forge menu stays the only way to change the settings. Changes
//! apply at once and are saved to the same INI as the Forge menu when the Options window closes. Records and API follow
//! Purple-lib's purplelib.h, API version 4 (Switch layout: 4-byte pointers), as in Better Skillinfo's purplelib.rs.
use core::ffi::{c_char,c_void};
use crate::STATE;

const API_VERSION:u32=4;
const ROW_LABELS:u32=0;

#[repr(C)]
pub struct Row{
    name:*const c_char,help:*const c_char,labels:*const *const c_char,label_count:u32,kind:u32,
    value_count:u32,first:i32,step:i32,suffix:*const c_char,flags:u32,
}
#[repr(C)]
pub struct Options{
    size:u32,context:*mut c_void,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    row_count:u32,
    row:Option<unsafe extern "C" fn(*mut c_void,u32,u32,*mut Row)->bool>,
    get:Option<unsafe extern "C" fn(*mut c_void,u32)->u32>,
    set:Option<unsafe extern "C" fn(*mut c_void,u32,u32)>,
    closed:Option<unsafe extern "C" fn(*mut c_void)>,
    value_text:Option<unsafe extern "C" fn(*mut c_void,u32,u32,u32)->*const c_char>,
    action:Option<unsafe extern "C" fn(*mut c_void,u32)>,
}
#[repr(C)]
pub struct Help{
    size:u32,context:*mut c_void,category:u32,page_count:u32,
    title:Option<unsafe extern "C" fn(*mut c_void,u32)->*const c_char>,
    page:Option<unsafe extern "C" fn(*mut c_void,u32,u32)->*const c_char>,
    open:Option<unsafe extern "C" fn(*mut c_void)>,
}
#[repr(C)]
pub struct Api{
    version:u32,
    add_options:Option<unsafe extern "C" fn(*const Options)->bool>,
    add_help:Option<unsafe extern "C" fn(*const Help)->bool>,
    hold_input:Option<unsafe extern "C" fn()>,
    options_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
    options_row:Option<unsafe extern "C" fn(*const Options)->i32>,
    add_set_help:Option<unsafe extern "C" fn(*const Options,*const Help)->bool>,
    help_shown:Option<unsafe extern "C" fn(*const Options)->bool>,
}
#[cfg(target_arch="arm")]
const _:()={
    assert!(core::mem::size_of::<Options>()==40);assert!(core::mem::size_of::<Row>()==40);
    assert!(core::mem::size_of::<Help>()==28);assert!(core::mem::size_of::<Api>()==32);
};

/// Strings the game may keep a pointer to: Purple-lib's rule is that they live forever.
struct Labels<const N:usize>([*const c_char;N]);
unsafe impl<const N:usize> Sync for Labels<N> {}
macro_rules! labels{($($t:literal),*)=>{Labels([$(concat!($t,"\0").as_ptr() as *const c_char),*])}}

/// One language's text. Rows: language, button layout, logging (the Forge menu's three settings). The language row
/// lists the mod's languages in State::language order (the Forge menu's), named in the page's language, because the
/// game's font may lack the native names (日本語, 简体中文) under the Korean patch.
struct Text{names:Labels<3>,helps:Labels<3>,on_off:Labels<2>,languages:Labels<9>,pages:Labels<2>}
/// In the mod's language order: English, Chinese (S), Chinese (T), Japanese, Korean, German, French, Italian, Spanish
static TEXT:[Text;9]=[
    // en
    Text{
        names:labels!("Language","Button layout","Enable logging"),
        helps:labels!("The language of the info page.","Button pictures in the prompts.","Write diagnostic logs. Warnings and errors are always logged."),
        on_off:labels!("Off","On"),
        languages:labels!("English","Chinese (S)","Chinese (T)","Japanese","Korean","German","French","Italian","Spanish"),
        pages:labels!("Adds hitzone, ailment, trap and material\r\ninfo as the last page of each monster\r\nin the Hunter's Notes.","Changes show at once. They are saved to\r\nbetter-monster-information.ini next to\r\nyour save when you close the window."),
    },
    // zh-Hans
    Text{
        names:labels!("语言","按键布局","启用日志记录"),
        helps:labels!("信息页面使用的语言。","按键提示图标的布局。","记录诊断日志。警告和错误始终记录。"),
        on_off:labels!("关","开"),
        languages:labels!("英语","简体中文","繁体中文","日语","韩语","德语","法语","意大利语","西班牙语"),
        pages:labels!("在猎人笔记的怪物图鉴最后一页\r\n添加肉质、异常状态、陷阱和素材\r\n信息。","更改会立即生效。关闭窗口时保存到\r\n存档旁的\r\nbetter-monster-information.ini。"),
    },
    // zh-Hant
    Text{
        names:labels!("語言","按鍵配置","啟用日誌記錄"),
        helps:labels!("資訊頁面使用的語言。","按鍵提示圖示的配置。","記錄診斷日誌。警告與錯誤一律記錄。"),
        on_off:labels!("關","開"),
        languages:labels!("英文","簡體中文","繁體中文","日文","韓文","德文","法文","義大利文","西班牙文"),
        pages:labels!("在獵人筆記的魔物圖鑑最後一頁\r\n新增肉質、異常狀態、陷阱和素材\r\n資訊。","變更會立即生效。關閉視窗時儲存到\r\n存檔旁的\r\nbetter-monster-information.ini。"),
    },
    // ja
    Text{
        names:labels!("言語","ボタン配置","ログ記録を有効にする"),
        helps:labels!("情報ページの言語です。","ボタン案内の絵の配置です。","診断ログを記録します。警告とエラーは常に記録されます。"),
        on_off:labels!("オフ","オン"),
        languages:labels!("英語","中国語（簡体）","中国語（繁体）","日本語","韓国語","ドイツ語","フランス語","イタリア語","スペイン語"),
        pages:labels!("ハンターノートのモンスター図鑑の最後の\r\nページに、肉質、状態異常、罠、素材の\r\n情報を追加します。","変更はすぐに反映されます。ウィンドウを\r\n閉じるとセーブと同じ場所の\r\nbetter-monster-information.ini\r\nに保存されます。"),
    },
    // ko
    Text{
        names:labels!("언어","버튼 배치","로그 기록 활성화"),
        helps:labels!("정보 페이지에 쓸 언어입니다.","버튼 안내 그림의 배치입니다.","진단 로그를 기록합니다. 경고와 오류는 항상 기록됩니다."),
        on_off:labels!("끄기","켜기"),
        languages:labels!("영어","중국어(간체)","중국어(번체)","일본어","한국어","독일어","프랑스어","이탈리아어","스페인어"),
        pages:labels!("헌터 노트 몬스터 목록의 마지막 페이지에\r\n육질, 상태 이상, 함정, 소재 정보를\r\n추가합니다.","변경 내용은 바로 반영됩니다. 창을 닫으면\r\n세이브 옆의\r\nbetter-monster-information.ini에\r\n저장됩니다."),
    },
    // de
    Text{
        names:labels!("Sprache","Tastenlayout","Protokollierung aktivieren"),
        helps:labels!("Sprache der Infoseite.","Tastensymbole in den Hinweisen.","Schreibt Diagnoseprotokolle. Warnungen und Fehler immer."),
        on_off:labels!("Aus","An"),
        languages:labels!("Englisch","Chinesisch (V)","Chinesisch (T)","Japanisch","Koreanisch","Deutsch","Französisch","Italienisch","Spanisch"),
        pages:labels!("Fügt jedem Monster im Jägerhandbuch\r\neine letzte Seite mit Trefferzonen,\r\nStatus, Fallen und Materialien hinzu.","Änderungen gelten sofort. Beim Schließen\r\nwerden sie neben dem Spielstand in\r\nbetter-monster-information.ini\r\ngespeichert."),
    },
    // fr
    Text{
        names:labels!("Langue","Disposition des boutons","Activer la journalisation"),
        helps:labels!("Langue de la page d'infos.","Icônes des boutons dans les aides.","Écrit les journaux de diagnostic. Avertissements et erreurs toujours."),
        on_off:labels!("Non","Oui"),
        languages:labels!("Anglais","Chinois (S)","Chinois (T)","Japonais","Coréen","Allemand","Français","Italien","Espagnol"),
        pages:labels!("Ajoute à chaque monstre du carnet de\r\nchasseur une dernière page avec zones,\r\naltérations, pièges et matériaux.","Les changements s'appliquent aussitôt et\r\nsont enregistrés dans\r\nbetter-monster-information.ini\r\nà la fermeture de la fenêtre."),
    },
    // it
    Text{
        names:labels!("Lingua","Layout pulsanti","Abilita registrazione log"),
        helps:labels!("Lingua della pagina info.","Icone dei pulsanti nei suggerimenti.","Scrive i log diagnostici. Avvisi ed errori sempre registrati."),
        on_off:labels!("No","Sì"),
        languages:labels!("Inglese","Cinese (S)","Cinese (T)","Giapponese","Coreano","Tedesco","Francese","Italiano","Spagnolo"),
        pages:labels!("Aggiunge a ogni mostro degli appunti\r\ndel cacciatore un'ultima pagina con\r\nzone, stati, trappole e materiali.","Le modifiche valgono subito e vengono\r\nsalvate in\r\nbetter-monster-information.ini\r\nalla chiusura della finestra."),
    },
    // es
    Text{
        names:labels!("Idioma","Diseño de botones","Activar registro de eventos"),
        helps:labels!("Idioma de la página de información.","Iconos de botones en las indicaciones.","Escribe registros de diagnóstico. Avisos y errores siempre."),
        on_off:labels!("No","Sí"),
        languages:labels!("Inglés","Chino (S)","Chino (T)","Japonés","Coreano","Alemán","Francés","Italiano","Español"),
        pages:labels!("Añade a cada monstruo de las notas del\r\ncazador una última página con zonas,\r\nestados, trampas y materiales.","Los cambios se aplican al instante y se\r\nguardan en\r\nbetter-monster-information.ini\r\nal cerrar la ventana."),
    },
];
/// Purple-lib passes the game's language (MT Framework order: 0 Japanese, 1 English, 2 French, 3 Spanish, 4 German,
/// 5 Italian, 6 Korean, 7 Chinese (T), 8 Chinese (S); 6 under the Korean patch). Unknown numbers read as English.
pub fn mod_language(game:u32)->usize{
    const MOD_LANGUAGE:[usize;9]=[3,0,6,8,5,7,4,2,1];
    MOD_LANGUAGE.get(game as usize).copied().unwrap_or(0)
}
fn text(language:u32)->&'static Text{&TEXT[mod_language(language)]}
static LAYOUTS:Labels<3>=labels!("Nintendo","Xbox","PlayStation");
const ROWS:u32=3;

unsafe extern "C" fn title(_:*mut c_void,_:u32)->*const c_char{c"Better Monster Information".as_ptr()}

unsafe extern "C" fn row(_:*mut c_void,language:u32,index:u32,out:*mut Row)->bool{
    let Some(out)=(unsafe{out.as_mut()}) else{return false};
    if index>=ROWS{return false}
    let t=text(language);
    out.name=t.names.0[index as usize];out.help=t.helps.0[index as usize];out.kind=ROW_LABELS;
    let(labels,count)=match index{0=>(t.languages.0.as_ptr(),9),1=>(LAYOUTS.0.as_ptr(),3),_=>(t.on_off.0.as_ptr(),2)};
    out.labels=labels;out.label_count=count;
    true
}

unsafe extern "C" fn get(_:*mut c_void,index:u32)->u32{
    STATE.with(|s|match index{0=>s.language.min(8) as u32,1=>s.layout.min(2) as u32,2=>s.logging as u32,_=>0})
}

unsafe extern "C" fn set(_:*mut c_void,index:u32,value:u32){
    match index{
        // The language pack and font glyphs switch on the next ImGui frame (ui::before_language_frame), which also saves
        0 if value<9=>{STATE.with(|s|s.language=value as usize);crate::ui::request_language(value as usize);}
        1 if value<3=>STATE.with(|s|{s.layout=value as usize;s.clock=0.;}),
        2=>{let on=value!=0;STATE.with(|s|s.logging=on);crate::diagnostics::set_enabled(on);}
        _=>{}
    }
}

unsafe extern "C" fn closed(_:*mut c_void){
    if !crate::storage::save_preferences(){forge::log::prelude::warn!("[monster-info] Mod Options: settings could not be saved");}
}

struct Record<T>(T);
unsafe impl<T> Sync for Record<T> {}
static OPTIONS:Record<Options>=Record(Options{
    size:core::mem::size_of::<Options>() as u32,context:core::ptr::null_mut(),title:Some(title),
    row_count:ROWS,row:Some(row),get:Some(get),set:Some(set),closed:Some(closed),value_text:None,action:None,
});

unsafe extern "C" fn help_page(_:*mut c_void,language:u32,index:u32)->*const c_char{
    text(language).pages.0.get(index as usize).copied().unwrap_or(core::ptr::null())
}
static SET_HELP:Record<Help>=Record(Help{
    size:core::mem::size_of::<Help>() as u32,context:core::ptr::null_mut(),category:0,
    page_count:2,title:Some(title),page:Some(help_page),open:None,
});

#[unsafe(no_mangle)]
pub unsafe extern "C" fn purplelib_on_register(api:*const Api){
    let Some(api)=(unsafe{api.as_ref()}) else{return};
    if api.version<API_VERSION{forge::log::prelude::warn!("[monster-info] Purple-lib is too old: no Mod Options page");return}
    let Some(add)=api.add_options else{return};
    if unsafe{add(&OPTIONS.0)}{
        if let Some(help)=api.add_set_help{unsafe{help(&OPTIONS.0,&SET_HELP.0);}}
        forge::log::prelude::info!("[monster-info] settings under Options > Mod Options");
    }else{
        forge::log::prelude::warn!("[monster-info] Purple-lib refused the Mod Options page");
    }
}
