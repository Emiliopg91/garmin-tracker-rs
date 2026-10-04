import { Tabs } from "@/models/tabs";
import { BackendClient } from "@/utils/backend/client";
import { BackendListener } from "@/utils/backend/listener";
import { AppEnvironment, DeviceListItem } from "@/utils/backend/models";
import {
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
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
  const [settingsOpened, setSettingsOpened] = useState(false);
  const [tab, setTab] = useState(Tabs.HOME);
  const [rcloneAvailable, setRcloneAvailable] = useState(false);
  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );
  const availableDevicesRef = useRef<DeviceListItem[]>([]);
  const [sessionsVersion, setSessionsVersion] = useState(0);

  const showSettings = useCallback(() => {
    setSettingsOpened(true);
  }, []);

  const closeSettings = useCallback(() => {
    setSettingsOpened(false);
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
        BackendClient.rcloneAvailable()
          .then((available) => {
            setRcloneAvailable(available);
          })
          .finally(() => {
            setBackendReady(true);
          });
      });

    return () => {
      unregisterConnection();
      unregisterDisconnection();
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
      tab,
      setTab,
      appReady,
      environment,
      availableDevices,
      settingsOpened,
      showSettings,
      closeSettings,
      sessionsVersion,
      rcloneAvailable,
    }),
    [
      tab,
      appReady,
      environment,
      availableDevices,
      settingsOpened,
      showSettings,
      closeSettings,
      sessionsVersion,
      rcloneAvailable,
    ],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}
