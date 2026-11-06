//! 界面国际化：语言枚举 + 7 种语言的静态字符串表。
//!
//! - 英文是默认 / 回退语言；启动时按 Windows UI 语言自动选择，识别不到就回退英文。
//! - 含动态值的文案用占位符 `{hk}`（热键标签）、`{path}`（文件路径 / 错误），
//!   调用处用 `str::replace` 填充，避免在 const 表里做格式化。
//! - 这里只放用户可见文案；代码注释仍保持中文，不属于翻译范围。

use windows_sys::Win32::Foundation::HMODULE;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};

/// 支持的界面语言。判别值会写入 `AtomicU8` 供字体渲染读取，顺序即语言按钮的循环顺序。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Lang {
    En = 0,
    ZhCn = 1,
    ZhTw = 2,
    Ja = 3,
    De = 4,
    Es = 5,
    Fr = 6,
}

// Windows 主语言 ID（LANGID 的低字节）。
const LANG_CHINESE: u32 = 0x04;
const LANG_ENGLISH: u32 = 0x09;
const LANG_GERMAN: u32 = 0x07;
const LANG_SPANISH: u32 = 0x0A;
const LANG_FRENCH: u32 = 0x0C;
const LANG_JAPANESE: u32 = 0x11;
// 中文子语言（LANGID 的高字节）：0x01 为繁体（台 / 港 / 澳），其余为简体。
const SUBLANG_CHINESE_TRADITIONAL: u32 = 0x01;

impl Lang {
    pub const ALL: [Lang; 7] = [
        Lang::En,
        Lang::ZhCn,
        Lang::ZhTw,
        Lang::Ja,
        Lang::De,
        Lang::Es,
        Lang::Fr,
    ];

    pub fn from_u8(n: u8) -> Lang {
        for l in Lang::ALL {
            if (l as u8) == n {
                return l;
            }
        }
        Lang::En
    }

    /// 工具条语言按钮：点一下切到下一种语言。
    pub fn next(self) -> Lang {
        Lang::ALL[((self as usize) + 1) % Lang::ALL.len()]
    }

    /// 工具条语言按钮上的短标签（尽量短，避免挤占工具条宽度）。
    pub fn short(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::ZhCn => "简",
            Lang::ZhTw => "繁",
            Lang::Ja => "日",
            Lang::De => "DE",
            Lang::Es => "ES",
            Lang::Fr => "FR",
        }
    }

    /// 该语言的母语名，切换时弹给用户看。
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::ZhCn => "简体中文",
            Lang::ZhTw => "繁體中文",
            Lang::Ja => "日本語",
            Lang::De => "Deutsch",
            Lang::Es => "Español",
            Lang::Fr => "Français",
        }
    }

    /// 首选正文字体。GDI 的字体链接（font linking）会自动补齐首选字体缺的字形，
    /// 因此即便某个字形不在该字体里也能显示，这里只是给每种语言挑最协调的字面。
    pub fn font_face(self) -> &'static str {
        match self {
            Lang::En | Lang::De | Lang::Es | Lang::Fr => "Segoe UI",
            Lang::ZhCn => "Microsoft YaHei",
            Lang::ZhTw => "Microsoft JhengHei",
            Lang::Ja => "Yu Gothic",
        }
    }

    /// 启动时按当前用户的 Windows UI 语言选择界面语言，识别不到回退英文。
    /// 用 GetProcAddress 动态取 kernel32!GetUserDefaultUILanguage，避免为此引入额外的
    /// windows-sys feature；该函数自 Win2000 起稳定存在，取不到时安全回退英文。
    pub fn detect() -> Lang {
        let langid = unsafe { raw_ui_langid() };
        let Some(langid) = langid else { return Lang::En };
        let primary = langid & 0xFF;
        let sublang = (langid >> 8) & 0xFF;
        match primary {
            LANG_CHINESE => {
                if sublang == SUBLANG_CHINESE_TRADITIONAL {
                    Lang::ZhTw
                } else {
                    Lang::ZhCn
                }
            }
            LANG_JAPANESE => Lang::Ja,
            LANG_GERMAN => Lang::De,
            LANG_SPANISH => Lang::Es,
            LANG_FRENCH => Lang::Fr,
            LANG_ENGLISH => Lang::En,
            _ => Lang::En,
        }
    }
}

/// 动态调用 kernel32!GetUserDefaultUILanguage 取当前用户 UI 语言的 LANGID。
unsafe fn raw_ui_langid() -> Option<u32> {
    let module_name: Vec<u16> = "kernel32.dll\0".encode_utf16().collect();
    let hmodule: HMODULE = GetModuleHandleW(module_name.as_ptr());
    if hmodule.is_null() {
        return None;
    }
    let proc_name: Vec<u8> = b"GetUserDefaultUILanguage\0".to_vec();
    let far = GetProcAddress(hmodule, proc_name.as_ptr())?;
    Some(far() as u32)
}

/// 某一语言下的全部用户可见文案。
pub struct Strings {
    /// 工具条工具名，顺序与 `ui::Tool` 一一对应。
    pub tools: [&'static str; 7],
    /// 8 个色块的名称，顺序与 `ui::COLORS` 一一对应。
    pub colors: [&'static str; 8],
    pub undo: &'static str,
    pub redo: &'static str,
    pub clear: &'static str,
    pub save: &'static str,
    pub freeze: &'static str,
    pub mouse: &'static str,
    pub help: &'static str,
    pub exit: &'static str,
    /// 帮助卡片 9 行；第 6 行含 `{hk}` 占位符。
    pub help_lines: [&'static str; 9],
    pub input_hint: &'static str,
    /// 文字输入框为空时的占位提示。
    pub placeholder: &'static str,
    /// 从鼠标穿透态回到标注模式时的提示。
    pub back_to_ink: &'static str,
    /// 进入鼠标模式、且全局热键可用时的提示，含 `{hk}`。
    pub mouse_hotkey: &'static str,
    /// 进入鼠标模式、但全局热键被占用时的提示。
    pub mouse_no_hotkey: &'static str,
    /// 保存成功提示，含 `{path}`。
    pub saved: &'static str,
    /// 保存失败提示，含 `{path}`（实际填入错误信息）。
    pub save_failed: &'static str,
    /// 热键标签在两个热键都注册失败时的兜底文案。
    pub click_toolbar: &'static str,
    /// 启动即发现全局热键被占用时的提示。
    pub hotkey_busy: &'static str,
}

impl Strings {
    /// 拼出帮助卡片的实际行，把 `{hk}` 替换成当前可用的热键标签。
    pub fn help_lines(&self, hotkey_label: &str) -> Vec<String> {
        self.help_lines.iter().map(|l| l.replace("{hk}", hotkey_label)).collect()
    }
}

pub fn strings(lang: Lang) -> &'static Strings {
    match lang {
        Lang::En => &EN,
        Lang::ZhCn => &ZH_CN,
        Lang::ZhTw => &ZH_TW,
        Lang::Ja => &JA,
        Lang::De => &DE,
        Lang::Es => &ES,
        Lang::Fr => &FR,
    }
}

static EN: Strings = Strings {
    tools: ["Pen", "Line", "Box", "Oval", "Arrow", "Text", "Eraser"],
    colors: ["Red", "Orange", "Yellow", "Green", "Teal", "Blue", "Purple", "White"],
    undo: "Undo",
    redo: "Redo",
    clear: "Clear",
    save: "Save",
    freeze: "Freeze",
    mouse: "Mouse",
    help: "Help",
    exit: "Exit",
    help_lines: [
        "ScreenInk  Screen Annotation",
        "P Pen  L Line  R Box  O Oval  A Arrow  T Text  E Eraser",
        "1-8 Color    Wheel / [ ] Stroke width",
        "Ctrl+Z Undo   Ctrl+Y Redo   C Clear",
        "M Mouse mode (click-through, UI unchanged)",
        "Click toolbar / {hk} to return to drawing",
        "S Save   F Freeze screen   Ctrl+V Paste text",
        "H Help   Tab Toolbar   Esc Exit",
        "Language button on toolbar: switch language",
    ],
    input_hint: "Type text: Enter to confirm · Esc to cancel · Ctrl+V to paste",
    placeholder: "Type here…",
    back_to_ink: "Back to annotation mode",
    mouse_hotkey: "Mouse mode · Click toolbar or {hk} to return",
    mouse_no_hotkey: "Mouse mode · Click toolbar to return (global hotkey unavailable)",
    saved: "Saved  {path}",
    save_failed: "Save failed  {path}",
    click_toolbar: "Click toolbar",
    hotkey_busy: "Global hotkey unavailable · In mouse mode click the toolbar to return",
};

static ZH_CN: Strings = Strings {
    tools: ["笔", "线", "框", "圆", "箭", "字", "擦"],
    colors: ["红", "橙", "黄", "绿", "青", "蓝", "紫", "白"],
    undo: "撤销",
    redo: "重做",
    clear: "清空",
    save: "保存",
    freeze: "冻结",
    mouse: "鼠标",
    help: "帮助",
    exit: "退出",
    help_lines: [
        "ScreenInk  屏幕标注",
        "P 笔  L 线  R 框  O 圆  A 箭  T 字  E 擦",
        "1-8 换颜色    滚轮 / [ ] 调粗细",
        "Ctrl+Z 撤销   Ctrl+Y 重做   C 清空",
        "M 鼠标模式(穿透交互·界面不变)",
        "点工具条 / {hk} 返回绘图",
        "S 保存   F 冻结屏幕   Ctrl+V 粘贴文字",
        "H 帮助   Tab 工具条   Esc 退出",
        "工具条语言按钮：切换界面语言",
    ],
    input_hint: "输入文字：Enter 确认 · Esc 取消 · Ctrl+V 粘贴",
    placeholder: "在这里输入文字…",
    back_to_ink: "已返回标注模式",
    mouse_hotkey: "鼠标模式 · 点工具条 或 {hk} 返回绘图",
    mouse_no_hotkey: "鼠标模式 · 点工具条返回绘图（全局热键被占用）",
    saved: "已保存  {path}",
    save_failed: "保存失败  {path}",
    click_toolbar: "点工具条",
    hotkey_busy: "全局热键被占用 · 进入鼠标模式后请点工具条返回",
};

static ZH_TW: Strings = Strings {
    tools: ["筆", "線", "框", "圓", "箭", "字", "擦"],
    colors: ["紅", "橙", "黃", "綠", "青", "藍", "紫", "白"],
    undo: "復原",
    redo: "重做",
    clear: "清除",
    save: "儲存",
    freeze: "凍結",
    mouse: "滑鼠",
    help: "說明",
    exit: "離開",
    help_lines: [
        "ScreenInk  螢幕標註",
        "P 筆  L 線  R 框  O 圓  A 箭  T 字  E 擦",
        "1-8 換顏色    滾輪 / [ ] 調粗細",
        "Ctrl+Z 復原   Ctrl+Y 重做   C 清除",
        "M 滑鼠模式（穿透互動·介面不變）",
        "點工具列 / {hk} 返回繪圖",
        "S 儲存   F 凍結螢幕   Ctrl+V 貼上文字",
        "H 說明   Tab 工具列   Esc 離開",
        "工具列語言按鈕：切換介面語言",
    ],
    input_hint: "輸入文字：Enter 確認 · Esc 取消 · Ctrl+V 貼上",
    placeholder: "在這裡輸入文字…",
    back_to_ink: "已返回標註模式",
    mouse_hotkey: "滑鼠模式 · 點工具列 或 {hk} 返回繪圖",
    mouse_no_hotkey: "滑鼠模式 · 點工具列返回繪圖（全域快捷鍵被占用）",
    saved: "已儲存  {path}",
    save_failed: "儲存失敗  {path}",
    click_toolbar: "點工具列",
    hotkey_busy: "全域快捷鍵被占用 · 進入滑鼠模式後請點工具列返回",
};

static JA: Strings = Strings {
    tools: ["ペン", "線", "枠", "円", "矢印", "文字", "消去"],
    colors: ["赤", "橙", "黄", "緑", "青緑", "青", "紫", "白"],
    undo: "戻る",
    redo: "やり直し",
    clear: "クリア",
    save: "保存",
    freeze: "固定",
    mouse: "マウス",
    help: "ヘルプ",
    exit: "終了",
    help_lines: [
        "ScreenInk  画面注釈",
        "P ペン  L 線  R 枠  O 円  A 矢印  T 文字  E 消去",
        "1-8 色切替    ホイール / [ ] 太さ",
        "Ctrl+Z 戻る   Ctrl+Y やり直し   C クリア",
        "M マウスモード（透過操作・UI はそのまま）",
        "ツールバー / {hk} で描画に戻る",
        "S 保存   F 画面固定   Ctrl+V テキスト貼付",
        "H ヘルプ   Tab ツールバー   Esc 終了",
        "ツールバーの言語ボタン：言語を切替",
    ],
    input_hint: "テキスト入力：Enter で確定 · Esc で取消 · Ctrl+V で貼り付け",
    placeholder: "ここに入力…",
    back_to_ink: "注釈モードに戻りました",
    mouse_hotkey: "マウスモード · ツールバーまたは {hk} で描画に戻る",
    mouse_no_hotkey: "マウスモード · ツールバーをクリックして復帰（グローバル Hotkey 使用不可）",
    saved: "保存しました  {path}",
    save_failed: "保存に失敗  {path}",
    click_toolbar: "ツールバーをクリック",
    hotkey_busy: "グローバル Hotkey 使用不可 · マウスモード中はツールバーをクリックして復帰",
};

static DE: Strings = Strings {
    tools: ["Stift", "Linie", "Box", "Ellipse", "Pfeil", "Text", "Radierer"],
    colors: ["Rot", "Orange", "Gelb", "Grün", "Türkis", "Blau", "Lila", "Weiß"],
    undo: "Rückgängig",
    redo: "Wiederholen",
    clear: "Leeren",
    save: "Speichern",
    freeze: "Fixieren",
    mouse: "Maus",
    help: "Hilfe",
    exit: "Beenden",
    help_lines: [
        "ScreenInk  Bildschirm-Annotation",
        "P Stift  L Linie  R Box  O Ellipse  A Pfeil  T Text  E Radierer",
        "1-8 Farbe    Mausrad / [ ] Stiftstärke",
        "Strg+Z Rückgängig   Strg+Y Wiederholen   C Leeren",
        "M Mausmodus (Klick-durch, UI unverändert)",
        "Symbolleiste / {hk} zurück zum Zeichnen",
        "S Speichern   F Bildschirm fixieren   Strg+V Text einfügen",
        "H Hilfe   Tab Symbolleiste   Esc Beenden",
        "Sprachschalter in der Symbolleiste",
    ],
    input_hint: "Text eingeben: Enter bestätigen · Esc abbrechen · Strg+V einfügen",
    placeholder: "Hier tippen…",
    back_to_ink: "Zurück im Annotationsmodus",
    mouse_hotkey: "Mausmodus · Symbolleiste oder {hk} zum Zurückkehren",
    mouse_no_hotkey: "Mausmodus · Symbolleiste klicken (globaler Hotkey belegt)",
    saved: "Gespeichert  {path}",
    save_failed: "Speichern fehlgeschlagen  {path}",
    click_toolbar: "Symbolleiste klicken",
    hotkey_busy: "Globaler Hotkey belegt · Im Mausmodus Symbolleiste klicken",
};

static ES: Strings = Strings {
    tools: ["Lápiz", "Línea", "Rect", "Óvalo", "Flecha", "Texto", "Borrador"],
    colors: ["Rojo", "Naranja", "Amarillo", "Verde", "Turquesa", "Azul", "Morado", "Blanco"],
    undo: "Deshacer",
    redo: "Rehacer",
    clear: "Borrar",
    save: "Guardar",
    freeze: "Congelar",
    mouse: "Ratón",
    help: "Ayuda",
    exit: "Salir",
    help_lines: [
        "ScreenInk  Anotación de pantalla",
        "P Lápiz  L Línea  R Rect  O Óvalo  A Flecha  T Texto  E Borrador",
        "1-8 Color    Rueda / [ ] Grosor",
        "Ctrl+Z Deshacer   Ctrl+Y Rehacer   C Borrar",
        "M Modo ratón (clic a través, interfaz intacta)",
        "Barra / {hk} para volver a dibujar",
        "S Guardar   F Congelar pantalla   Ctrl+V Pegar texto",
        "H Ayuda   Tab Barra   Esc Salir",
        "Botón de idioma en la barra: cambiar idioma",
    ],
    input_hint: "Escriba texto: Enter confirmar · Esc cancelar · Ctrl+V pegar",
    placeholder: "Escriba aquí…",
    back_to_ink: "Vuelta al modo de anotación",
    mouse_hotkey: "Modo ratón · Barra o {hk} para volver",
    mouse_no_hotkey: "Modo ratón · Haga clic en la barra (atajo global ocupado)",
    saved: "Guardado  {path}",
    save_failed: "Error al guardar  {path}",
    click_toolbar: "Barra de herramientas",
    hotkey_busy: "Atajo global ocupado · En modo ratón haga clic en la barra",
};

static FR: Strings = Strings {
    tools: ["Crayon", "Ligne", "Rect", "Ovale", "Flèche", "Texte", "Gomme"],
    colors: ["Rouge", "Orange", "Jaune", "Vert", "Turquoise", "Bleu", "Violet", "Blanc"],
    undo: "Annuler",
    redo: "Rétablir",
    clear: "Effacer",
    save: "Enregistrer",
    freeze: "Geler",
    mouse: "Souris",
    help: "Aide",
    exit: "Quitter",
    help_lines: [
        "ScreenInk  Annotation d'écran",
        "P Crayon  L Ligne  R Rect  O Ovale  A Flèche  T Texte  E Gomme",
        "1-8 Couleur    Molette / [ ] Épaisseur",
        "Ctrl+Z Annuler   Ctrl+Y Rétablir   C Effacer",
        "M Mode souris (clic à travers, interface inchangée)",
        "Barre d'outils / {hk} pour revenir au dessin",
        "S Enregistrer   F Geler l'écran   Ctrl+V Coller le texte",
        "H Aide   Tab Barre d'outils   Esc Quitter",
        "Bouton de langue dans la barre : changer de langue",
    ],
    input_hint: "Saisir le texte : Entrée valider · Échap annuler · Ctrl+V coller",
    placeholder: "Saisir ici…",
    back_to_ink: "Retour en mode annotation",
    mouse_hotkey: "Mode souris · Barre d'outils ou {hk} pour revenir",
    mouse_no_hotkey: "Mode souris · Cliquez la barre (raccourci global occupé)",
    saved: "Enregistré  {path}",
    save_failed: "Échec de l'enregistrement  {path}",
    click_toolbar: "Barre d'outils",
    hotkey_busy: "Raccourci global occupé · En mode souris cliquez la barre d'outils",
};
