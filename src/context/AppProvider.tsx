import { BackendClient } from "@/utils/backend/client";
import { BackendListener } from "@/utils/backend/listener";
import { AppEnvironment } from "@/utils/backend/models";
import { useContext, useEffect, useMemo, useState } from "react";
import { JSX } from "react/jsx-runtime";
import { AppContext } from "./AppContext";
import { I18nSettingsContext } from "./I18nSettingsContext";

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const { i18nReady } = useContext(I18nSettingsContext);
  const [environment, setEnvironment] = useState(AppEnvironment.Release);
  const [backendReady, setBackendReady] = useState(false);
  const [appReady, setAppReady] = useState(false);
  const [sessionsVersion, setSessionsVersion] = useState(0);

  useEffect(() => {
    const unregisterSessionsAdded = BackendListener.onSessionsAdded(() => {
      setSessionsVersion((previous) => previous + 1);
    });

    BackendClient.getEnvironment()
      .then((env) => {
        setEnvironment(env);

        if (env == AppEnvironment.Release) {
          document.addEventListener("contextmenu", (e) => {
            e.preventDefault();
          });
        }
      })
      .finally(() => {
        setBackendReady(true);
      });

    return () => {
      unregisterSessionsAdded();
    };
  }, []);

  // The backend is notified once both this provider and the i18n one have loaded
  useEffect(() => {
    if (backendReady && i18nReady) {
      BackendClient.notifyFrontendReady().then(() => {
        setAppReady(true);
      });
    }
  }, [backendReady, i18nReady]);

  const value = useMemo(
    () => ({
      appReady,
      environment,
      sessionsVersion,
    }),
    [appReady, environment, sessionsVersion],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}
