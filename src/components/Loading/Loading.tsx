import { I18nSettingsContext } from "@/context/I18nSettingsContext";
import { Backdrop, CircularProgress } from "@mui/material";
import { useContext } from "react";
import "@/styles/Loading/Loading.css";

export function Loading() {
  const { translate } = useContext(I18nSettingsContext);

  return (
    <Backdrop open={true} className="loading-backdrop">
      <CircularProgress aria-label={translate("loading")} size="5rem" />
    </Backdrop>
  );
}
