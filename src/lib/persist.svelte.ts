// 工具里的输入内容存在 localStorage 里，切换工具、重启应用后还在

/** 一个会自动保存的响应式值：const s = persisted("json.input", ""); s.value = "..." */
export function persisted<T>(key: string, init: T) {
  const full = "tooltree.tool." + key;
  let initial = init;
  try {
    const raw = localStorage.getItem(full);
    if (raw !== null) initial = JSON.parse(raw);
  } catch {
    // 读不出来就用默认值
  }
  let value = $state(initial);
  let timer: ReturnType<typeof setTimeout> | undefined;
  return {
    get value() {
      return value;
    },
    set value(v: T) {
      value = v;
      // 输入时连续变化，稍等一下再写，不用每个字都写一次
      clearTimeout(timer);
      timer = setTimeout(() => {
        try {
          localStorage.setItem(full, JSON.stringify(v));
        } catch {
          // 内容太大存不下就算了
        }
      }, 300);
    },
  };
}
