import { createContext, JSX, useState } from "react";

export enum Categories {
  FEATURE = 0,
  RELEASE,
  BUGFIX,
  HOTFIX,
}

interface AppContexType {
  tab: Categories;
  setTab: (category: Categories) => void;
}

const defaultValue: AppContexType = {
  tab: Categories.FEATURE,
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
  const [tab, setTab] = useState(Categories.FEATURE);

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
