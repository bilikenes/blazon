export type SymbolDef = {
  id: string; // Gravity UI ikon adı
  label: string; // Türkçe görünen ad
  labelEn: string; // İngilizce görünen ad
  viewBox: string;
  body: string; // <svg> içeriği, renkler currentColor
};

export type SymbolCategory = {
  id: string;
  label: string;
  labelEn: string;
  icons: SymbolDef[];
};
