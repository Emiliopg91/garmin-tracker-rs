import { ALERT_DURATION, AlertDefinition } from "@/models/alert";
import { Tabs } from "@/models/tabs";
import { createContext, JSX, useState } from "react";

interface AppContexType {
  tab: Tabs;
  setTab: (category: Tabs) => void;
  alerts: AlertDefinition[];
  addAlert: (definition: AlertDefinition) => void;
}

const defaultValue: AppContexType = {
  tab: Tabs.WORKOUTS,
  setTab: () => {
    /* empty */
  },
  alerts: [],
  addAlert: (_: AlertDefinition) => {
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
  const [alerts, setAlerts] = useState<Record<string, AlertDefinition>>({});

  const addAlert = (definition: AlertDefinition) => {
    const k = Date.now() + "-" + Math.floor(Math.random() * 1000);
    setAlerts((prev) => {
      return {
        ...prev,
        [k]: definition,
      };
    });
    setTimeout(() => {
      setAlerts((prev) => {
        const next = {
          ...prev,
        };
        delete next[k];

        return next;
      });
    }, ALERT_DURATION[definition.type]);
  };

  return (
    <AppContext.Provider
      value={{
        tab,
        setTab,
        alerts: Object.entries(alerts)
          .sort(
            ([aKey], [bKey]) =>
              Number(bKey.split("-")[0]) - Number(aKey.split("-")[0]),
          ) // orden ascendente
          .map(([, def]) => def),
        addAlert,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}
