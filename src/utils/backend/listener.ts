//Auto generated file, do not edit manually

import { listen } from "@tauri-apps/api/event";

import { DeviceListItem } from "./models";

export class BackendListener {
	public static onDeviceConnected(callback: (payload: DeviceListItem) => void): () => void {
	  return BackendListener.inner_listen<DeviceListItem>("device_connected", callback);
	}

	public static onDeviceDisconnected(callback: (payload: DeviceListItem) => void): () => void {
	  return BackendListener.inner_listen<DeviceListItem>("device_disconnected", callback);
	}

  private static inner_listen<R>(event_name: string, callback: (payload: R) => void ): () => void {
    const unlisten = listen<R>(event_name, (event) => {
      callback(event.payload);
    });

    return () => { unlisten.then((fn) => fn()); };
  }
}