import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { Backdrop, CircularProgress } from "@mui/material";
import { useContext } from "react";
import "@/styles/Loading/Loading.css";
import { LoadingStateContext } from "@/context/LoadingContext";
import { AppContext } from "@/context/AppContext";

export function Loading() {
  const { translate } = useContext(I18nSettingsContext);
  const { appReady } = useContext(AppContext);

  const loading = useContext(LoadingStateContext);

  if (appReady && !loading) {
    return null;
  }
  return (
    <Backdrop open={true} className="loading-backdrop">
      <CircularProgress aria-label={translate("loading")} size="5rem" />
    </Backdrop>
  );
}
