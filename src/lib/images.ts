// 剪贴板图片的缓存：PNG 数据从后端取来后转成 blob URL，同一张图不重复取
import { api } from "./api";

const cache = new Map<string, Promise<string>>();

export function clipImageUrl(id: number, thumb: boolean): Promise<string> {
  const key = `${id}:${thumb ? "t" : "f"}`;
  let p = cache.get(key);
  if (!p) {
    p = api.clipImage(id, thumb).then((buf) => URL.createObjectURL(new Blob([buf], { type: "image/png" })));
    p.catch(() => cache.delete(key));
    cache.set(key, p);
  }
  return p;
}
