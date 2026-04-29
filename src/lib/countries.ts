// 仅常用国家 emoji 映射；缺失返回 🌐
export function flagOf(code?: string | null): string {
  if (!code || code.length !== 2) return "🌐";
  const cc = code.toUpperCase();
  return String.fromCodePoint(...[...cc].map((c) => 0x1f1a5 + c.charCodeAt(0)));
}

/** 常用国家/地区清单：ISO alpha-2 + 中文名（按 Claude 可用度 / 用户可能性排序） */
export const COUNTRY_LIST: { code: string; name: string }[] = [
  { code: "US", name: "美国" },
  { code: "JP", name: "日本" },
  { code: "SG", name: "新加坡" },
  { code: "TW", name: "台湾" },
  { code: "HK", name: "香港" },
  { code: "KR", name: "韩国" },
  { code: "CA", name: "加拿大" },
  { code: "GB", name: "英国" },
  { code: "AU", name: "澳大利亚" },
  { code: "NZ", name: "新西兰" },
  { code: "DE", name: "德国" },
  { code: "FR", name: "法国" },
  { code: "NL", name: "荷兰" },
  { code: "IE", name: "爱尔兰" },
  { code: "CH", name: "瑞士" },
  { code: "SE", name: "瑞典" },
  { code: "NO", name: "挪威" },
  { code: "FI", name: "芬兰" },
  { code: "DK", name: "丹麦" },
  { code: "BE", name: "比利时" },
  { code: "AT", name: "奥地利" },
  { code: "ES", name: "西班牙" },
  { code: "IT", name: "意大利" },
  { code: "PT", name: "葡萄牙" },
  { code: "PL", name: "波兰" },
  { code: "CZ", name: "捷克" },
  { code: "IL", name: "以色列" },
  { code: "AE", name: "阿联酋" },
  { code: "TR", name: "土耳其" },
  { code: "IN", name: "印度" },
  { code: "TH", name: "泰国" },
  { code: "MY", name: "马来西亚" },
  { code: "ID", name: "印度尼西亚" },
  { code: "PH", name: "菲律宾" },
  { code: "VN", name: "越南" },
  { code: "MX", name: "墨西哥" },
  { code: "BR", name: "巴西" },
  { code: "AR", name: "阿根廷" },
  { code: "CL", name: "智利" },
  { code: "ZA", name: "南非" },
];

export function nameOf(code: string): string {
  const found = COUNTRY_LIST.find((c) => c.code === code.toUpperCase());
  return found?.name ?? code;
}
