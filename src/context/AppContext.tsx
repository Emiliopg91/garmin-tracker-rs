import { Tabs } from "@/models/tabs";
import { createContext, JSX, useState } from "react";

interface AppContexType {
  tab: Tabs;
  setTab: (category: Tabs) => void;
}

const defaultValue: AppContexType = {
  tab: Tabs.WORKOUTS,
  setTab: () => {
    /* empty */
  },
};

export const AppContext = createContext(defaultValue);

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const [tab, setTab] = useState(Tabs.WORKOUTS);

  return (
    <AppContext.Provider
      value={{
        tab,
        setTab,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}
