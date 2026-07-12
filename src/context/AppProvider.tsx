import { AppContext } from "./AppContext";
import { AppEnvironment, DeviceListItem } from "@/utils/backend/models";
import { JSX } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import { Tabs } from "@/models/tabs";
import { BackendListener } from "@/utils/backend/listener";
import { BackendClient } from "@/utils/backend/client";

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const [environment, setEnvironment] = useState(AppEnvironment.Release);
  const [appReady, setAppReady] = useState(false);
  const [tab, setTab] = useState(Tabs.SESSIONS);
  const [loading, setLoading] = useState(false);
  const [translations, setTranslations] = useState<Record<string, string>>({});
  const [availableUpdate, setAvailableUpdate] = useState<string | undefined>(
    undefined,
  );
  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );
  const availableDevicesRef = useRef<DeviceListItem[]>([]);

  useEffect(() => {
    const unregisterConnection = BackendListener.onDeviceConnected((device) => {
      const previous = availableDevicesRef.current;
      const devices = [...previous, device];

      availableDevicesRef.current = devices;
      setAvailableDevices(devices);
    });

    const unregisterDisconnection = BackendListener.onDeviceDisconnected(
      (device) => {
        const previous = availableDevicesRef.current;
        const devices = previous.filter(
          (d) => d.serial_number !== device.serial_number,
        );

        availableDevicesRef.current = devices;
        setAvailableDevices(devices);
      },
    );

    const unregisterUpdateAvailable = BackendListener.onUpdateAvailable(
      (version) => {
        setAvailableUpdate(version);
      },
    );

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
        BackendClient.getTranslations()
          .then((translations) => {
            setTranslations(translations);
          })
          .finally(() => {
            BackendClient.notifyFrontendReady().then(() => {
              setAppReady(true);
            });
          });
      });

    return () => {
      unregisterConnection();
      unregisterDisconnection();
      unregisterUpdateAvailable();
    };
  }, []);

  const translate = (key: string, replacements?: any[]) => {
    if (!translations[key]) {
      console.warn("Missing translation", key);
      return key;
    }
    let translation = translations[key];
    if (replacements) {
      replacements.forEach((r) => {
        translation = translation.replace("{}", r);
      });
    }
    return translation;
  };

  return (
    <AppContext.Provider
      value={{
        tab,
        setTab,
        loading,
        setLoading,
        availableDevices,
        appReady,
        availableUpdate,
        environment,
        translate,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}
