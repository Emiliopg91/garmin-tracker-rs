import { Tabs } from "@/models/tabs";
import { AppEnvironment, DeviceListItem } from "@/utils/backend/models";
import { createContext } from "react";

interface AppContexType {
  appReady: boolean;
  tab: Tabs;
  setTab: (category: Tabs) => void;
  availableDevices: DeviceListItem[];
  environment: AppEnvironment;
  showSettings: () => void;
  closeSettings: () => void;
  settingsOpened: boolean;
}

const defaultValue: AppContexType = {
  appReady: false,
  tab: Tabs.HOME,
  setTab: () => {
    /* empty */
  },
  availableDevices: [],
  environment: AppEnvironment.Release,
  settingsOpened: false,
  closeSettings: () => {
    /* */
  },
  showSettings: () => {
    /* */
  },
};

export const AppContext = createContext(defaultValue);
