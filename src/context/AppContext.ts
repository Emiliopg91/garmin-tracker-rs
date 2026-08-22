import { Tabs } from "@/models/tabs";
import {
  AppEnvironment,
  DeviceListItem,
  DistanceUnit,
  Settings,
  WeightUnit,
} from "@/utils/backend/models";
import { createContext } from "react";

interface AppContexType {
  appReady: boolean;
  tab: Tabs;
  setTab: (category: Tabs) => void;
  loading: boolean;
  startLoading: () => void;
  finishLoading: () => void;
  availableDevices: DeviceListItem[];
  environment: AppEnvironment;
  settings: Settings;
  translate: (key: string, replacements?: string[]) => string;
}

const defaultValue: AppContexType = {
  appReady: false,
  tab: Tabs.SESSIONS,
  setTab: () => {
    /* empty */
  },
  loading: false,
  startLoading: () => {
    /* empty */
  },
  finishLoading: () => {
    /* empty */
  },
  availableDevices: [],
  environment: AppEnvironment.Release,
  translate: () => {
    return "";
  },
  settings: {
    distance_unit: DistanceUnit.Kilometers,
    weight_unit: WeightUnit.Kilograms,
    auto_sync: true,
  },
};

export const AppContext = createContext(defaultValue);
