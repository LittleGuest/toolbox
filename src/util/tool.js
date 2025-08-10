import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { NIcon } from "naive-ui";

export const notification = async (body) => {
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === "granted";
  }
  if (permissionGranted) {
    sendNotification({ title: "Toolbox", body: body });
  }
};

export const renderIcon = (icon) => {
  // 如果是字符串路径，则渲染为SVG图像
  if (typeof icon === "string") {
    return () =>
      h("img", {
        src: icon,
        width: "20",
        height: "20",
        style: "vertical-align: middle;",
      });
  }
  return () => h(NIcon, null, { default: () => h(icon) });
};
