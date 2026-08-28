import { Tabs } from "@/models/tabs";
import {
  AppEnvironment,
  DeviceListItem,
  DistanceUnit,
  Languages,
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
  refreshTranslations: () => void;
  showSettings: () => void;
  closeSettings: () => void;
  settingsOpened: boolean;
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
  refreshTranslations: () => {
    /* */
  },
  settings: {
    distance_unit: DistanceUnit.Kilometers,
    weight_unit: WeightUnit.Kilograms,
    auto_sync: true,
    start_boot: false,
    language: Languages.English,
  },
  settingsOpened: false,
  closeSettings: () => {
    /* */
  },
  showSettings: () => {
    /* */
  },
};

export const AppContext = createContext(defaultValue);
