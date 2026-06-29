import { Tabs } from "@/models/tabs";
import { DeviceListItem } from "@/utils/backend/models";
import { createContext } from "react";

interface AppContexType {
  appReady: boolean;
  tab: Tabs;
  setTab: (category: Tabs) => void;
  loading: boolean;
  setLoading: (loading: boolean) => void;
  availableDevices: DeviceListItem[];
}

const defaultValue: AppContexType = {
  appReady: false,
  tab: Tabs.SESSIONS,
  setTab: () => {
    /* empty */
  },
  loading: false,
  setLoading: () => {
    /* empty */
  },
  availableDevices: [],
};

export const AppContext = createContext(defaultValue);
