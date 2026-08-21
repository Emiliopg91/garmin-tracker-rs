import { Tabs } from "@/models/tabs";
import { BackendClient } from "@/utils/backend/client";
import { BackendListener } from "@/utils/backend/listener";
import {
  AppEnvironment,
  DeviceListItem,
  DistanceUnit,
  Settings,
  WeightUnit,
} from "@/utils/backend/models";
import { TRANSLATIONS } from "@/utils/translations";
import { useEffect, useRef, useState } from "react";
import { JSX } from "react/jsx-runtime";
import { AppContext } from "./AppContext";

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const [environment, setEnvironment] = useState(AppEnvironment.Release);
  const [appReady, setAppReady] = useState(false);
  const [tab, setTab] = useState(Tabs.SESSIONS);
  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );
  const availableDevicesRef = useRef<DeviceListItem[]>([]);
  const [loadingCount, setLoadingCount] = useState(0);
  const [settings, setSettings] = useState<Settings>({
    distance_unit: DistanceUnit.Kilometers,
    weight_unit: WeightUnit.Kilograms,
  });

  const startLoading = () => {
    setLoadingCount((previous) => previous + 1);
  };

  const finishLoading = () => {
    setLoadingCount((previous) => Math.max(0, previous - 1));
  };

  const loading = loadingCount > 0;

  const translate = (key: string, replacements?: string[]) => {
    if (!TRANSLATIONS[key]) {
      console.warn("Missing translation", key);
      return key;
    }
    let translation = TRANSLATIONS[key];
    if (replacements) {
      replacements.forEach((r) => {
        translation = translation.replace("{}", r);
      });
    }
    return translation;
  };

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

    const unregisterStartLoading = BackendListener.onStartLoading(() => {
      startLoading();
    });

    const unregisterFinishLoading = BackendListener.onFinishLoading(() => {
      finishLoading();
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
        BackendClient.getSettings()
          .then((settings) => {
            setSettings(settings);
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
      unregisterStartLoading();
      unregisterFinishLoading();
    };
  }, []);

  return (
    <AppContext.Provider
      value={{
        tab,
        setTab,
        startLoading,
        finishLoading,
        loading,
        availableDevices,
        appReady,
        environment,
        translate,
        settings,
      }}
    >
      {children}
    </AppContext.Provider>
  );
}
