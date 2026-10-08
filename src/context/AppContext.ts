import { AppEnvironment } from "@/utils/backend/models";
import { createContext } from "react";

interface AppContexType {
  appReady: boolean;
  environment: AppEnvironment;
  sessionsVersion: number;
}

const defaultValue: AppContexType = {
  appReady: false,
  environment: AppEnvironment.Release,
  sessionsVersion: 0,
};

export const AppContext = createContext(defaultValue);
