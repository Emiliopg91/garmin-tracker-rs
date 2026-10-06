import { Tabs } from "@/models/tabs";
import { AppEnvironment, DeviceListItem } from "@/utils/backend/models";
import { createContext } from "react";

interface AppContexType {
  appReady: boolean;
  tab: Tabs;
  setTab: (category: Tabs) => void;
  registeredDevices: DeviceListItem[];
  availableDevices: DeviceListItem[];
  environment: AppEnvironment;
  showSettings: () => void;
  closeSettings: () => void;
  settingsOpened: boolean;
  sessionsVersion: number;
  rcloneAvailable: boolean;
}

const defaultValue: AppContexType = {
  appReady: false,
  tab: Tabs.HOME,
  setTab: () => {
    /* empty */
  },
  registeredDevices: [],
  availableDevices: [],
  environment: AppEnvironment.Release,
  settingsOpened: false,
  closeSettings: () => {
    /* */
  },
  showSettings: () => {
    /* */
  },
  sessionsVersion: 0,
  rcloneAvailable: false,
};

export const AppContext = createContext(defaultValue);
