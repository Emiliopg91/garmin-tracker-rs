import {
  DistanceUnit,
  Languages,
  Settings,
  WeightUnit,
} from "@/utils/backend/models";
import { createContext } from "react";

interface I18nSettingsContextType {
  settings: Settings;
  translate: (key: string, replacements?: string[]) => string;
  refreshTranslations: () => void;
}

const defaultValue: I18nSettingsContextType = {
  settings: {
    distance_unit: DistanceUnit.Kilometers,
    weight_unit: WeightUnit.Kilograms,
    auto_sync: true,
    start_boot: false,
    language: Languages.English,
    on_device_connect: false,
  },
  translate: () => {
    return "";
  },
  refreshTranslations: () => {
    /* empty */
  },
};

export const I18nSettingsContext = createContext(defaultValue);
