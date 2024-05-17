
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

const notification = async (body) => {
  let permissionGranted = await isPermissionGranted();
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
  }
  if (permissionGranted) {
    sendNotification({ title: 'Toolbox', body: body });
  }
};

export {
notification 
};
