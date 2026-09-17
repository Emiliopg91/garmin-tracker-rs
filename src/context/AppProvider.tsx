import { Tabs } from "@/models/tabs";
import { BackendClient } from "@/utils/backend/client";
import { BackendListener } from "@/utils/backend/listener";
import {
  AppEnvironment,
  DeviceListItem,
  DistanceUnit,
  Languages,
  Settings,
  WeightUnit,
} from "@/utils/backend/models";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { JSX } from "react/jsx-runtime";
import { AppContext } from "./AppContext";
import { LoadingContext } from "./LoadingContext";
import { I18nSettingsContext } from "./I18nSettingsContext";

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const [environment, setEnvironment] = useState(AppEnvironment.Release);
  const [appReady, setAppReady] = useState(false);
  const [settingsOpened, setSettingsOpened] = useState(false);
  const [tab, setTab] = useState(Tabs.HOME);
  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );
  const availableDevicesRef = useRef<DeviceListItem[]>([]);
  const [loadingCount, setLoadingCount] = useState(0);
  const [settings, setSettings] = useState<Settings>({
    distance_unit: DistanceUnit.Kilometers,
    weight_unit: WeightUnit.Kilograms,
    auto_sync: true,
    start_boot: false,
    language: Languages.English,
    on_device_connect: false,
  });
  const [translations, setTranslations] = useState<Record<string, string>>({});

  const startLoading = useCallback(() => {
    setLoadingCount((previous) => previous + 1);
  }, []);

  const finishLoading = useCallback(() => {
    setLoadingCount((previous) => Math.max(0, previous - 1));
  }, []);

  const showSettings = useCallback(() => {
    setSettingsOpened(true);
  }, []);

  const closeSettings = useCallback(() => {
    setSettingsOpened(false);
  }, []);

  const loading = loadingCount > 0;

  const translate = useCallback(
    (key: string, replacements?: string[]) => {
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
    },
    [translations],
  );

  const refreshTranslations = useCallback(() => {
    BackendClient.getTranslations().then((translations) => {
      setTranslations(translations);
    });
  }, []);

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
      });

    return () => {
      unregisterConnection();
      unregisterDisconnection();
      unregisterStartLoading();
      unregisterFinishLoading();
    };
  }, [startLoading, finishLoading]);

  const loadingValue = useMemo(
    () => ({ loading, startLoading, finishLoading }),
    [loading, startLoading, finishLoading],
  );

  const appValue = useMemo(
    () => ({
      tab,
      setTab,
      appReady,
      environment,
      availableDevices,
      settingsOpened,
      showSettings,
      closeSettings,
    }),
    [
      tab,
      appReady,
      environment,
      availableDevices,
      settingsOpened,
      showSettings,
      closeSettings,
    ],
  );

  const i18nSettingsValue = useMemo(
    () => ({ settings, translate, refreshTranslations }),
    [settings, translate, refreshTranslations],
  );

  return (
    <LoadingContext.Provider value={loadingValue}>
      <I18nSettingsContext.Provider value={i18nSettingsValue}>
        <AppContext.Provider value={appValue}>{children}</AppContext.Provider>
      </I18nSettingsContext.Provider>
    </LoadingContext.Provider>
  );
}
