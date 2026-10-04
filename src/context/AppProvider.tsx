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
import { AppContext, ExerciseOption } from "./AppContext";
import { LoadingContext } from "./LoadingContext";
import { I18nSettingsContext, KG_TO_LB, KM_TO_MI } from "./I18nSettingsContext";
import { LocalizationProvider } from "@mui/x-date-pickers/LocalizationProvider";
import { AdapterDateFns } from "@mui/x-date-pickers/AdapterDateFns";
import { enUS, es, type Locale } from "date-fns/locale";

// Both locales start the week on Monday
const DATE_LOCALES: Record<Languages, Locale> = {
  [Languages.Spanish]: es,
  [Languages.English]: enUS,
};

const NUMBER_LOCALES: Record<Languages, string> = {
  [Languages.Spanish]: "es-ES",
  [Languages.English]: "en-US",
};

export function AppProvider({
  children,
}: {
  children: JSX.Element;
}): JSX.Element {
  const [environment, setEnvironment] = useState(AppEnvironment.Release);
  const [appReady, setAppReady] = useState(false);
  const [settingsOpened, setSettingsOpened] = useState(false);
  const [tab, setTab] = useState(Tabs.HOME);
  const [rcloneAvailable, setRcloneAvailable] = useState(false);
  const [availableDevices, setAvailableDevices] = useState<DeviceListItem[]>(
    [],
  );
  const availableDevicesRef = useRef<DeviceListItem[]>([]);
  const [loadingCount, setLoadingCount] = useState(0);
  const [sessionsVersion, setSessionsVersion] = useState(0);
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

  const toKm = useCallback(
    (value: number) => {
      switch (settings.distance_unit) {
        case DistanceUnit.Miles:
          return value / KM_TO_MI;
        default:
          return value;
      }
    },
    [settings],
  );

  const fromKm = useCallback(
    (km: number) => {
      switch (settings.distance_unit) {
        case DistanceUnit.Miles:
          return km * KM_TO_MI;
        default:
          return km;
      }
    },
    [settings],
  );

  const toKg = useCallback(
    (value: number) => {
      switch (settings.weight_unit) {
        case WeightUnit.Pounds:
          return value / KG_TO_LB;
        default:
          return value;
      }
    },
    [settings],
  );

  const fromKg = useCallback(
    (kg: number) => {
      switch (settings.weight_unit) {
        case WeightUnit.Pounds:
          return kg * KG_TO_LB;
        default:
          return kg;
      }
    },
    [settings],
  );

  const getDistanceUnit = useCallback(() => {
    switch (settings.distance_unit) {
      case DistanceUnit.Kilometers:
        return "Km";
      case DistanceUnit.Miles:
        return "Mi";
    }
  }, [settings]);

  const getWeightUnit = useCallback(() => {
    switch (settings.weight_unit) {
      case WeightUnit.Kilograms:
        return "Kg";
      case WeightUnit.Pounds:
        return "Lb";
    }
  }, [settings]);

  const formatNumber = useCallback(
    (value: number, decimals: number) =>
      new Intl.NumberFormat(NUMBER_LOCALES[settings.language], {
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
      }).format(value),
    [settings],
  );

  const formatPercent = useCallback(
    (ratio: number, decimals: number, signed = false) =>
      new Intl.NumberFormat(NUMBER_LOCALES[settings.language], {
        style: "percent",
        minimumFractionDigits: decimals,
        maximumFractionDigits: decimals,
        signDisplay: signed ? "always" : "auto",
      }).format(ratio),
    [settings],
  );

  const formatDuration = useCallback((seconds: number) => {
    if (seconds == 0) {
      return "0:00";
    }

    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = Math.floor(seconds % 60);

    let res: string;
    if (h > 0) {
      res = `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    } else {
      res = `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
    }

    while (res.startsWith("0") && !res.startsWith("0:")) {
      res = res.slice(1);
    }

    return res;
  }, []);

  const formatDate = useCallback(
    (date: number) => {
      const datetime = new Date(date * 1000);

      const day = String(datetime.getDate()).padStart(2, "0");
      const month = String(datetime.getMonth() + 1).padStart(2, "0");
      const year = String(datetime.getFullYear()).padStart(4, "0");

      switch (settings.language) {
        case Languages.English:
          return `${month}/${day}/${year}`;
        default:
          return `${day}/${month}/${year}`;
      }
    },
    [settings],
  );

  const formatTimeDate = useCallback(
    (date: number) => {
      const datetime = new Date(date * 1000);

      const hours = String(datetime.getHours()).padStart(2, "0");
      const minutes = String(datetime.getMinutes()).padStart(2, "0");

      return `${hours}:${minutes} ${formatDate(date)}`;
    },
    [formatDate],
  );

  const parseLocalDateTime = useCallback(
    (dateStr: string) => {
      const match = dateStr.match(
        /^(\d{2}):(\d{2}) (\d{2})\/(\d{2})\/(\d{4})$/,
      );

      if (!match) {
        throw new Error("Wrong date format");
      }

      const [, hourStr, minStr, firstStr, secondStr, yearStr] = match;
      const [dayStr, monthStr] =
        settings.language == Languages.English
          ? [secondStr, firstStr]
          : [firstStr, secondStr];

      const hour = Number(hourStr);
      const minute = Number(minStr);
      const day = Number(dayStr);
      const month = Number(monthStr);
      const year = Number(yearStr);

      const local = new Date(year, month - 1, day, hour, minute, 0);

      if (
        local.getFullYear() !== year ||
        local.getMonth() !== month - 1 ||
        local.getDate() !== day ||
        local.getHours() !== hour ||
        local.getMinutes() !== minute
      ) {
        throw new Error("Wrong date format");
      }

      return local;
    },
    [settings],
  );

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

  const exerciseCatalog = useMemo<ExerciseOption[]>(
    () =>
      Object.keys(translations)
        .map((key) => key.match(/^exercise_(\d+)_(\d+)$/))
        .filter((m) => m != null)
        .map((m) => ({
          label: translations[m[0]],
          ex_cat: Number(m[1]),
          ex_id: Number(m[2]),
        }))
        // Sorted by category first, as Autocomplete's groupBy expects grouped options to be contiguous
        .sort(
          (a, b) =>
            (translations["exercise_" + a.ex_cat] ?? "").localeCompare(
              translations["exercise_" + b.ex_cat] ?? "",
            ) || a.label.localeCompare(b.label),
        ),
    [translations],
  );

  const refreshTranslations = useCallback(() => {
    BackendClient.getTranslations().then((translations) => {
      setTranslations(Object.freeze(translations));
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
                BackendClient.rcloneAvailable()
                  .then((available) => {
                    setRcloneAvailable(available);
                  })
                  .finally(() => {
                    BackendClient.notifyFrontendReady().then(() => {
                      setAppReady(true);
                    });
                  });
              });
          });
      });

    return () => {
      unregisterConnection();
      unregisterDisconnection();
      unregisterStartLoading();
      unregisterFinishLoading();
      unregisterSessionsAdded();
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
      sessionsVersion,
      rcloneAvailable,
      exerciseCatalog,
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
      exerciseCatalog,
    ],
  );

  const i18nSettingsValue = useMemo(
    () => ({
      settings,
      translate,
      refreshTranslations,
      toKg,
      fromKg,
      toKm,
      fromKm,
      getDistanceUnit,
      getWeightUnit,
      formatDuration,
      formatDate,
      formatTimeDate,
      parseLocalDateTime,
      formatNumber,
      formatPercent,
    }),
    [
      settings,
      translate,
      refreshTranslations,
      toKg,
      fromKg,
      toKm,
      fromKm,
      getDistanceUnit,
      getWeightUnit,
      formatDuration,
      formatDate,
      formatTimeDate,
      parseLocalDateTime,
      formatNumber,
      formatPercent,
    ],
  );

  return (
    <LoadingContext.Provider value={loadingValue}>
      <I18nSettingsContext.Provider value={i18nSettingsValue}>
        <LocalizationProvider
          dateAdapter={AdapterDateFns}
          adapterLocale={DATE_LOCALES[settings.language]}
        >
          <AppContext.Provider value={appValue}>{children}</AppContext.Provider>
        </LocalizationProvider>
      </I18nSettingsContext.Provider>
    </LoadingContext.Provider>
  );
}
