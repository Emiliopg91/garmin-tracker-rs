import { Tabs } from "@/models/tabs";
import { createContext, JSX, useState } from "react";

interface AppContexType {
  tab: Tabs;
  setTab: (category: Tabs) => void;
}

const defaultValue: AppContexType = {
  tab: Tabs.SESSIONS,
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
  const [tab, setTab] = useState(Tabs.SESSIONS);

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
