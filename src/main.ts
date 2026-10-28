import { mount, type Component } from "svelte";
import "./app.css";
import App from "./App.svelte";
import ClipPanel from "./windows/ClipPanel.svelte";
import Shot from "./windows/Shot.svelte";
import Pin from "./windows/Pin.svelte";
import Launcher from "./windows/Launcher.svelte";

// 所有窗口共用一个页面，按地址里的 # 区分：#panel 剪贴板面板，#launcher 启动器，#shot/0 截图，#pin/3 贴图
const kind = location.hash.slice(1).split("/")[0];
const views: Record<string, Component> = { panel: ClipPanel, launcher: Launcher, shot: Shot, pin: Pin };
document.documentElement.classList.toggle("panel", kind in views);
const app = mount(views[kind] ?? App, { target: document.getElementById("app")! });
// 启动画面交给界面
document.getElementById("boot")?.remove();

export default app;
