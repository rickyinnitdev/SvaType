export function toKhmerDigits(value: number | string): string {
  return String(value).replace(/[0-9]/g, (c) => '០១២៣៤៥៦៧៨៩'[Number(c)]);
}
export function formatDate(timestamp: number, language: 'km' | 'en'): string {
  const d = new Date(timestamp);
  if (!Number.isFinite(d.getTime())) return '—';
  const pad = (n: number) => String(n).padStart(2, '0');
  const text = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
  return language === 'km' ? toKhmerDigits(text) : text;
}
