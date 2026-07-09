//Auto generated file, do not edit manually

import { listen } from "@tauri-apps/api/event";

import { DeviceListItem } from "./models";

export class BackendListener {
  public static onDeviceConnected(
    callback: (payload: DeviceListItem) => void,
  ): () => void {
    return BackendListener.inner_listen<DeviceListItem>(
      "device_connected",
      callback,
    );
  }

  public static onDeviceDisconnected(
    callback: (payload: DeviceListItem) => void,
  ): () => void {
    return BackendListener.inner_listen<DeviceListItem>(
      "device_disconnected",
      callback,
    );
  }

  public static onUpdateAvailable(
    callback: (payload: string) => void,
  ): () => void {
    return BackendListener.inner_listen<string>("update_available", callback);
  }

  private static inner_listen<R>(
    event_name: string,
    callback: (payload: R) => void,
  ): () => void {
    console.debug("Listening to event '" + event_name + "'");
    const unlisten = listen<R>(event_name, (event) => {
      console.debug(
        "Received event for '" + event_name + "', payload: ",
        event.payload,
      );
      callback(event.payload);
    });

    return () => {
      console.debug("Stopping listening to event '" + event_name + "'");
      unlisten.then((fn) => fn());
    };
  }
}
