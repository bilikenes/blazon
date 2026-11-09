// Arayüz metinleri (Türkçe + İngilizce). Dil, Windows arayüz diline göre seçilir;
// başlık çubuğundaki düğmeyle değiştirilebilir (seçim bu bilgisayarda hatırlanır).

export type Lang = "tr" | "en";

const STORAGE_KEY = "blazon.lang";

function detect(): Lang {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved === "tr" || saved === "en") return saved;
  } catch {
    /* depolama yoksa sistem diline düş */
  }
  return navigator.language.toLowerCase().startsWith("tr") ? "tr" : "en";
}

export const lang: Lang = detect();
document.documentElement.lang = lang;

/** Dili değiştirir ve arayüzü yeniden yükler (metinler modül yüklenirken seçilir). */
export function switchLang(next: Lang) {
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    /* yoksay */
  }
  location.reload();
}

const tr = {
  "win.minimize": "Küçült",
  "win.maximize": "Büyüt",
  "win.close": "Kapat",
  "win.language": "Dil: Türkçe — English'e geç",

  "stage.drag": "Bir klasörü pencereye sürükle",
  "stage.drop": "Bırak",
  "stage.pick": "Klasör seç",
  "stage.change": "Değiştir",
  "stage.onlyInApp": "Klasör seçimi yalnızca uygulamada çalışır",

  "color.label": "Renk",
  "color.folder": "Klasör",
  "color.symbol": "Sembol",
  "color.custom": "Özel renk",
  "color.customName": "Özel · {hex}",
  "color.done": "Tamam",
  "color.auto": "Otomatik",
  "color.sky": "Gök",
  "color.indigo": "Çivit",
  "color.violet": "Mor",
  "color.rose": "Gül",
  "color.coral": "Mercan",
  "color.amber": "Kehribar",
  "color.sage": "Adaçayı",
  "color.sea": "Deniz",
  "color.sand": "Kum",
  "color.slate": "Arduvaz",
  "color.graphite": "Grafit",
  "color.white": "Beyaz",
  "color.night": "Gece",

  "symbol.label": "Sembol",
  "symbol.none": "Yok",
  "symbol.noneTitle": "Sembol yok",
  "symbol.search": "Sembol ara",
  "symbol.searchShort": "Ara",
  "symbol.closeSearch": "Aramayı kapat",
  "symbol.noResults": "“{q}” için sonuç yok",

  "action.apply": "Uygula",
  "action.reset": "Varsayılana döndür",
  "toast.applied": "{name} güncellendi",
  "toast.reset": "{name} varsayılana döndü",

  "error.not_a_folder": "Bu bir klasör değil ya da artık yok.",
  "error.drive_root": "Sürücülerin (C:\\ gibi) ikonu değiştirilemez. İçindeki bir klasörü seç.",
  "error.access_denied": "Bu klasöre yazma iznin yok. Program Files ve Windows gibi sistem klasörleri korunur.",
  "error.read_only_media": "Bu sürücü salt okunur, ikon yazılamıyor.",
  "error.in_use": "Klasördeki bir dosya başka bir programda açık. Kapatıp tekrar dene.",
  "error.invalid_image": "İkon oluşturulamadı. Lütfen tekrar dene.",
  "error.shell_rejected": "Windows bu klasörün ayarını kabul etmedi.",
  "error.io": "Beklenmeyen bir dosya hatası oluştu.",
  "error.unknown": "Bir şeyler ters gitti. Lütfen tekrar dene.",

  "about.title": "Blazon hakkında",
  "about.version": "Sürüm {version}",
  "about.tagline": "Klasörlerine renk ver.",
  "about.report": "Sorun bildir",
  "about.license": "MIT lisansı",

  "update.check": "Güncellemeleri denetle",
  "update.checking": "Denetleniyor…",
  "update.latest": "En güncel sürümü kullanıyorsun",
  "update.available": "Yeni sürüm hazır: {version}",
  "update.pill": "Güncelle · {version}",
  "update.installNow": "{version} sürümünü kur",
  "update.downloading": "İndiriliyor…",
  "update.downloadingPct": "İndiriliyor · %{pct}",
  "update.checkFailed": "Güncelleme denetlenemedi. İnternet bağlantını kontrol et.",
  "update.installFailed": "Güncelleme kurulamadı. Daha sonra tekrar dene.",
};

type Key = keyof typeof tr;

const en: Record<Key, string> = {
  "win.minimize": "Minimize",
  "win.maximize": "Maximize",
  "win.close": "Close",
  "win.language": "Language: English — Türkçe'ye geç",

  "stage.drag": "Drag a folder onto the window",
  "stage.drop": "Drop it",
  "stage.pick": "Choose folder",
  "stage.change": "Change",
  "stage.onlyInApp": "Choosing a folder only works in the app",

  "color.label": "Colour",
  "color.folder": "Folder",
  "color.symbol": "Symbol",
  "color.custom": "Custom colour",
  "color.customName": "Custom · {hex}",
  "color.done": "Done",
  "color.auto": "Auto",
  "color.sky": "Sky",
  "color.indigo": "Indigo",
  "color.violet": "Violet",
  "color.rose": "Rose",
  "color.coral": "Coral",
  "color.amber": "Amber",
  "color.sage": "Sage",
  "color.sea": "Sea",
  "color.sand": "Sand",
  "color.slate": "Slate",
  "color.graphite": "Graphite",
  "color.white": "White",
  "color.night": "Night",

  "symbol.label": "Symbol",
  "symbol.none": "None",
  "symbol.noneTitle": "No symbol",
  "symbol.search": "Search symbols",
  "symbol.searchShort": "Search",
  "symbol.closeSearch": "Close search",
  "symbol.noResults": "No results for “{q}”",

  "action.apply": "Apply",
  "action.reset": "Reset to default",
  "toast.applied": "{name} updated",
  "toast.reset": "{name} reset to default",

  "error.not_a_folder": "That isn't a folder, or it no longer exists.",
  "error.drive_root": "Drives (like C:\\) can't get a folder icon. Pick a folder inside it.",
  "error.access_denied": "You don't have permission to change this folder. System folders like Program Files and Windows are protected.",
  "error.read_only_media": "This drive is read-only, so the icon can't be saved.",
  "error.in_use": "A file in this folder is open in another program. Close it and try again.",
  "error.invalid_image": "Couldn't create the icon. Please try again.",
  "error.shell_rejected": "Windows didn't accept the setting for this folder.",
  "error.io": "An unexpected file error occurred.",
  "error.unknown": "Something went wrong. Please try again.",

  "about.title": "About Blazon",
  "about.version": "Version {version}",
  "about.tagline": "Give your folders some colour.",
  "about.report": "Report a problem",
  "about.license": "MIT licence",

  "update.check": "Check for updates",
  "update.checking": "Checking…",
  "update.latest": "You're on the latest version",
  "update.available": "New version ready: {version}",
  "update.pill": "Update · {version}",
  "update.installNow": "Install {version}",
  "update.downloading": "Downloading…",
  "update.downloadingPct": "Downloading · {pct}%",
  "update.checkFailed": "Couldn't check for updates. Check your internet connection.",
  "update.installFailed": "The update couldn't be installed. Please try again later.",
};

const dict: Record<Key, string> = lang === "tr" ? tr : en;

export function t(key: Key, vars?: Record<string, string>): string {
  let s = dict[key];
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replace(`{${k}}`, v);
  return s;
}

/** Komutlardan gelen hatayı ({code, detail}) kullanıcı diline çevirir; ayrıntıyı konsola yazar. */
export function errorMessage(err: unknown): string {
  console.error(err);
  const code = typeof err === "object" && err && "code" in err ? String((err as { code: unknown }).code) : "";
  const key = `error.${code}` as Key;
  return key in dict ? dict[key] : t("error.unknown");
}

/** Sembol/kategori etiketini seçili dilde döndürür. */
export const pick = (item: { label: string; labelEn: string }) => (lang === "tr" ? item.label : item.labelEn);
